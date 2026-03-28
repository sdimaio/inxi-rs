//! Network collectors and parsers.
//!
//! The current design prefers interface-level truth over bus-level marketing
//! names. That choice keeps the data useful on headless and virtualized hosts
//! where PCI naming is often less relevant than interface state and addresses.

use std::collections::BTreeMap;
use std::path::Path;

use crate::command::{AuditedCommand, run_audited_command};
use crate::model::{
    CapabilityReport, DataState, NetworkInterface, NetworkSection, SectionData, SectionEnvelope,
    SourceTrace, Warning,
};
use crate::util::{ReadOutcome, path_exists, read_dir, read_text, read_trimmed};

use super::{command_warning, maybe_filter_ip, maybe_filter_mac, push_unique};

pub(crate) fn collect_network(
    capabilities: &CapabilityReport,
    filter_sensitive: bool,
) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let mut sources = vec![SourceTrace::sysfs("/sys/class/net")];
    let mut interfaces = collect_sysfs_interfaces(filter_sensitive);
    let mut used_proc_ipv6_fallback = false;

    if capabilities.commands.get("ip").copied().unwrap_or(false) {
        match run_audited_command(AuditedCommand::IpBriefAddress) {
            Ok(output) => {
                sources.push(SourceTrace::command(AuditedCommand::IpBriefAddress.label()));
                apply_ip_brief_addresses(&mut interfaces, &output.stdout, filter_sensitive);
            }
            Err(error) => {
                warnings.push(command_warning(
                    "network",
                    AuditedCommand::IpBriefAddress,
                    &error,
                ));
                used_proc_ipv6_fallback = apply_proc_if_inet6(&mut interfaces, filter_sensitive)
                    || used_proc_ipv6_fallback;
                if used_proc_ipv6_fallback {
                    warnings.push(Warning::new(
                        "fallback_used",
                        "Network address collection fell back to /proc/net/if_inet6; IPv4 data may be partial.",
                        Some("network"),
                        Some("/proc/net/if_inet6"),
                    ));
                }
            }
        }
    } else {
        warnings.push(Warning::new(
            "missing_tool",
            "The audited ip command is unavailable; IPv4 address data may be partial.",
            Some("network"),
            Some(AuditedCommand::IpBriefAddress.label()),
        ));
        used_proc_ipv6_fallback =
            apply_proc_if_inet6(&mut interfaces, filter_sensitive) || used_proc_ipv6_fallback;
        if used_proc_ipv6_fallback {
            warnings.push(Warning::new(
                "fallback_used",
                "Network address collection fell back to /proc/net/if_inet6; only IPv6 addresses are available without ip.",
                Some("network"),
                Some("/proc/net/if_inet6"),
            ));
        }
    }

    // Even when `ip` succeeds, procfs can still contribute IPv6 addresses that
    // were omitted from the command output on some systems.
    if !used_proc_ipv6_fallback && apply_proc_if_inet6(&mut interfaces, filter_sensitive) {
        used_proc_ipv6_fallback = true;
    }
    if used_proc_ipv6_fallback {
        sources.push(SourceTrace::procfs("/proc/net/if_inet6"));
    }

    let interfaces = interfaces.into_values().collect::<Vec<_>>();
    if interfaces.is_empty() {
        return (
            SectionData::Network(SectionEnvelope::without_value(DataState::Missing, sources)),
            warnings,
        );
    }

    let envelope = SectionEnvelope::ok(NetworkSection { interfaces }, sources);
    (SectionData::Network(envelope), warnings)
}

fn collect_sysfs_interfaces(filter_sensitive: bool) -> BTreeMap<String, NetworkInterface> {
    let mut interfaces = BTreeMap::new();
    let Ok(entries) = read_dir("/sys/class/net") else {
        return interfaces;
    };

    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().to_string();
        let base = entry.path();
        let mac_address = read_trimmed(base.join("address"))
            .into_option()
            .filter(|value| value != "00:00:00:00:00:00")
            .map(|value| maybe_filter_mac(&value, filter_sensitive));

        let interface = NetworkInterface {
            name: name.clone(),
            link_kind: infer_network_kind(&name, &base),
            state: read_trimmed(base.join("operstate")).into_option(),
            mac_address,
            mtu: read_trimmed(base.join("mtu"))
                .into_option()
                .and_then(|value| value.parse().ok()),
            ipv4: Vec::new(),
            ipv6: Vec::new(),
        };

        interfaces.insert(name, interface);
    }

    interfaces
}

