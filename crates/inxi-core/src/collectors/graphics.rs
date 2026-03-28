//! Graphics collectors and parsers.
//!
//! Graphics is intentionally assembled from multiple imperfect sources: sysfs
//! for connector visibility, `lspci` for GPU identity, and `xrandr` for active
//! display state. No single source is complete enough on its own.

use std::collections::BTreeMap;

use crate::command::{AuditedCommand, run_audited_command};
use crate::model::{
    CapabilityReport, DataState, DisplayOutput, GpuDevice, GraphicsSection, SectionData,
    SectionEnvelope, SourceTrace, Warning,
};
use crate::util::{read_dir, read_link_name, read_text, read_trimmed};

use super::{clean_option, command_warning};

pub(crate) fn collect_graphics(capabilities: &CapabilityReport) -> (SectionData, Vec<Warning>) {
    let mut warnings = Vec::new();
    let mut sources = vec![SourceTrace::sysfs("/sys/class/drm")];

    let mut gpus = Vec::new();
    if capabilities.commands.get("lspci").copied().unwrap_or(false) {
        match run_audited_command(AuditedCommand::LspciMachine) {
            Ok(output) => {
                sources.push(SourceTrace::command(AuditedCommand::LspciMachine.label()));
                gpus = parse_lspci_gpus(&output.stdout);
            }
            Err(error) => warnings.push(command_warning(
                "graphics",
                AuditedCommand::LspciMachine,
                &error,
            )),
        }
    } else {
        warnings.push(Warning::new(
            "missing_tool",
            "The audited lspci command is unavailable; GPU detection may be partial.",
            Some("graphics"),
            Some(AuditedCommand::LspciMachine.label()),
        ));
    }

    let mut displays = collect_drm_displays();
    if capabilities.has_display
        && capabilities
            .commands
            .get("xrandr")
            .copied()
            .unwrap_or(false)
    {
        match run_audited_command(AuditedCommand::XrandrQuery) {
            Ok(output) => {
                sources.push(SourceTrace::command(AuditedCommand::XrandrQuery.label()));
                let xrandr_displays = parse_xrandr_displays(&output.stdout);
                // Sysfs sees connectors even when they are inactive, while
                // xrandr knows which output is primary and what resolution is
                // currently active. Merging both gives the most stable picture.
                displays = merge_displays(displays, xrandr_displays);
            }
            Err(error) => warnings.push(command_warning(
                "graphics",
                AuditedCommand::XrandrQuery,
                &error,
            )),
        }
    } else if capabilities.has_display {
        warnings.push(Warning::new(
            "missing_tool",
            "The audited xrandr command is unavailable; display metadata may be partial.",
            Some("graphics"),
            Some(AuditedCommand::XrandrQuery.label()),
        ));
    }

    if gpus.is_empty() && displays.is_empty() {
        return (
            SectionData::Graphics(SectionEnvelope::without_value(DataState::Missing, sources)),
            warnings,
        );
    }

    let envelope = SectionEnvelope::ok(GraphicsSection { gpus, displays }, sources);
    (SectionData::Graphics(envelope), warnings)
}

fn collect_drm_displays() -> Vec<DisplayOutput> {
    let mut displays = Vec::new();
    let Ok(entries) = read_dir("/sys/class/drm") else {
        return displays;
    };

    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || !name.contains('-') {
            continue;
        }

        let connector = name
            .split_once('-')
            .map(|(_, connector)| connector.to_string())
            .unwrap_or(name.clone());
        let entry_path = entry.path();
        let status = read_trimmed(entry_path.join("status"))
            .into_option()
            .unwrap_or_else(|| "unknown".to_string());
        let resolution = read_text(entry_path.join("modes"))
            .into_option()
            .and_then(|text| text.lines().next().map(ToOwned::to_owned));

        displays.push(DisplayOutput {
            name: canonical_display_name(&connector),
            status,
            primary: false,
            resolution,
        });
    }

    displays.sort_by(|left, right| left.name.cmp(&right.name));
    displays
}

pub(crate) fn merge_displays(
    base: Vec<DisplayOutput>,
    overlay: Vec<DisplayOutput>,
) -> Vec<DisplayOutput> {
    let mut merged = base
        .into_iter()
        .map(|display| (display.name.clone(), display))
        .collect::<BTreeMap<_, _>>();

    for display in overlay {
        merged
            .entry(display.name.clone())
            .and_modify(|current| {
                current.status = display.status.clone();
                current.primary = display.primary;
                if display.resolution.is_some() {
                    current.resolution = display.resolution.clone();
                }
            })
            .or_insert(display);
    }

    merged.into_values().collect()
}

pub(crate) fn parse_lspci_gpus(stdout: &str) -> Vec<GpuDevice> {
    let mut gpus = stdout
        .lines()
        .filter_map(|line| {
            let bus = line.split_whitespace().next()?.to_string();
            let quoted = extract_quoted_fields(line);
            let class = quoted.first()?.to_string();
            if !matches!(
                class.as_str(),
                "VGA compatible controller" | "3D controller" | "Display controller"
            ) {
                return None;
            }

            let sysfs_bus = if bus.starts_with("0000:") {
                bus.clone()
            } else {
                format!("0000:{bus}")
            };

            Some(GpuDevice {
                bus,
                vendor: clean_option(quoted.get(1).cloned()),
                device: clean_option(quoted.get(2).cloned()),
                class,
                driver: read_link_name(format!("/sys/bus/pci/devices/{sysfs_bus}/driver"))
                    .into_option(),
            })
        })
        .collect::<Vec<_>>();

    gpus.sort_by(|left, right| left.bus.cmp(&right.bus));
    gpus
}

pub(crate) fn parse_xrandr_displays(stdout: &str) -> Vec<DisplayOutput> {
    let mut displays = Vec::new();

    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if !(line.contains(" connected") || line.contains(" disconnected")) {
            continue;
        }

        let tokens = line.split_whitespace().collect::<Vec<_>>();
        if tokens.len() < 2 {
            continue;
        }

        let resolution = tokens
            .iter()
            .find(|token| {
                token
                    .chars()
                    .next()
                    .map(|character| character.is_ascii_digit())
                    .unwrap_or(false)
                    && token.contains('x')
            })
            .map(|token| normalize_resolution_token(token).to_string());

        displays.push(DisplayOutput {
            name: canonical_display_name(tokens[0]),
            status: tokens[1].to_string(),
            primary: tokens.contains(&"primary"),
            resolution,
        });
    }

    displays
}

fn normalize_resolution_token(token: &str) -> &str {
    token.split('+').next().unwrap_or(token)
}

pub(crate) fn canonical_display_name(name: &str) -> String {
    // Different sources disagree on HDMI naming. Canonicalization keeps golden
    // tests stable and avoids duplicate displays in merged output.
    if let Some(suffix) = name.strip_prefix("HDMI-A-") {
        format!("HDMI-{suffix}")
    } else {
        name.to_string()
    }
}

pub(crate) fn extract_quoted_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut inside_quotes = false;

    for character in line.chars() {
        match character {
            '"' if inside_quotes => {
                fields.push(current.clone());
                current.clear();
                inside_quotes = false;
            }
            '"' => inside_quotes = true,
            _ if inside_quotes => current.push(character),
            _ => {}
        }
    }

    fields
}
