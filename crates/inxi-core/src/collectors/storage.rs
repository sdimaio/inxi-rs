//! Storage collectors and parsers.
//!
//! Storage is the main place where we intentionally combine rich command output
//! with procfs fallbacks. `lsblk` gives us the best structured view when it is
//! available, but the clone must still remain useful on minimal systems.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::command::{AuditedCommand, run_audited_command};
use crate::model::{
    CapabilityReport, DataState, DrivesSection, PartitionEntry, PartitionsSection, PhysicalDisk,
    SectionData, SectionEnvelope, SourceTrace, Warning,
};
use crate::util::{ReadOutcome, path_exists, read_text, read_trimmed};

use super::{clean_option, command_warning, maybe_filter_mountpoint, maybe_filter_uuid};

pub(crate) fn collect_drives(capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let mut sources = vec![SourceTrace::sysfs("/sys/block")];

    let drives = match read_lsblk(capabilities, "drives") {
        Ok(lsblk) => {
            sources.push(SourceTrace::command(AuditedCommand::LsblkJson.label()));
            lsblk
                .blockdevices
                .iter()
                .filter(|device| device.device_type == "disk")
                .map(build_drive)
                .collect::<Vec<_>>()
        }
        Err(warning) => {
            warnings.push(warning);
            sources.push(SourceTrace::procfs("/proc/partitions"));
            warnings.push(Warning::new(
                "fallback_used",
                "Drive discovery fell back to /proc/partitions and /sys/block; model metadata may be partial.",
                Some("drives"),
                Some("/proc/partitions"),
            ));
            collect_drives_from_proc()
        }
    };

    if drives.is_empty() {
        return (
            SectionData::Drives(SectionEnvelope::without_value(DataState::Missing, sources)),
            warnings,
        );
    }

    let envelope = SectionEnvelope::ok(DrivesSection { drives }, sources);
    (SectionData::Drives(envelope), warnings)
}

pub(crate) fn collect_partitions(
    capabilities: &CapabilityReport,
    filter_sensitive: bool,
) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let mut sources = Vec::new();

    let partitions = match read_lsblk(capabilities, "partitions") {
        Ok(lsblk) => {
            sources.push(SourceTrace::command(AuditedCommand::LsblkJson.label()));
            let mut partitions = Vec::new();
            for device in &lsblk.blockdevices {
                collect_partition_entries(device, None, filter_sensitive, &mut partitions);
            }
            partitions
        }
        Err(warning) => {
            warnings.push(warning);
            sources.push(SourceTrace::procfs("/proc/partitions"));
            sources.push(SourceTrace::procfs("/proc/self/mounts"));
            warnings.push(Warning::new(
                "fallback_used",
                "Partition discovery fell back to /proc/partitions and /proc/self/mounts; UUID data is unavailable.",
                Some("partitions"),
                Some("/proc/partitions"),
            ));
            collect_partitions_from_proc(filter_sensitive)
        }
    };

    if partitions.is_empty() {
        return (
            SectionData::Partitions(SectionEnvelope::without_value(DataState::Missing, sources)),
            warnings,
        );
    }
    sources.push(SourceTrace::sysfs("/sys/block"));
    let envelope = SectionEnvelope::ok(PartitionsSection { partitions }, sources);
    (SectionData::Partitions(envelope), warnings)
}

fn read_lsblk(capabilities: &CapabilityReport, section: &str) -> Result<LsblkOutput, Warning> {
    if !capabilities.commands.get("lsblk").copied().unwrap_or(false) {
        return Err(Warning::new(
            "missing_tool",
            "The audited lsblk command is unavailable.",
            Some(section),
            Some(AuditedCommand::LsblkJson.label()),
        ));
    }

    let output = run_audited_command(AuditedCommand::LsblkJson)
        .map_err(|error| command_warning(section, AuditedCommand::LsblkJson, &error))?;

    serde_json::from_str::<LsblkOutput>(&output.stdout).map_err(|error| {
        Warning::new(
            "parse_error",
            format!("Failed to parse lsblk JSON output: {error}"),
            Some(section),
            Some(AuditedCommand::LsblkJson.label()),
        )
    })
}

