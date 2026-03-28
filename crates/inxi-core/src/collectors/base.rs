//! Collectors for baseline system information.
//!
//! These sections rely almost entirely on procfs, sysfs, and environment data.
//! They form the safest and most portable slice of the current collector set.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    CapabilityReport, CpuSection, CpuSpeedInfo, CpuTopology, DataState, DesktopInfo, DistroInfo,
    FirmwareInfo, InfoSection, KernelInfo, MachineSection, MemorySection, MotherboardInfo,
    SectionData, SectionEnvelope, SourceTrace, SwapSection, SystemSection, Warning,
};
use crate::util::{
    ReadOutcome, basename, env_var, parse_key_value_lines, read_dir, read_text, read_trimmed,
};

pub(crate) fn collect_system(capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let os_release_text = match read_text("/etc/os-release") {
        ReadOutcome::Present(text) => Some(text),
        _ => None,
    };
    let os_release = os_release_text
        .as_deref()
        .map(parse_key_value_lines)
        .unwrap_or_default();

    if os_release.is_empty() {
        warnings.push(Warning::new(
            "missing_file",
            "Could not read /etc/os-release; distro identification is partial.",
            Some("system"),
            Some("/etc/os-release"),
        ));
    }

    let desktop = build_desktop_info(capabilities);
    let envelope = SectionEnvelope::ok(
        SystemSection {
            kernel: KernelInfo {
                os_type: read_trimmed("/proc/sys/kernel/ostype").into_option(),
                release: read_trimmed("/proc/sys/kernel/osrelease").into_option(),
                version: read_trimmed("/proc/sys/kernel/version").into_option(),
                architecture: std::env::consts::ARCH.to_string(),
            },
            distro: DistroInfo {
                id: os_release.get("ID").cloned(),
                name: os_release.get("NAME").cloned(),
                version: os_release.get("VERSION_ID").cloned(),
                codename: os_release.get("VERSION_CODENAME").cloned(),
                pretty_name: os_release.get("PRETTY_NAME").cloned(),
            },
            hostname: capabilities.hostname.clone(),
            desktop,
        },
        vec![
            SourceTrace::procfs("/proc/sys/kernel/ostype"),
            SourceTrace::procfs("/proc/sys/kernel/osrelease"),
            SourceTrace::procfs("/proc/sys/kernel/version"),
            SourceTrace::procfs("/proc/sys/kernel/hostname"),
            SourceTrace::file("/etc/os-release"),
            SourceTrace::env("XDG_CURRENT_DESKTOP"),
            SourceTrace::env("XDG_SESSION_TYPE"),
        ],
    );

    (SectionData::System(envelope), warnings)
}

