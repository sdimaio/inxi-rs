//! Request and selection types shared by every frontend.
//!
//! These types exist to freeze the semantic contract before we add more
//! frontends. The CLI, tests, and future TUI should all describe requests in
//! the same language instead of each inventing its own flag mapping.

use std::collections::BTreeSet;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    System,
    Machine,
    Cpu,
    Memory,
    Graphics,
    Network,
    Drives,
    Partitions,
    Swap,
    Info,
}

impl SectionKind {
    /// Minimal milestone used during the first bootstrap of the clone.
    ///
    /// We keep this list even after adding more sections because it remains a
    /// useful lower bound for diagnostics and reduced environments.
    pub const M1: [SectionKind; 5] = [
        SectionKind::System,
        SectionKind::Machine,
        SectionKind::Cpu,
        SectionKind::Memory,
        SectionKind::Info,
    ];

    /// Default basic report for `inxi-rs`.
    ///
    /// The list is explicit instead of derived from every known section so the
    /// default output can stay stable while the project grows.
    pub const BASIC: [SectionKind; 9] = [
        SectionKind::System,
        SectionKind::Machine,
        SectionKind::Cpu,
        SectionKind::Memory,
        SectionKind::Graphics,
        SectionKind::Network,
        SectionKind::Drives,
        SectionKind::Swap,
        SectionKind::Info,
    ];

    /// Returns the human-facing section title used in screen output and
    /// self-check reports.
    pub fn title(self) -> &'static str {
        match self {
            SectionKind::System => "System",
            SectionKind::Machine => "Machine",
            SectionKind::Cpu => "CPU",
            SectionKind::Memory => "Memory",
            SectionKind::Graphics => "Graphics",
            SectionKind::Network => "Network",
            SectionKind::Drives => "Drives",
            SectionKind::Partitions => "Partitions",
            SectionKind::Swap => "Swap",
            SectionKind::Info => "Info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DetailLevel {
    Basic,
    Normal,
    Extended,
    Full,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Screen,
    Json,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub sections: BTreeSet<SectionKind>,
    pub detail: DetailLevel,
    pub output: OutputFormat,
    pub filter_sensitive: bool,
}

impl Request {
    /// Builds a normalized request object shared by planners and renderers.
    pub fn new(
        sections: impl IntoIterator<Item = SectionKind>,
        detail: DetailLevel,
        output: OutputFormat,
        filter_sensitive: bool,
    ) -> Self {
        Self {
            sections: sections.into_iter().collect(),
            detail,
            output,
            filter_sensitive,
        }
    }

    /// Creates the default basic request used when the CLI receives no
    /// explicit section flags.
    pub fn basic(output: OutputFormat, filter_sensitive: bool) -> Self {
        Self::new(
            SectionKind::BASIC,
            DetailLevel::Basic,
            output,
            filter_sensitive,
        )
    }

    /// Returns sections in deterministic order.
    ///
    /// A stable order matters for reproducible screen output, predictable JSON,
    /// and low-noise golden tests.
    pub fn requested_sections(&self) -> Vec<SectionKind> {
        self.sections.iter().copied().collect()
    }
}