fn build_drive(device: &LsblkDevice) -> PhysicalDisk {
    let kname = &device.kname;
    PhysicalDisk {
        name: device.name.clone(),
        path: format!("/dev/{kname}"),
        size_bytes: device.size,
        model: clean_option(device.model.clone()),
        vendor: clean_option(device.vendor.clone()),
        rotational: read_trimmed(format!("/sys/block/{kname}/queue/rotational"))
            .into_option()
            .and_then(|value| match value.as_str() {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            }),
        removable: read_trimmed(format!("/sys/block/{kname}/removable"))
            .into_option()
            .and_then(|value| match value.as_str() {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            }),
    }
}

fn collect_drives_from_proc() -> Vec<PhysicalDisk> {
    let ReadOutcome::Present(text) = read_text("/proc/partitions") else {
        return Vec::new();
    };

    parse_proc_partitions(&text)
        .into_iter()
        .filter(|record| !is_virtual_block_device(&record.name))
        .filter(|record| !is_partition_block_device(&record.name))
        .map(build_drive_from_proc)
        .collect()
}

fn build_drive_from_proc(record: ProcPartitionRecord) -> PhysicalDisk {
    let name = record.name;
    let sysfs_base = format!("/sys/class/block/{name}");

    PhysicalDisk {
        name: name.clone(),
        path: format!("/dev/{name}"),
        size_bytes: Some(record.blocks_kib.saturating_mul(1024)),
        model: read_trimmed(format!("{sysfs_base}/device/model")).into_option(),
        vendor: read_trimmed(format!("{sysfs_base}/device/vendor")).into_option(),
        rotational: read_trimmed(format!("{sysfs_base}/queue/rotational"))
            .into_option()
            .and_then(|value| match value.as_str() {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            }),
        removable: read_trimmed(format!("{sysfs_base}/removable"))
            .into_option()
            .and_then(|value| match value.as_str() {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            }),
    }
}

fn collect_partition_entries(
    device: &LsblkDevice,
    parent: Option<&str>,
    filter_sensitive: bool,
    out: &mut Vec<PartitionEntry>,
) {
    if device.device_type == "part" {
        out.push(PartitionEntry {
            name: device.name.clone(),
            path: format!("/dev/{}", device.kname),
            filesystem: clean_option(device.fstype.clone()),
            mountpoint: device
                .mountpoint
                .as_deref()
                .map(|mountpoint| maybe_filter_mountpoint(mountpoint, filter_sensitive)),
            uuid: device
                .uuid
                .as_deref()
                .map(|uuid| maybe_filter_uuid(uuid, filter_sensitive)),
            size_bytes: device.size,
            filesystem_size_bytes: parse_optional_u64(device.filesystem_size.as_deref()),
            available_bytes: parse_optional_u64(device.filesystem_available.as_deref()),
            used_bytes: parse_optional_u64(device.filesystem_used.as_deref()),
            used_percent: parse_optional_percent(device.filesystem_used_percent.as_deref()),
            parent: parent.map(ToOwned::to_owned),
        });
    }

    // We recurse through the lsblk tree because children are the only reliable
    // way to preserve parent-child relationships across SATA, NVMe, and other
    // naming schemes.
    for child in &device.children {
        collect_partition_entries(child, Some(&device.kname), filter_sensitive, out);
    }
}

fn collect_partitions_from_proc(filter_sensitive: bool) -> Vec<PartitionEntry> {
    let ReadOutcome::Present(partitions_text) = read_text("/proc/partitions") else {
        return Vec::new();
    };
    let mounts = match read_text("/proc/self/mounts") {
        ReadOutcome::Present(text) => parse_proc_mounts(&text),
        _ => BTreeMap::new(),
    };

    parse_proc_partitions(&partitions_text)
        .into_iter()
        .filter(|record| !is_virtual_block_device(&record.name))
        .filter(|record| is_partition_block_device(&record.name))
        .map(|record| {
            let device_path = format!("/dev/{}", record.name);
            let mount = mounts.get(&device_path);

            PartitionEntry {
                name: record.name.clone(),
                path: device_path,
                filesystem: mount.map(|record| record.filesystem.clone()),
                mountpoint: mount
                    .map(|record| maybe_filter_mountpoint(&record.mountpoint, filter_sensitive)),
                uuid: None,
                size_bytes: Some(record.blocks_kib.saturating_mul(1024)),
                filesystem_size_bytes: None,
                available_bytes: None,
                used_bytes: None,
                used_percent: None,
                parent: infer_parent_block(&record.name),
            }
        })
        .collect()
}