pub(crate) fn collect_machine(
    capabilities: &CapabilityReport,
    filter_sensitive: bool,
) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let base = "/sys/class/dmi/id";
    let fields = [
        "sys_vendor",
        "product_name",
        "product_version",
        "product_family",
        "product_serial",
        "board_vendor",
        "board_name",
        "board_version",
        "bios_vendor",
        "bios_version",
        "bios_date",
    ]
    .into_iter()
    .map(|field| (field, read_trimmed(format!("{base}/{field}"))))
    .collect::<BTreeMap<_, _>>();

    let values_found = fields
        .values()
        .any(|value| matches!(value, ReadOutcome::Present(_)));
    let permission_denied = fields
        .values()
        .all(|value| matches!(value, ReadOutcome::PermissionDenied));

    if !capabilities.paths.get(base).copied().unwrap_or(false) {
        warnings.push(Warning::new(
            "missing_path",
            "DMI sysfs tree is missing; machine details are unavailable.",
            Some("machine"),
            Some(base),
        ));
    } else if permission_denied {
        warnings.push(Warning::new(
            "permission_required",
            "DMI data exists but current privileges are insufficient to read it.",
            Some("machine"),
            Some(base),
        ));
    }

    let serial = match fields.get("product_serial") {
        Some(ReadOutcome::Present(value)) if !filter_sensitive => Some(value.clone()),
        Some(ReadOutcome::Present(_)) => Some("[filtered]".to_string()),
        _ => None,
    };

    let state = if values_found {
        DataState::Ok
    } else if permission_denied {
        DataState::PermissionRequired
    } else {
        DataState::Missing
    };

    let envelope = SectionEnvelope {
        state,
        value: values_found.then_some(MachineSection {
            vendor: fields
                .get("sys_vendor")
                .cloned()
                .and_then(ReadOutcome::into_option),
            product_name: fields
                .get("product_name")
                .cloned()
                .and_then(ReadOutcome::into_option),
            product_version: fields
                .get("product_version")
                .cloned()
                .and_then(ReadOutcome::into_option),
            product_family: fields
                .get("product_family")
                .cloned()
                .and_then(ReadOutcome::into_option),
            serial,
            board: MotherboardInfo {
                vendor: fields
                    .get("board_vendor")
                    .cloned()
                    .and_then(ReadOutcome::into_option),
                name: fields
                    .get("board_name")
                    .cloned()
                    .and_then(ReadOutcome::into_option),
                version: fields
                    .get("board_version")
                    .cloned()
                    .and_then(ReadOutcome::into_option),
            },
            firmware: FirmwareInfo {
                vendor: fields
                    .get("bios_vendor")
                    .cloned()
                    .and_then(ReadOutcome::into_option),
                version: fields
                    .get("bios_version")
                    .cloned()
                    .and_then(ReadOutcome::into_option),
                date: fields
                    .get("bios_date")
                    .cloned()
                    .and_then(ReadOutcome::into_option),
            },
        }),
        sources: vec![SourceTrace::sysfs(base)],
    };

    (SectionData::Machine(envelope), warnings)
}

pub(crate) fn collect_cpu(_capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let cpuinfo_text = match read_text("/proc/cpuinfo") {
        ReadOutcome::Present(text) => text,
        _ => {
            warnings.push(Warning::new(
                "missing_file",
                "Could not read /proc/cpuinfo; CPU section is unavailable.",
                Some("cpu"),
                Some("/proc/cpuinfo"),
            ));
            return (
                SectionData::Cpu(SectionEnvelope::without_value(
                    DataState::Missing,
                    vec![SourceTrace::procfs("/proc/cpuinfo")],
                )),
                warnings,
            );
        }
    };

    let entries = parse_cpuinfo_entries(&cpuinfo_text);
    if entries.is_empty() {
        warnings.push(Warning::new(
            "parse_error",
            "Parsed /proc/cpuinfo but found no processor entries.",
            Some("cpu"),
            Some("/proc/cpuinfo"),
        ));
        return (
            SectionData::Cpu(SectionEnvelope::without_value(
                DataState::Unknown,
                vec![SourceTrace::procfs("/proc/cpuinfo")],
            )),
            warnings,
        );
    }

    let first = &entries[0];
    let logical_cpus = entries.len() as u32;
    let physical_packages = count_unique_u32(&entries, "physical id");
    let cores_per_package = first.get("cpu cores").and_then(|value| value.parse().ok());
    let siblings = first
        .get("siblings")
        .and_then(|value| value.parse::<u32>().ok());
    let threads_per_core = siblings
        .zip(cores_per_package)
        .and_then(|(siblings, cores)| (cores != 0).then_some(siblings / cores))
        .or_else(|| {
            // Some kernels or architectures omit `siblings`, so we keep a
            // second inference path that uses total logical CPUs instead.
            physical_packages
                .zip(cores_per_package)
                .and_then(|(packages, cores)| {
                    let denominator = packages.saturating_mul(cores);
                    (denominator != 0).then_some(logical_cpus / denominator)
                })
        });

    let current_mhz = first.get("cpu MHz").and_then(|value| value.parse().ok());
    let min_mhz = read_frequency_mhz("/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq")
        .or_else(|| read_frequency_mhz("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq"));
    let max_mhz = read_frequency_mhz("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq")
        .or_else(|| read_frequency_mhz("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq"));

    let envelope = SectionEnvelope::ok(
        CpuSection {
            vendor: first.get("vendor_id").cloned(),
            model_name: first.get("model name").cloned(),
            architecture: std::env::consts::ARCH.to_string(),
            topology: CpuTopology {
                logical_cpus,
                physical_packages,
                cores_per_package,
                threads_per_core,
            },
            speed: CpuSpeedInfo {
                current_mhz,
                min_mhz,
                max_mhz,
            },
        },
        vec![
            SourceTrace::procfs("/proc/cpuinfo"),
            SourceTrace::sysfs("/sys/devices/system/cpu/cpu0/cpufreq"),
            SourceTrace::derived("logical CPU and topology inference"),
        ],
    );

    (SectionData::Cpu(envelope), warnings)
}