pub(crate) fn apply_ip_brief_addresses(
    interfaces: &mut BTreeMap<String, NetworkInterface>,
    stdout: &str,
    filter_sensitive: bool,
) {
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(state) = parts.next() else {
            continue;
        };

        let interface = interfaces
            .entry(name.to_string())
            .or_insert_with(|| NetworkInterface {
                name: name.to_string(),
                link_kind: None,
                state: Some(state.to_string()),
                mac_address: None,
                mtu: None,
                ipv4: Vec::new(),
                ipv6: Vec::new(),
            });
        interface.state = Some(state.to_string());

        let mut skip_next = false;
        for token in parts {
            if skip_next {
                skip_next = false;
                continue;
            }
            if token == "peer" {
                skip_next = true;
                continue;
            }

            if token.contains(':') {
                push_unique(
                    &mut interface.ipv6,
                    maybe_filter_ip(token, filter_sensitive).to_string(),
                );
            } else if token.contains('.') {
                push_unique(
                    &mut interface.ipv4,
                    maybe_filter_ip(token, filter_sensitive).to_string(),
                );
            }
        }
    }
}

fn apply_proc_if_inet6(
    interfaces: &mut BTreeMap<String, NetworkInterface>,
    filter_sensitive: bool,
) -> bool {
    let ReadOutcome::Present(text) = read_text("/proc/net/if_inet6") else {
        return false;
    };

    let parsed = parse_proc_if_inet6(&text, filter_sensitive);
    if parsed.is_empty() {
        return false;
    }

    for (name, ipv6_addresses) in parsed {
        let interface = interfaces
            .entry(name.clone())
            .or_insert_with(|| NetworkInterface {
                name,
                link_kind: None,
                state: None,
                mac_address: None,
                mtu: None,
                ipv4: Vec::new(),
                ipv6: Vec::new(),
            });

        for ipv6 in ipv6_addresses {
            push_unique(&mut interface.ipv6, ipv6);
        }
    }

    true
}

fn infer_network_kind(name: &str, base: &Path) -> Option<String> {
    if name == "lo" {
        Some("loopback".to_string())
    } else if path_exists(base.join("wireless")) || name.starts_with("wl") {
        Some("wireless".to_string())
    } else if name.starts_with("en") || name.starts_with("eth") {
        Some("ethernet".to_string())
    } else if name.starts_with("ppp") {
        Some("point_to_point".to_string())
    } else if ["tap", "tun", "docker", "virbr", "veth"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        Some("virtual".to_string())
    } else {
        None
    }
}

pub(crate) fn parse_proc_if_inet6(
    input: &str,
    filter_sensitive: bool,
) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();

    for line in input.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 6 {
            continue;
        }

        let interface = parts[5].to_string();
        let prefix_len = u8::from_str_radix(parts[2], 16).ok();
        let address = format_ipv6_hex(parts[0]).map(|address| match prefix_len {
            Some(prefix_len) => format!("{address}/{prefix_len}"),
            None => address,
        });

        if let Some(address) = address {
            let rendered = maybe_filter_ip(&address, filter_sensitive).to_string();
            map.entry(interface).or_insert_with(Vec::new).push(rendered);
        }
    }

    map
}

fn format_ipv6_hex(hex: &str) -> Option<String> {
    if hex.len() != 32 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }

    let segments = (0..8)
        .map(|index| u16::from_str_radix(&hex[index * 4..index * 4 + 4], 16).ok())
        .collect::<Option<Vec<_>>>()?;

    Some(
        std::net::Ipv6Addr::new(
            segments[0],
            segments[1],
            segments[2],
            segments[3],
            segments[4],
            segments[5],
            segments[6],
            segments[7],
        )
        .to_string(),
    )
}
