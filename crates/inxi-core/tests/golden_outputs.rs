use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use inxi_core::model::{
    CapabilityReport, CpuSection, CpuSpeedInfo, CpuTopology, DataState, DesktopInfo, DisplayOutput,
    DistroInfo, DrivesSection, ExternalCommandPolicy, FirmwareInfo, GpuDevice, GraphicsSection,
    InfoSection, KernelInfo, MachineSection, MemorySection, Meta, MotherboardInfo,
    NetworkInterface, NetworkSection, PartitionEntry, PartitionsSection, PhysicalDisk, Report,
    SafetyMode, SafetyReport, SectionEnvelope, Sections, SourceTrace, SwapDevice, SwapSection,
    SystemSection, Warning,
};
use inxi_core::{
    DetailLevel, OutputFormat, Request, SectionKind, build_self_check, render_json, render_screen,
    render_self_check_json, render_self_check_screen,
};

#[test]
fn screen_renderer_matches_golden() {
    let report = sample_report();
    let actual = render_screen(&report);
    let expected = read_golden("report.screen.txt");
    assert_eq!(actual, expected);
}

#[test]
fn json_renderer_matches_golden() {
    let report = sample_report();
    let actual = render_json(&report).expect("json render must succeed");
    let expected = read_golden("report.json")
        .trim_end_matches('\n')
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn self_check_screen_matches_golden() {
    let report = sample_report();
    let self_check = build_self_check(&report, &sample_request(OutputFormat::Screen));
    let actual = render_self_check_screen(&self_check);
    let expected = read_golden("self-check.screen.txt");
    assert_eq!(actual, expected);
}

#[test]
fn self_check_json_matches_golden() {
    let report = sample_report();
    let self_check = build_self_check(&report, &sample_request(OutputFormat::Json));
    let actual = render_self_check_json(&self_check).expect("self-check json render must succeed");
    let expected = read_golden("self-check.json")
        .trim_end_matches('\n')
        .to_string();
    assert_eq!(actual, expected);
}

fn read_golden(name: &str) -> String {
    fs::read_to_string(golden_path(name)).expect("golden fixture must exist")
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name)
}

fn sample_request(output: OutputFormat) -> Request {
    Request::new(
        [
            SectionKind::System,
            SectionKind::Machine,
            SectionKind::Cpu,
            SectionKind::Memory,
            SectionKind::Graphics,
            SectionKind::Network,
            SectionKind::Drives,
            SectionKind::Partitions,
            SectionKind::Swap,
            SectionKind::Info,
        ],
        DetailLevel::Extended,
        output,
        true,
    )
}