pub(crate) fn collect_memory(_capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let meminfo = match read_text("/proc/meminfo") {
        ReadOutcome::Present(text) => parse_meminfo(&text),
        _ => {
            warnings.push(Warning::new(
                "missing_file",
                "Could not read /proc/meminfo; memory section is unavailable.",
                Some("memory"),
                Some("/proc/meminfo"),
            ));
            return (
                SectionData::Memory(SectionEnvelope::without_value(
                    DataState::Missing,
                    vec![SourceTrace::procfs("/proc/meminfo")],
                )),
                warnings,
            );
        }
    };

    let Some(total_bytes) = meminfo.get("MemTotal").copied() else {
        warnings.push(Warning::new(
            "parse_error",
            "MemTotal is missing from /proc/meminfo.",
            Some("memory"),
            Some("/proc/meminfo"),
        ));
        return (
            SectionData::Memory(SectionEnvelope::without_value(
                DataState::Unknown,
                vec![SourceTrace::procfs("/proc/meminfo")],
            )),
            warnings,
        );
    };

    let envelope = SectionEnvelope::ok(
        MemorySection {
            total_bytes,
            available_bytes: meminfo.get("MemAvailable").copied(),
            used_bytes: meminfo
                .get("MemAvailable")
                .copied()
                .map(|available| total_bytes.saturating_sub(available)),
            swap_total_bytes: meminfo.get("SwapTotal").copied(),
            swap_free_bytes: meminfo.get("SwapFree").copied(),
        },
        vec![SourceTrace::procfs("/proc/meminfo")],
    );

    (SectionData::Memory(envelope), warnings)
}

pub(crate) fn collect_swap(_capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let swaps_text = match read_text("/proc/swaps") {
        ReadOutcome::Present(text) => text,
        _ => {
            warnings.push(Warning::new(
                "missing_file",
                "Could not read /proc/swaps; swap section is unavailable.",
                Some("swap"),
                Some("/proc/swaps"),
            ));
            return (
                SectionData::Swap(SectionEnvelope::without_value(
                    DataState::Missing,
                    vec![SourceTrace::procfs("/proc/swaps")],
                )),
                warnings,
            );
        }
    };

    let devices = parse_swaps(&swaps_text);
    if devices.is_empty() {
        return (
            SectionData::Swap(SectionEnvelope::without_value(
                DataState::Missing,
                vec![SourceTrace::procfs("/proc/swaps")],
            )),
            warnings,
        );
    }

    let envelope = SectionEnvelope::ok(
        SwapSection { devices },
        vec![SourceTrace::procfs("/proc/swaps")],
    );
    (SectionData::Swap(envelope), warnings)
}

