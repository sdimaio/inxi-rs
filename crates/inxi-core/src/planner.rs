//! Execution planning for collector runs.
//!
//! The planner is deliberately simple today, but it already defines the seam
//! where capability-aware or privilege-aware scheduling can evolve later
//! without leaking that logic into the CLI or renderers.

use crate::model::CapabilityReport;
use crate::request::{DetailLevel, Request, SectionKind};

#[derive(Debug, Clone)]
pub struct CollectorTask {
    pub section: SectionKind,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub detail: DetailLevel,
    pub tasks: Vec<CollectorTask>,
}

/// Produces the ordered list of collectors to run for a request.
///
/// The plan currently preserves request order and does not skip based on
/// capabilities. That is a conscious choice: collectors remain responsible for
/// reporting partial data and fallbacks, which keeps planning deterministic and
/// keeps diagnostics visible in self-check output.
pub fn build_plan(request: &Request, _capabilities: &CapabilityReport) -> ExecutionPlan {
    let sections = if request.sections.is_empty() {
        SectionKind::M1.to_vec()
    } else {
        request.requested_sections()
    };

    let tasks = sections
        .into_iter()
        .map(|section| CollectorTask { section })
        .collect();

    ExecutionPlan {
        detail: request.detail,
        tasks,
    }
}