fn parse_optional_u64(value: Option<&str>) -> Option<u64> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok())
}

fn parse_optional_percent(value: Option<&str>) -> Option<u8> {
    value
        .map(str::trim)
        .and_then(|value| value.strip_suffix('%').unwrap_or(value).parse::<u8>().ok())
}

pub(crate) fn parse_proc_partitions(input: &str) -> Vec<ProcPartitionRecord> {
    input
        .lines()
        .skip(2)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() != 4 {
                return None;
            }

            Some(ProcPartitionRecord {
                name: parts[3].to_string(),
                blocks_kib: parts[2].parse().ok()?,
            })
        })
        .collect()
}

pub(crate) fn parse_proc_mounts(input: &str) -> BTreeMap<String, ProcMountRecord> {
    input
        .lines()
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 3 {
                return None;
            }

            Some((
                unescape_mount_field(parts[0]),
                ProcMountRecord {
                    mountpoint: unescape_mount_field(parts[1]),
                    filesystem: parts[2].to_string(),
                },
            ))
        })
        .collect()
}

fn unescape_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

fn is_partition_block_device(name: &str) -> bool {
    path_exists(format!("/sys/class/block/{name}/partition")) || infer_parent_block(name).is_some()
}

fn is_virtual_block_device(name: &str) -> bool {
    ["loop", "ram", "zram", "dm-", "md", "sr"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

pub(crate) fn infer_parent_block(name: &str) -> Option<String> {
    // NVMe and mmc devices use the `pN` suffix convention, while classic disk
    // names append digits directly. We support both forms explicitly because
    // trying to infer them from regex alone is easy to get subtly wrong.
    if let Some((base, suffix)) = name.rsplit_once('p')
        && !base.is_empty()
        && base
            .chars()
            .last()
            .map(|character| character.is_ascii_digit())
            .unwrap_or(false)
        && !suffix.is_empty()
        && suffix.chars().all(|character| character.is_ascii_digit())
    {
        return Some(base.to_string());
    }

    let split_at = name
        .rfind(|character: char| !character.is_ascii_digit())
        .map(|index| index + 1)
        .unwrap_or(0);
    if split_at == 0 || split_at == name.len() {
        return None;
    }

    let base = &name[..split_at];
    let suffix = &name[split_at..];
    if !suffix.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    ["sd", "hd", "vd", "xvd"]
        .iter()
        .any(|prefix| base.starts_with(prefix))
        .then(|| base.to_string())
}

#[derive(Debug, Deserialize)]
pub(crate) struct LsblkOutput {
    #[serde(default)]
    pub(crate) blockdevices: Vec<LsblkDevice>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LsblkDevice {
    pub(crate) name: String,
    pub(crate) kname: String,
    #[serde(rename = "type")]
    pub(crate) device_type: String,
    pub(crate) size: Option<u64>,
    pub(crate) fstype: Option<String>,
    pub(crate) mountpoint: Option<String>,
    pub(crate) uuid: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) vendor: Option<String>,
    #[serde(rename = "fsavail")]
    pub(crate) filesystem_available: Option<String>,
    #[serde(rename = "fssize")]
    pub(crate) filesystem_size: Option<String>,
    #[serde(rename = "fsused")]
    pub(crate) filesystem_used: Option<String>,
    #[serde(rename = "fsuse%")]
    pub(crate) filesystem_used_percent: Option<String>,
    #[serde(default)]
    pub(crate) children: Vec<LsblkDevice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcPartitionRecord {
    pub(crate) name: String,
    pub(crate) blocks_kib: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcMountRecord {
    pub(crate) mountpoint: String,
    pub(crate) filesystem: String,
}