pub(crate) fn collect_info(_capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let uptime_seconds = match read_text("/proc/uptime") {
        ReadOutcome::Present(text) => text
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .map(|seconds| seconds as u64),
        _ => None,
    };

    let process_count = read_dir("/proc").ok().map(|entries| {
        entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .chars()
                    .all(|character| character.is_ascii_digit())
            })
            .count() as u64
    });

    let envelope = SectionEnvelope::ok(
        InfoSection {
            uptime_seconds,
            shell: env_var("SHELL").map(|value| basename(&value)),
            user: env_var("USER").or_else(|| env_var("LOGNAME")),
            terminal: env_var("TERM"),
            locale: env_var("LANG"),
            process_count,
        },
        vec![
            SourceTrace::procfs("/proc/uptime"),
            SourceTrace::procfs("/proc"),
            SourceTrace::env("SHELL"),
            SourceTrace::env("USER"),
            SourceTrace::env("TERM"),
            SourceTrace::env("LANG"),
        ],
    );

    (SectionData::Info(envelope), Vec::new())
}

fn build_desktop_info(capabilities: &CapabilityReport) -> Option<DesktopInfo> {
    let session_type = env_var("XDG_SESSION_TYPE");
    let current_desktop = env_var("XDG_CURRENT_DESKTOP");
    let window_manager = env_var("DESKTOP_SESSION");
    let display_server = capabilities.display_protocol.clone();

    let has_any = session_type.is_some()
        || current_desktop.is_some()
        || window_manager.is_some()
        || display_server.is_some();

    // We only return desktop info when at least one signal exists so consumers
    // can distinguish "not applicable" from "present but empty".
    has_any.then_some(DesktopInfo {
        session_type,
        current_desktop,
        window_manager,
        display_server,
    })
}

pub(crate) fn parse_swaps(input: &str) -> Vec<crate::model::SwapDevice> {
    input
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 5 {
                return None;
            }

            let size_kib = parts[2].parse::<u64>().ok()?;
            let used_kib = parts[3].parse::<u64>().ok();
            let priority = parts[4].parse::<i32>().ok();

            Some(crate::model::SwapDevice {
                path: parts[0].to_string(),
                size_bytes: size_kib.saturating_mul(1024),
                used_bytes: used_kib.map(|value| value.saturating_mul(1024)),
                priority,
            })
        })
        .collect()
}

pub(crate) fn parse_cpuinfo_entries(input: &str) -> Vec<BTreeMap<String, String>> {
    input
        .split("\n\n")
        .filter_map(|block| {
            let entry = block
                .lines()
                .filter_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    Some((key.trim().to_string(), value.trim().to_string()))
                })
                .collect::<BTreeMap<_, _>>();

            (!entry.is_empty()).then_some(entry)
        })
        .collect()
}

fn count_unique_u32(entries: &[BTreeMap<String, String>], key: &str) -> Option<u32> {
    let values = entries
        .iter()
        .filter_map(|entry| entry.get(key))
        .filter_map(|value| value.parse::<u32>().ok())
        .collect::<BTreeSet<_>>();

    (!values.is_empty()).then_some(values.len() as u32)
}

fn read_frequency_mhz(path: &str) -> Option<f64> {
    read_trimmed(path)
        .into_option()
        .and_then(|value| value.parse::<f64>().ok())
        .map(|khz| khz / 1000.0)
}

pub(crate) fn parse_meminfo(input: &str) -> BTreeMap<String, u64> {
    input
        .lines()
        .filter_map(|line| {
            let (key, rest) = line.split_once(':')?;
            let mut parts = rest.split_whitespace();
            let value = parts.next()?.parse::<u64>().ok()?;
            let unit = parts.next().unwrap_or("kB");
            let bytes = match unit {
                "kB" => value.saturating_mul(1024),
                "mB" | "MB" => value.saturating_mul(1024 * 1024),
                "gB" | "GB" => value.saturating_mul(1024 * 1024 * 1024),
                _ => value,
            };
            Some((key.trim().to_string(), bytes))
        })
        .collect()
}
