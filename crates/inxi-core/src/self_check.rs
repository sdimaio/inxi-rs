//! Self-check reporting.
//!
//! This module exists to expose collector provenance and fallback decisions in a
//! form that is easier to debug than the user-facing report. It is designed for
//! developers and operators first, not for end-user cosmetics.

use serde::Serialize;

use crate::model::{DataState, Report, SectionEnvelope, Warning};
use crate::request::Request;

#[derive(Debug, Clone, Serialize)]
pub struct SelfCheckReport {
    pub meta: crate::model::Meta,
    pub request: SelfCheckRequest,
    pub capabilities: crate::model::CapabilityReport,
    pub safety: crate::model::SafetyReport,
    pub sections: Vec<SelfCheckSection>,
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfCheckRequest {
    pub requested_sections: Vec<String>,
    pub detail: crate::request::DetailLevel,
    pub output: crate::request::OutputFormat,
    pub filter_sensitive: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfCheckSection {
    pub name: String,
    pub state: DataState,
    pub has_value: bool,
    pub fallback_used: bool,
    pub warning_codes: Vec<String>,
    pub sources: Vec<String>,
}

/// Builds a diagnostic view of a report and the request that produced it.
pub fn build_self_check(report: &Report, request: &Request) -> SelfCheckReport {
    let mut sections = Vec::new();

    push_section(
        &mut sections,
        "system",
        report.sections.system.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "machine",
        report.sections.machine.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "cpu",
        report.sections.cpu.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "memory",
        report.sections.memory.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "graphics",
        report.sections.graphics.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "network",
        report.sections.network.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "drives",
        report.sections.drives.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "partitions",
        report.sections.partitions.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "swap",
        report.sections.swap.as_ref(),
        &report.warnings,
    );
    push_section(
        &mut sections,
        "info",
        report.sections.info.as_ref(),
        &report.warnings,
    );

    SelfCheckReport {
        meta: report.meta.clone(),
        request: SelfCheckRequest {
            requested_sections: request
                .requested_sections()
                .into_iter()
                .map(|section| section.title().to_string())
                .collect(),
            detail: request.detail,
            output: request.output,
            filter_sensitive: request.filter_sensitive,
        },
        capabilities: report.capabilities.clone(),
        safety: report.safety.clone(),
        sections,
        warnings: report.warnings.clone(),
    }
}

/// Serializes the self-check report into JSON.
pub fn render_self_check_json(self_check: &SelfCheckReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(self_check)
}

/// Renders the self-check report in a compact screen format.
pub fn render_self_check_screen(self_check: &SelfCheckReport) -> String {
    let mut output = String::new();

    push_header(&mut output, "SelfCheck");
    push_line(
        &mut output,
        "Requested",
        &self_check.request.requested_sections.join(", "),
    );
    push_line(
        &mut output,
        "Detail",
        &format!("{:?}", self_check.request.detail).to_lowercase(),
    );
    push_line(
        &mut output,
        "Output",
        &format!("{:?}", self_check.request.output).to_lowercase(),
    );
    push_line(
        &mut output,
        "Privacy",
        if self_check.request.filter_sensitive {
            "enabled"
        } else {
            "disabled"
        },
    );

    push_header(&mut output, "Capabilities");
    push_line(
        &mut output,
        "Commands",
        &self_check
            .capabilities
            .commands
            .iter()
            .map(|(name, available)| format!("{name}={}", yes_no(*available)))
            .collect::<Vec<_>>()
            .join(" "),
    );

    push_header(&mut output, "Sections");
    for section in &self_check.sections {
        let sources = if section.sources.is_empty() {
            "none".to_string()
        } else {
            section.sources.join(", ")
        };
        let warnings = if section.warning_codes.is_empty() {
            "none".to_string()
        } else {
            section.warning_codes.join(",")
        };
        push_line(
            &mut output,
            &section.name,
            &format!(
                "state={} value={} fallback={} warnings={} sources={}",
                state_label(&section.state),
                yes_no(section.has_value),
                yes_no(section.fallback_used),
                warnings,
                sources,
            ),
        );
    }

    if !self_check.warnings.is_empty() {
        push_header(&mut output, "Warnings");
        for warning in &self_check.warnings {
            let section = warning.section.as_deref().unwrap_or("global");
            output.push_str(&format!(
                "  - [{}:{}] {}\n",
                section, warning.code, warning.message
            ));
        }
    }

    output
}

fn push_section<T>(
    sections: &mut Vec<SelfCheckSection>,
    name: &str,
    envelope: Option<&SectionEnvelope<T>>,
    warnings: &[Warning],
) {
    let Some(envelope) = envelope else {
        return;
    };

    let warning_codes = warnings
        .iter()
        .filter(|warning| warning.section.as_deref() == Some(name))
        .map(|warning| warning.code.clone())
        .collect::<Vec<_>>();

    sections.push(SelfCheckSection {
        name: name.to_string(),
        state: envelope.state.clone(),
        has_value: envelope.value.is_some(),
        fallback_used: warning_codes.iter().any(|code| code == "fallback_used"),
        warning_codes,
        sources: envelope
            .sources
            .iter()
            .map(|source| source.location.clone())
            .collect(),
    });
}

fn push_header(output: &mut String, title: &str) {
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(title);
    output.push_str(":\n");
}

fn push_line(output: &mut String, label: &str, value: &str) {
    output.push_str(&format!("  {:<10} {}\n", label, value));
}

fn state_label(state: &DataState) -> &'static str {
    match state {
        DataState::Ok => "ok",
        DataState::Missing => "missing",
        DataState::PermissionRequired => "permission_required",
        DataState::Unsupported => "unsupported",
        DataState::Unknown => "unknown",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
