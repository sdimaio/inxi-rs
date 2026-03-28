//! CLI frontend for `inxi-rs`.
//!
//! The CLI intentionally stays thin: it translates flags into a `Request` and
//! delegates the rest to `inxi-core`. Keeping it small now will make future
//! frontends easier to add without duplicating semantics.

use std::collections::BTreeSet;
use std::process::ExitCode;

use clap::{ArgAction, Parser, ValueEnum};
use inxi_core::{
    DetailLevel, OutputFormat, Request, SectionKind, build_self_check, collect_report, render_json,
    render_screen, render_self_check_json, render_self_check_screen,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputArg {
    Screen,
    Json,
}

impl From<OutputArg> for OutputFormat {
    fn from(value: OutputArg) -> Self {
        match value {
            OutputArg::Screen => OutputFormat::Screen,
            OutputArg::Json => OutputFormat::Json,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "inxi-rs",
    version,
    about = "Safety-first Rust system information tool for Linux, inspired by inxi"
)]
struct Cli {
    /// Requests the project-defined basic report instead of the original
    /// inxi preset, because our v1 support surface is intentionally narrower.
    #[arg(short = 'b', long = "basic")]
    basic: bool,
    /// Includes the system section.
    #[arg(short = 'S', long = "system")]
    system: bool,
    /// Includes the machine section.
    #[arg(short = 'M', long = "machine")]
    machine: bool,
    /// Includes the CPU section.
    #[arg(short = 'C', long = "cpu")]
    cpu: bool,
    /// Includes the memory section.
    #[arg(short = 'm', long = "memory")]
    memory: bool,
    /// Includes the graphics section.
    #[arg(short = 'G', long = "graphics")]
    graphics: bool,
    /// Includes the network section.
    #[arg(short = 'N', long = "network")]
    network: bool,
    /// Includes the drives section.
    #[arg(short = 'D', long = "disk")]
    drives: bool,
    /// Includes the partitions section.
    #[arg(short = 'P', long = "partition")]
    partitions: bool,
    /// Includes the swap section.
    #[arg(short = 'j', long = "swap")]
    swap: bool,
    /// Includes the info section.
    #[arg(short = 'I', long = "info")]
    info: bool,
    /// Increases detail level without changing the requested section set.
    #[arg(short = 'x', action = ArgAction::Count)]
    detail: u8,
    /// Requests admin-level detail. The core still stays read-only and emits a
    /// warning when the process lacks the privileges needed for future admin collectors.
    #[arg(short = 'a', long = "admin")]
    admin: bool,
    /// Filters user-sensitive values such as IPs, MACs, UUIDs, and user mount paths.
    #[arg(short = 'z', long = "filter", conflicts_with = "no_filter")]
    filter: bool,
    /// Disables filtering explicitly.
    #[arg(short = 'Z', long = "no-filter")]
    no_filter: bool,
    /// Shows collector provenance, warnings, and fallback decisions instead of
    /// the normal user-facing report.
    #[arg(long = "self-check")]
    self_check: bool,
    /// Selects the stable output contract.
    #[arg(long = "output", value_enum, default_value_t = OutputArg::Screen)]
    output: OutputArg,
}

impl Cli {
    /// Converts CLI flags into the stable frontend-neutral request model.
    fn to_request(&self) -> Request {
        let mut sections = BTreeSet::new();
        if self.system {
            sections.insert(SectionKind::System);
        }
        if self.machine {
            sections.insert(SectionKind::Machine);
        }
        if self.cpu {
            sections.insert(SectionKind::Cpu);
        }
        if self.memory {
            sections.insert(SectionKind::Memory);
        }
        if self.graphics {
            sections.insert(SectionKind::Graphics);
        }
        if self.network {
            sections.insert(SectionKind::Network);
        }
        if self.drives {
            sections.insert(SectionKind::Drives);
        }
        if self.partitions {
            sections.insert(SectionKind::Partitions);
        }
        if self.swap {
            sections.insert(SectionKind::Swap);
        }
        if self.info {
            sections.insert(SectionKind::Info);
        }

        if self.basic || sections.is_empty() {
            sections.extend(SectionKind::BASIC);
        }

        let detail = if self.admin {
            DetailLevel::Admin
        } else {
            match self.detail {
                0 if self.basic || sections.len() == SectionKind::BASIC.len() => DetailLevel::Basic,
                0 => DetailLevel::Normal,
                1 => DetailLevel::Extended,
                _ => DetailLevel::Full,
            }
        };

        Request::new(
            sections,
            detail,
            self.output.into(),
            self.filter && !self.no_filter,
        )
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let request = cli.to_request();
    let report = collect_report(request.clone());

    let rendered = if cli.self_check {
        let self_check = build_self_check(&report, &request);
        match request.output {
            OutputFormat::Screen => Ok(render_self_check_screen(&self_check)),
            OutputFormat::Json => {
                render_self_check_json(&self_check).map_err(|error| error.to_string())
            }
        }
    } else {
        match request.output {
            OutputFormat::Screen => Ok(render_screen(&report)),
            OutputFormat::Json => render_json(&report).map_err(|error| error.to_string()),
        }
    };

    match rendered {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("render error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use inxi_core::{DetailLevel, OutputFormat, SectionKind};

    use super::Cli;

    #[test]
    fn empty_cli_defaults_to_basic_screen_request() {
        let cli = Cli::parse_from(["inxi-rs"]);
        let request = cli.to_request();

        assert_eq!(request.detail, DetailLevel::Basic);
        assert_eq!(request.output, OutputFormat::Screen);
        assert!(!request.filter_sensitive);
        assert_eq!(request.requested_sections(), SectionKind::BASIC.to_vec());
    }

    #[test]
    fn explicit_sections_preserve_selection_and_filtering() {
        let cli = Cli::parse_from(["inxi-rs", "-N", "-D", "-P", "-z", "--output", "json"]);
        let request = cli.to_request();

        assert_eq!(request.detail, DetailLevel::Normal);
        assert_eq!(request.output, OutputFormat::Json);
        assert!(request.filter_sensitive);
        assert_eq!(
            request.requested_sections(),
            vec![
                SectionKind::Network,
                SectionKind::Drives,
                SectionKind::Partitions,
            ]
        );
    }

    #[test]
    fn admin_flag_overrides_detail_counter() {
        let cli = Cli::parse_from(["inxi-rs", "-S", "-x", "-a"]);
        let request = cli.to_request();

        assert_eq!(request.detail, DetailLevel::Admin);
        assert_eq!(request.requested_sections(), vec![SectionKind::System]);
    }
}
