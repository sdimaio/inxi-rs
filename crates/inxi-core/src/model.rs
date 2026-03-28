//! Core data model for collected reports.
//!
//! The model is intentionally explicit and slightly verbose because it doubles
//! as the JSON contract. Keeping section boundaries and source provenance
//! visible in the types makes future frontends and regression tests much safer.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::request::SectionKind;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataState {
    Ok,
    Missing,
    PermissionRequired,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    File,
    Command,
    Environment,
    Procfs,
    Sysfs,
    Derived,
}

/// Provenance record for one data source used by a collector.
///
/// Source tracing is a first-class concept because the project aims to be
/// inspectable: when data is wrong or partial, we want to know where it came
/// from without reverse engineering the code path afterwards.
#[derive(Debug, Clone, Serialize)]
pub struct SourceTrace {
    pub kind: SourceKind,
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SourceTrace {
    /// Marks a regular file source.
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::File,
            location: path.into(),
            detail: None,
        }
    }

    /// Marks a procfs source.
    pub fn procfs(path: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Procfs,
            location: path.into(),
            detail: None,
        }
    }

    /// Marks a sysfs source.
    pub fn sysfs(path: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Sysfs,
            location: path.into(),
            detail: None,
        }
    }

    /// Marks an environment variable source.
    pub fn env(name: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Environment,
            location: name.into(),
            detail: None,
        }
    }

    /// Marks an audited external command source.
    pub fn command(name: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Command,
            location: name.into(),
            detail: None,
        }
    }

    /// Marks a synthetic source derived from other raw inputs.
    pub fn derived(detail: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Derived,
            location: "derived".to_string(),
            detail: Some(detail.into()),
        }
    }
}

/// Warning emitted during planning or collection.
///
/// Warnings are kept separate from section state because a section can be
/// useful and still be partial, degraded, or collected through a fallback.
#[derive(Debug, Clone, Serialize)]
pub struct Warning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl Warning {
    /// Builds a warning with optional section and source attribution.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        section: Option<&str>,
        source: Option<&str>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            section: section.map(ToOwned::to_owned),
            source: source.map(ToOwned::to_owned),
        }
    }
}

/// Metadata about the tool invocation rather than the machine being inspected.
#[derive(Debug, Clone, Serialize)]
pub struct Meta {
    pub tool: String,
    pub version: String,
    pub host: String,
    pub timestamp: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyMode {
    ReadOnly,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalCommandPolicy {
    WhitelistOnly,
}

/// Serialized safety contract exposed to users and future frontends.
#[derive(Debug, Clone, Serialize)]
pub struct SafetyReport {
    pub mode: SafetyMode,
    pub file_writes_allowed: bool,
    pub shell_execution_allowed: bool,
    pub network_access_allowed: bool,
    pub privilege_escalation_allowed: bool,
    pub external_commands_policy: ExternalCommandPolicy,
    pub audited_commands: Vec<String>,
    pub allowed_read_roots: Vec<String>,
    pub trusted_command_roots: Vec<String>,
}

/// Runtime capability snapshot used to explain collector behavior.
#[derive(Debug, Clone, Serialize)]
pub struct CapabilityReport {
    pub platform: String,
    pub hostname: String,
    pub is_root: bool,
    pub has_display: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_protocol: Option<String>,
    pub commands: BTreeMap<String, bool>,
    pub paths: BTreeMap<String, bool>,
}

/// Wraps a section payload with state and provenance.
///
/// Using the same envelope for every section keeps diagnostics uniform and
/// avoids inventing ad hoc "partial" markers inside each domain object.
#[derive(Debug, Clone, Serialize)]
pub struct SectionEnvelope<T> {
    pub state: DataState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
    pub sources: Vec<SourceTrace>,
}

impl<T> SectionEnvelope<T> {
    /// Builds a successful section envelope.
    pub fn ok(value: T, sources: Vec<SourceTrace>) -> Self {
        Self {
            state: DataState::Ok,
            value: Some(value),
            sources,
        }
    }

    /// Builds a section envelope without a payload.
    pub fn without_value(state: DataState, sources: Vec<SourceTrace>) -> Self {
        Self {
            state,
            value: None,
            sources,
        }
    }
}

/// Container for all supported sections.
///
/// Each field is optional because the request decides which sections are
/// collected, and because missing sections should not pollute the JSON shape.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Sections {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SectionEnvelope<SystemSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<SectionEnvelope<MachineSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<SectionEnvelope<CpuSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<SectionEnvelope<MemorySection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphics: Option<SectionEnvelope<GraphicsSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<SectionEnvelope<NetworkSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drives: Option<SectionEnvelope<DrivesSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partitions: Option<SectionEnvelope<PartitionsSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<SectionEnvelope<SwapSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<SectionEnvelope<InfoSection>>,
}

impl Sections {
    /// Inserts a section payload into the matching slot.
    pub fn set(&mut self, section: SectionKind, envelope: SectionData) {
        match (section, envelope) {
            (SectionKind::System, SectionData::System(value)) => self.system = Some(value),
            (SectionKind::Machine, SectionData::Machine(value)) => self.machine = Some(value),
            (SectionKind::Cpu, SectionData::Cpu(value)) => self.cpu = Some(value),
            (SectionKind::Memory, SectionData::Memory(value)) => self.memory = Some(value),
            (SectionKind::Graphics, SectionData::Graphics(value)) => self.graphics = Some(value),
            (SectionKind::Network, SectionData::Network(value)) => self.network = Some(value),
            (SectionKind::Drives, SectionData::Drives(value)) => self.drives = Some(value),
            (SectionKind::Partitions, SectionData::Partitions(value)) => {
                self.partitions = Some(value)
            }
            (SectionKind::Swap, SectionData::Swap(value)) => self.swap = Some(value),
            (SectionKind::Info, SectionData::Info(value)) => self.info = Some(value),
            _ => {}
        }
    }
}

/// Full report returned by the collection pipeline.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub meta: Meta,
    pub sections: Sections,
    pub warnings: Vec<Warning>,
    pub capabilities: CapabilityReport,
    pub safety: SafetyReport,
}

