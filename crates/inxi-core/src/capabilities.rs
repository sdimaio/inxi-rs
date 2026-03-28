//! Runtime capability probing.
//!
//! Capability scanning is kept intentionally cheap and read-only. The goal is
//! not to discover everything the machine could do, but to collect only the
//! signals needed to explain why later collectors did or did not use certain
//! sources.

use std::collections::BTreeMap;

use crate::model::CapabilityReport;
use crate::util::{env_var, find_command_in_path, read_trimmed};

/// Scans the small set of runtime capabilities that influence collector
/// choices and diagnostics.
pub fn scan_capabilities() -> CapabilityReport {
    let commands = ["ip", "lsblk", "lspci", "xrandr"]
        .into_iter()
        .map(|command| (command.to_string(), find_command_in_path(command)))
        .collect::<BTreeMap<_, _>>();

    let paths = [
        "/etc/os-release",
        "/proc/cpuinfo",
        "/proc/meminfo",
        "/proc/net/if_inet6",
        "/proc/partitions",
        "/proc/self/mounts",
        "/proc/swaps",
        "/proc/uptime",
        "/sys/class/drm",
        "/sys/class/dmi/id",
        "/sys/class/net",
        "/sys/block",
    ]
    .into_iter()
    .map(|path| (path.to_string(), std::path::Path::new(path).exists()))
    .collect::<BTreeMap<_, _>>();

    let display_protocol = env_var("WAYLAND_DISPLAY")
        .map(|_| "wayland".to_string())
        .or_else(|| env_var("DISPLAY").map(|_| "x11".to_string()));

    let hostname = read_trimmed("/proc/sys/kernel/hostname")
        .into_option()
        .unwrap_or_else(|| "unknown-host".to_string());

    CapabilityReport {
        platform: std::env::consts::OS.to_string(),
        hostname,
        is_root: is_root(),
        has_display: display_protocol.is_some(),
        display_protocol,
        commands,
        paths,
    }
}

fn is_root() -> bool {
    // We use the effective UID rather than the real UID because that is what
    // matters for permission checks during the current process lifetime.
    unsafe { libc::geteuid() == 0 }
}
