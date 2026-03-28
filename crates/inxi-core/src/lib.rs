//! Public entry points for the `inxi-core` crate.
//!
//! The crate is intentionally split between a headless collection pipeline and
//! thin frontends. That separation is what will let us add a future TUI
//! without rewriting collectors, data models, or safety policy.

pub mod capabilities;
pub mod collectors;
pub mod command;
pub mod model;
pub mod planner;
pub mod render;
pub mod request;
pub mod safety;
pub mod self_check;
pub mod util;

use chrono::Utc;

pub use capabilities::scan_capabilities;
pub use command::{AuditedCommand, run_audited_command};
pub use model::{
    CapabilityReport, DataState, DrivesSection, GraphicsSection, InfoSection, MachineSection,
    MemorySection, Meta, NetworkSection, PartitionsSection, Report, SafetyMode, SafetyReport,
    SectionEnvelope, Sections, SourceKind, SourceTrace, SwapSection, SystemSection, Warning,
};
pub use planner::{CollectorTask, ExecutionPlan, build_plan};
pub use render::{render_json, render_screen};
pub use request::{DetailLevel, OutputFormat, Request, SectionKind};
pub use safety::default_safety_report;
pub use self_check::{
    SelfCheckReport, build_self_check, render_self_check_json, render_self_check_screen,
};

/// Collects a full report for the requested sections.
///
/// The function keeps orchestration in one place on purpose: capabilities,
/// planning, collection, and warning aggregation are all part of a single
/// deterministic pipeline. That makes the behavior easier to reason about,
/// test, and eventually drive from non-CLI frontends.
pub fn collect_report(request: Request) -> Report {
    let capabilities = scan_capabilities();
    let plan = build_plan(&request, &capabilities);
    let mut report = Report {
        meta: Meta {
            tool: "inxi-rs".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            host: capabilities.hostname.clone(),
            timestamp: Utc::now().to_rfc3339(),
            platform: capabilities.platform.clone(),
        },
        sections: Sections::default(),
        warnings: Vec::new(),
        capabilities,
        safety: default_safety_report(),
    };

    if matches!(request.detail, DetailLevel::Admin) && !report.capabilities.is_root {
        report.warnings.push(Warning::new(
            "permission_required",
            "Admin detail requested without root privileges; some future collectors may stay partial.",
            Some("planner"),
            Some("runtime privilege check"),
        ));
    }

    for task in plan.tasks {
        let (envelope, mut warnings) = match task.section {
            SectionKind::System => collectors::collect_system(&report.capabilities),
            SectionKind::Machine => {
                collectors::collect_machine(&report.capabilities, request.filter_sensitive)
            }
            SectionKind::Cpu => collectors::collect_cpu(&report.capabilities),
            SectionKind::Memory => collectors::collect_memory(&report.capabilities),
            SectionKind::Graphics => collectors::collect_graphics(&report.capabilities),
            SectionKind::Network => {
                collectors::collect_network(&report.capabilities, request.filter_sensitive)
            }
            SectionKind::Drives => collectors::collect_drives(&report.capabilities),
            SectionKind::Partitions => {
                collectors::collect_partitions(&report.capabilities, request.filter_sensitive)
            }
            SectionKind::Swap => collectors::collect_swap(&report.capabilities),
            SectionKind::Info => collectors::collect_info(&report.capabilities),
        };

        report.sections.set(task.section, envelope);
        report.warnings.append(&mut warnings);
    }

    report
}