/// Internal transport enum used while building `Sections`.
#[derive(Debug, Clone)]
pub enum SectionData {
    System(SectionEnvelope<SystemSection>),
    Machine(SectionEnvelope<MachineSection>),
    Cpu(SectionEnvelope<CpuSection>),
    Memory(SectionEnvelope<MemorySection>),
    Graphics(SectionEnvelope<GraphicsSection>),
    Network(SectionEnvelope<NetworkSection>),
    Drives(SectionEnvelope<DrivesSection>),
    Partitions(SectionEnvelope<PartitionsSection>),
    Swap(SectionEnvelope<SwapSection>),
    Info(SectionEnvelope<InfoSection>),
}

#[derive(Debug, Clone, Serialize)]
pub struct KernelInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DistroInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretty_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_desktop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_manager: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_server: Option<String>,
}

/// System identity and desktop/session context.
#[derive(Debug, Clone, Serialize)]
pub struct SystemSection {
    pub kernel: KernelInfo,
    pub distro: DistroInfo,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop: Option<DesktopInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FirmwareInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MotherboardInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Machine and firmware identity sourced mostly from DMI.
#[derive(Debug, Clone, Serialize)]
pub struct MachineSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    pub board: MotherboardInfo,
    pub firmware: FirmwareInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuTopology {
    pub logical_cpus: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_packages: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores_per_package: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads_per_core: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuSpeedInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_mhz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_mhz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_mhz: Option<f64>,
}

/// CPU topology and clock information.
#[derive(Debug, Clone, Serialize)]
pub struct CpuSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    pub architecture: String,
    pub topology: CpuTopology,
    pub speed: CpuSpeedInfo,
}

/// Memory totals collected from procfs.
#[derive(Debug, Clone, Serialize)]
pub struct MemorySection {
    pub total_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_total_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_free_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuDevice {
    pub bus: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    pub class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayOutput {
    pub name: String,
    pub status: String,
    pub primary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// GPU devices plus attached display outputs.
#[derive(Debug, Clone, Serialize)]
pub struct GraphicsSection {
    pub gpus: Vec<GpuDevice>,
    pub displays: Vec<DisplayOutput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
}

/// Network interfaces as seen by the currently supported collectors.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkSection {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhysicalDisk {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotational: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removable: Option<bool>,
}

/// Physical block devices.
#[derive(Debug, Clone, Serialize)]
pub struct DrivesSection {
    pub drives: Vec<PhysicalDisk>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionEntry {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mountpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem_size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

/// Partition-level block device view.
#[derive(Debug, Clone, Serialize)]
pub struct PartitionsSection {
    pub partitions: Vec<PartitionEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwapDevice {
    pub path: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// Active swap devices.
#[derive(Debug, Clone, Serialize)]
pub struct SwapSection {
    pub devices: Vec<SwapDevice>,
}

/// Session-adjacent runtime information that does not belong to hardware sections.
#[derive(Debug, Clone, Serialize)]
pub struct InfoSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_count: Option<u64>,
}
