//! Collector entry points and shared helpers.
//!
//! The root module intentionally only contains glue and cross-domain helpers.
//! Domain-specific collection logic lives in submodules so future growth does
//! not turn collector maintenance back into a single-file problem.

mod base;
mod graphics;
mod network;
mod storage;

use crate::command::{AuditedCommand, CommandError};
use crate::model::Warning;

pub(crate) use base::{
    collect_cpu, collect_info, collect_machine, collect_memory, collect_swap, collect_system,
};
pub(crate) use graphics::collect_graphics;
pub(crate) use network::collect_network;
pub(crate) use storage::{collect_drives, collect_partitions};

fn clean_option(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "-" {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn push_unique(values: &mut Vec<String>, candidate: String) {
    if !values.iter().any(|value| value == &candidate) {
        values.push(candidate);
    }
}

fn maybe_filter_ip(value: &str, filter_sensitive: bool) -> &str {
    if filter_sensitive {
        "[filtered]"
    } else {
        value
    }
}

fn maybe_filter_mac(value: &str, filter_sensitive: bool) -> String {
    if filter_sensitive {
        "[filtered]".to_string()
    } else {
        value.to_string()
    }
}

fn maybe_filter_uuid(value: &str, filter_sensitive: bool) -> String {
    if filter_sensitive {
        "[filtered]".to_string()
    } else {
        value.to_string()
    }
}

fn maybe_filter_mountpoint(value: &str, filter_sensitive: bool) -> String {
    if !filter_sensitive {
        return value.to_string();
    }

    // We only rewrite mount paths that commonly embed usernames. Rewriting more
    // aggressively would make output less useful while not adding much privacy.
    if let Some(rest) = value.strip_prefix("/home/") {
        return rest
            .split_once('/')
            .map(|(_, suffix)| format!("/home/[filtered]/{suffix}"))
            .unwrap_or_else(|| "/home/[filtered]".to_string());
    }
    if let Some(rest) = value.strip_prefix("/media/") {
        return rest
            .split_once('/')
            .map(|(_, suffix)| format!("/media/[filtered]/{suffix}"))
            .unwrap_or_else(|| "/media/[filtered]".to_string());
    }

    if let Some(rest) = value.strip_prefix("/run/media/") {
        return rest
            .split_once('/')
            .map(|(_, suffix)| format!("/run/media/[filtered]/{suffix}"))
            .unwrap_or_else(|| "/run/media/[filtered]".to_string());
    }

    value.to_string()
}

fn command_warning(section: &str, command: AuditedCommand, error: &CommandError) -> Warning {
    match error {
        CommandError::NotInstalled(_) => Warning::new(
            "missing_tool",
            format!("Audited command '{}' is unavailable.", command.label()),
            Some(section),
            Some(command.label()),
        ),
        CommandError::PolicyViolation(message) => Warning::new(
            "command_policy",
            format!(
                "Audited command '{}' was blocked: {message}.",
                command.label()
            ),
            Some(section),
            Some(command.label()),
        ),
        CommandError::SpawnFailed(message) | CommandError::WaitFailed(message) => Warning::new(
            "command_error",
            format!("Audited command '{}' failed: {message}.", command.label()),
            Some(section),
            Some(command.label()),
        ),
        CommandError::TimedOut(_) => Warning::new(
            "command_timeout",
            format!("Audited command '{}' timed out.", command.label()),
            Some(section),
            Some(command.label()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{DisplayOutput, NetworkInterface};
    use std::collections::BTreeMap;

    use super::{
        base::{parse_cpuinfo_entries, parse_meminfo, parse_swaps},
        graphics::{
            canonical_display_name, extract_quoted_fields, merge_displays, parse_lspci_gpus,
            parse_xrandr_displays,
        },
        maybe_filter_mountpoint,
        network::{apply_ip_brief_addresses, parse_proc_if_inet6},
        storage::{
            LsblkOutput, ProcMountRecord, ProcPartitionRecord, infer_parent_block,
            parse_proc_mounts, parse_proc_partitions,
        },
    };

    const IP_BRIEF_FIXTURE: &str = include_str!("../tests/fixtures/ip-brief-address.txt");
    const LSBLK_FIXTURE: &str = include_str!("../tests/fixtures/lsblk.json");
    const LSPCI_FIXTURE: &str = include_str!("../tests/fixtures/lspci-mm.txt");
    const PROC_IF_INET6_FIXTURE: &str = include_str!("../tests/fixtures/proc-if_inet6.txt");
    const PROC_MOUNTS_FIXTURE: &str = include_str!("../tests/fixtures/proc-mounts.txt");
    const PROC_PARTITIONS_FIXTURE: &str = include_str!("../tests/fixtures/proc-partitions.txt");
    const XRANDR_FIXTURE: &str = include_str!("../tests/fixtures/xrandr-query.txt");

    #[test]
    fn parses_meminfo_values_in_bytes() {
        let parsed = parse_meminfo(
            "MemTotal:       16384256 kB\nMemAvailable:   12000000 kB\nSwapTotal:       2097148 kB\n",
        );

        assert_eq!(parsed.get("MemTotal"), Some(&(16384256_u64 * 1024)));
        assert_eq!(parsed.get("MemAvailable"), Some(&(12000000_u64 * 1024)));
        assert_eq!(parsed.get("SwapTotal"), Some(&(2097148_u64 * 1024)));
    }

    #[test]
    fn parses_cpuinfo_entries() {
        let parsed = parse_cpuinfo_entries(
            "processor\t: 0\nvendor_id\t: GenuineIntel\nmodel name\t: Example CPU\ncpu cores\t: 8\n\nprocessor\t: 1\nvendor_id\t: GenuineIntel\nmodel name\t: Example CPU\ncpu cores\t: 8\n",
        );

        assert_eq!(parsed.len(), 2);
        assert_eq!(
            parsed[0].get("vendor_id").map(String::as_str),
            Some("GenuineIntel")
        );
        assert_eq!(parsed[1].get("processor").map(String::as_str), Some("1"));
    }

    #[test]
    fn parses_proc_swaps() {
        let parsed = parse_swaps(
            "Filename\t\tType\t\tSize\tUsed\tPriority\n/swapfile                               file\t2097148\t1024\t-2\n",
        );

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].path, "/swapfile");
        assert_eq!(parsed[0].size_bytes, 2097148_u64 * 1024);
        assert_eq!(parsed[0].used_bytes, Some(1024_u64 * 1024));
    }

    #[test]
    fn extracts_lspci_quoted_fields() {
        let fields = extract_quoted_fields(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"UHD Graphics 620\"",
        );

        assert_eq!(fields[0], "VGA compatible controller");
        assert_eq!(fields[1], "Intel Corporation");
        assert_eq!(fields[2], "UHD Graphics 620");
    }

    #[test]
    fn filters_user_mountpoints() {
        assert_eq!(
            maybe_filter_mountpoint("/home/alice/work", true),
            "/home/[filtered]/work"
        );
        assert_eq!(
            maybe_filter_mountpoint("/media/alice/DISK", true),
            "/media/[filtered]/DISK"
        );
    }

    #[test]
    fn parses_lspci_fixture_gpu_devices() {
        let gpus = parse_lspci_gpus(LSPCI_FIXTURE);

        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].vendor.as_deref(), Some("Intel Corporation"));
        assert_eq!(gpus[0].class, "VGA compatible controller");
    }

    #[test]
    fn parses_xrandr_fixture_displays() {
        let displays = parse_xrandr_displays(XRANDR_FIXTURE);

        assert_eq!(displays.len(), 3);
        assert_eq!(displays[0].name, "eDP-1");
        assert!(displays[0].primary);
        assert_eq!(displays[0].resolution.as_deref(), Some("1920x1080"));
    }

    #[test]
    fn canonicalizes_and_merges_display_names() {
        let merged = merge_displays(
            vec![DisplayOutput {
                name: canonical_display_name("HDMI-A-1"),
                status: "disconnected".to_string(),
                primary: false,
                resolution: None,
            }],
            vec![DisplayOutput {
                name: canonical_display_name("HDMI-1"),
                status: "connected".to_string(),
                primary: false,
                resolution: Some("2560x1440".to_string()),
            }],
        );

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "HDMI-1");
        assert_eq!(merged[0].status, "connected");
        assert_eq!(merged[0].resolution.as_deref(), Some("2560x1440"));
    }

    #[test]
    fn applies_ip_brief_fixture_addresses() {
        let mut interfaces = BTreeMap::from([(
            "enp1s0".to_string(),
            NetworkInterface {
                name: "enp1s0".to_string(),
                link_kind: Some("ethernet".to_string()),
                state: Some("DOWN".to_string()),
                mac_address: None,
                mtu: None,
                ipv4: Vec::new(),
                ipv6: Vec::new(),
            },
        )]);

        apply_ip_brief_addresses(&mut interfaces, IP_BRIEF_FIXTURE, false);

        assert_eq!(
            interfaces["wlp0s20f3"].ipv4,
            vec!["192.168.10.25/24".to_string()]
        );
        assert_eq!(interfaces["enp1s0"].state.as_deref(), Some("UP"));
        assert_eq!(interfaces["ppp0"].ipv4, vec!["10.0.0.9".to_string()]);
    }

    #[test]
    fn parses_lsblk_fixture_json() {
        let parsed: LsblkOutput = serde_json::from_str(LSBLK_FIXTURE).expect("fixture must parse");

        assert_eq!(parsed.blockdevices.len(), 2);
        assert_eq!(parsed.blockdevices[0].device_type, "disk");
        assert_eq!(parsed.blockdevices[0].children.len(), 2);
        assert_eq!(parsed.blockdevices[1].name, "sdb");
        assert_eq!(
            parsed.blockdevices[0].children[1]
                .filesystem_used
                .as_deref(),
            Some("214748364800")
        );
        assert_eq!(
            parsed.blockdevices[0].children[1]
                .filesystem_used_percent
                .as_deref(),
            Some("84%")
        );
    }

    #[test]
    fn parses_proc_partitions_fixture() {
        let parsed = parse_proc_partitions(PROC_PARTITIONS_FIXTURE);

        assert_eq!(
            parsed[0],
            ProcPartitionRecord {
                name: "sda".to_string(),
                blocks_kib: 976762584,
            }
        );
        assert_eq!(parsed[2].name, "nvme0n1");
    }

    #[test]
    fn parses_proc_mounts_fixture() {
        let parsed = parse_proc_mounts(PROC_MOUNTS_FIXTURE);

        assert_eq!(
            parsed["/dev/sda1"],
            ProcMountRecord {
                mountpoint: "/media/alice/BACKUP".to_string(),
                filesystem: "ntfs3".to_string(),
            }
        );
        assert_eq!(parsed["/dev/nvme0n1p2"].mountpoint, "/");
    }

    #[test]
    fn parses_proc_if_inet6_fixture() {
        let parsed = parse_proc_if_inet6(PROC_IF_INET6_FIXTURE, false);

        assert_eq!(parsed["lo"], vec!["::1/128".to_string()]);
        assert_eq!(
            parsed["wlp0s20f3"],
            vec!["fe80::a00:27ff:fe4e:66a1/64".to_string()]
        );
    }

    #[test]
    fn infers_partition_parents_without_misclassifying_nvme_roots() {
        assert_eq!(infer_parent_block("sda1").as_deref(), Some("sda"));
        assert_eq!(infer_parent_block("nvme0n1p2").as_deref(), Some("nvme0n1"));
        assert_eq!(infer_parent_block("mmcblk0p1").as_deref(), Some("mmcblk0"));
        assert_eq!(infer_parent_block("nvme0n1"), None);
    }
}