fn sample_report() -> Report {
    Report {
        meta: Meta {
            tool: "inxi-rs".to_string(),
            version: "0.1.0-test".to_string(),
            host: "golden-host".to_string(),
            timestamp: "2026-03-27T12:00:00Z".to_string(),
            platform: "linux".to_string(),
        },
        sections: Sections {
            system: Some(SectionEnvelope::ok(
                SystemSection {
                    kernel: KernelInfo {
                        os_type: Some("Linux".to_string()),
                        release: Some("6.8.0-test".to_string()),
                        version: Some("#1 SMP PREEMPT_DYNAMIC".to_string()),
                        architecture: "x86_64".to_string(),
                    },
                    distro: DistroInfo {
                        id: Some("testos".to_string()),
                        name: Some("TestOS".to_string()),
                        version: Some("1.0".to_string()),
                        codename: None,
                        pretty_name: Some("TestOS 1.0".to_string()),
                    },
                    hostname: "golden-host".to_string(),
                    desktop: Some(DesktopInfo {
                        session_type: Some("tty".to_string()),
                        current_desktop: Some("Turbo".to_string()),
                        window_manager: None,
                        display_server: None,
                    }),
                },
                vec![
                    SourceTrace::file("/etc/os-release"),
                    SourceTrace::procfs("/proc/sys/kernel/osrelease"),
                ],
            )),
            machine: Some(SectionEnvelope::ok(
                MachineSection {
                    vendor: Some("Lenovo".to_string()),
                    product_name: Some("ThinkPad X1".to_string()),
                    product_version: Some("Gen 9".to_string()),
                    product_family: None,
                    serial: None,
                    board: MotherboardInfo {
                        vendor: Some("Lenovo".to_string()),
                        name: Some("20XWCTO1WW".to_string()),
                        version: None,
                    },
                    firmware: FirmwareInfo {
                        vendor: Some("LENOVO".to_string()),
                        version: Some("N32ET76W".to_string()),
                        date: None,
                    },
                },
                vec![SourceTrace::sysfs("/sys/class/dmi/id")],
            )),
            cpu: Some(SectionEnvelope::ok(
                CpuSection {
                    vendor: Some("GenuineIntel".to_string()),
                    model_name: Some("Intel(R) Core(TM) i7-1185G7".to_string()),
                    architecture: "x86_64".to_string(),
                    topology: CpuTopology {
                        logical_cpus: 8,
                        physical_packages: Some(1),
                        cores_per_package: Some(4),
                        threads_per_core: Some(2),
                    },
                    speed: CpuSpeedInfo {
                        current_mhz: Some(1800.0),
                        min_mhz: Some(400.0),
                        max_mhz: Some(4800.0),
                    },
                },
                vec![SourceTrace::procfs("/proc/cpuinfo")],
            )),
            memory: Some(SectionEnvelope::ok(
                MemorySection {
                    total_bytes: 17_179_869_184,
                    available_bytes: Some(8_589_934_592),
                    used_bytes: Some(8_589_934_592),
                    swap_total_bytes: None,
                    swap_free_bytes: None,
                },
                vec![SourceTrace::procfs("/proc/meminfo")],
            )),
            graphics: Some(SectionEnvelope::ok(
                GraphicsSection {
                    gpus: vec![GpuDevice {
                        bus: "0000:00:02.0".to_string(),
                        vendor: Some("Intel".to_string()),
                        device: Some("Iris Xe".to_string()),
                        class: "VGA compatible controller".to_string(),
                        driver: Some("i915".to_string()),
                    }],
                    displays: vec![
                        DisplayOutput {
                            name: "eDP-1".to_string(),
                            status: "connected".to_string(),
                            primary: true,
                            resolution: Some("1920x1200".to_string()),
                        },
                        DisplayOutput {
                            name: "HDMI-1".to_string(),
                            status: "disconnected".to_string(),
                            primary: false,
                            resolution: None,
                        },
                    ],
                },
                vec![
                    SourceTrace::sysfs("/sys/class/drm"),
                    SourceTrace::command("lspci -mm"),
                ],
            )),
            network: Some(SectionEnvelope::ok(
                NetworkSection {
                    interfaces: vec![
                        NetworkInterface {
                            name: "eth0".to_string(),
                            link_kind: Some("ethernet".to_string()),
                            state: Some("up".to_string()),
                            mac_address: None,
                            mtu: None,
                            ipv4: vec!["192.168.1.20/24".to_string()],
                            ipv6: vec!["fe80::1/64".to_string()],
                        },
                        NetworkInterface {
                            name: "wlan0".to_string(),
                            link_kind: Some("wifi".to_string()),
                            state: Some("down".to_string()),
                            mac_address: None,
                            mtu: None,
                            ipv4: Vec::new(),
                            ipv6: Vec::new(),
                        },
                    ],
                },
                vec![
                    SourceTrace::sysfs("/sys/class/net"),
                    SourceTrace::procfs("/proc/net/if_inet6"),
                ],
            )),
            drives: Some(SectionEnvelope::ok(
                DrivesSection {
                    drives: vec![
                        PhysicalDisk {
                            name: "nvme0n1".to_string(),
                            path: "/dev/nvme0n1".to_string(),
                            size_bytes: Some(512_110_190_592),
                            model: Some("Samsung PM9A1".to_string()),
                            vendor: None,
                            rotational: Some(false),
                            removable: None,
                        },
                        PhysicalDisk {
                            name: "sda".to_string(),
                            path: "/dev/sda".to_string(),
                            size_bytes: Some(1_000_204_886_016),
                            model: Some("Backup Disk".to_string()),
                            vendor: None,
                            rotational: Some(true),
                            removable: None,
                        },
                    ],
                },
                vec![
                    SourceTrace::sysfs("/sys/block"),
                    SourceTrace::command("lsblk --json --bytes"),
                ],
            )),
            partitions: Some(SectionEnvelope::ok(
                PartitionsSection {
                    partitions: vec![
                        PartitionEntry {
                            name: "nvme0n1p1".to_string(),
                            path: "/dev/nvme0n1p1".to_string(),
                            filesystem: Some("vfat".to_string()),
                            mountpoint: Some("/boot/efi".to_string()),
                            uuid: None,
                            size_bytes: Some(536_870_912),
                            filesystem_size_bytes: Some(536_870_912),
                            available_bytes: Some(402_653_184),
                            used_bytes: Some(134_217_728),
                            used_percent: Some(25),
                            parent: Some("nvme0n1".to_string()),
                        },
                        PartitionEntry {
                            name: "nvme0n1p2".to_string(),
                            path: "/dev/nvme0n1p2".to_string(),
                            filesystem: Some("ext4".to_string()),
                            mountpoint: Some("/".to_string()),
                            uuid: None,
                            size_bytes: Some(255_013_683_200),
                            filesystem_size_bytes: Some(255_013_683_200),
                            available_bytes: Some(40_265_318_400),
                            used_bytes: Some(214_748_364_800),
                            used_percent: Some(84),
                            parent: Some("nvme0n1".to_string()),
                        },
                    ],
                },
                vec![
                    SourceTrace::procfs("/proc/partitions"),
                    SourceTrace::procfs("/proc/self/mounts"),
                ],
            )),
            swap: Some(SectionEnvelope::ok(
                SwapSection {
                    devices: vec![SwapDevice {
                        path: "/swapfile".to_string(),
                        size_bytes: 8_589_934_592,
                        used_bytes: Some(1_073_741_824),
                        priority: Some(-2),
                    }],
                },
                vec![SourceTrace::procfs("/proc/swaps")],
            )),
            info: Some(SectionEnvelope::ok(
                InfoSection {
                    uptime_seconds: Some(93_784),
                    shell: Some("/usr/bin/bash".to_string()),
                    user: Some("sdimaio".to_string()),
                    terminal: Some("xterm-256color".to_string()),
                    locale: Some("it_IT.UTF-8".to_string()),
                    process_count: Some(312),
                },
                vec![
                    SourceTrace::procfs("/proc/uptime"),
                    SourceTrace::env("USER"),
                ],
            )),
        },
        warnings: vec![
            Warning::new(
                "partial_data",
                "Display metadata is partial without xrandr.",
                Some("graphics"),
                Some("xrandr --query"),
            ),
            Warning::new(
                "fallback_used",
                "Used procfs fallback for network addresses.",
                Some("network"),
                Some("/proc/net/if_inet6"),
            ),
        ],
        capabilities: CapabilityReport {
            platform: "linux".to_string(),
            hostname: "golden-host".to_string(),
            is_root: false,
            has_display: true,
            display_protocol: Some("tty".to_string()),
            commands: BTreeMap::from([("ip".to_string(), true), ("lsblk".to_string(), true)]),
            paths: BTreeMap::from([
                ("/proc/cpuinfo".to_string(), true),
                ("/sys/class/net".to_string(), true),
            ]),
        },
        safety: SafetyReport {
            mode: SafetyMode::ReadOnly,
            file_writes_allowed: false,
            shell_execution_allowed: false,
            network_access_allowed: false,
            privilege_escalation_allowed: false,
            external_commands_policy: ExternalCommandPolicy::WhitelistOnly,
            audited_commands: vec!["lsblk --json --bytes".to_string()],
            allowed_read_roots: vec!["/proc".to_string(), "/sys".to_string()],
            trusted_command_roots: vec!["/usr/bin".to_string()],
        },
    }
}

#[test]
fn sample_report_is_fully_populated() {
    let report = sample_report();
    assert!(matches!(
        report
            .sections
            .system
            .as_ref()
            .map(|section| &section.state),
        Some(DataState::Ok)
    ));
    assert!(report.sections.partitions.is_some());
}
