//! Report renderers.
//!
//! The renderer deliberately favors a compact, deterministic format over trying
//! to mimic every formatting choice of the original `inxi`. Stable formatting
//! matters more here because we want golden tests and future frontends to sit on
//! top of a predictable contract.

use crate::model::{
    CpuSection, DrivesSection, GraphicsSection, InfoSection, MachineSection, MemorySection,
    NetworkSection, PartitionsSection, Report, SectionEnvelope, SwapSection, SystemSection,
};

/// Serializes the report into the canonical JSON representation.
pub fn render_json(report: &Report) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

/// Renders the report in the current plain-text screen format.
pub fn render_screen(report: &Report) -> String {
    let mut output = String::new();

    push_header(&mut output, "Meta");
    push_line(
        &mut output,
        "Tool",
        &format!(
            "{} {} on {}",
            report.meta.tool, report.meta.version, report.meta.platform
        ),
    );
    push_line(&mut output, "Host", &report.meta.host);
    push_line(&mut output, "Timestamp", &report.meta.timestamp);

    push_header(&mut output, "Safety");
    push_line(&mut output, "Mode", "read_only");
    push_line(&mut output, "Writes", "disabled");
    push_line(&mut output, "Shell", "disabled");
    push_line(&mut output, "Network", "disabled");
    push_line(&mut output, "Commands", "whitelist_only");

    render_section(
        &mut output,
        "System",
        report.sections.system.as_ref(),
        render_system,
    );
    render_section(
        &mut output,
        "Machine",
        report.sections.machine.as_ref(),
        render_machine,
    );
    render_section(&mut output, "CPU", report.sections.cpu.as_ref(), render_cpu);
    render_section(
        &mut output,
        "Memory",
        report.sections.memory.as_ref(),
        render_memory,
    );
    render_section(
        &mut output,
        "Graphics",
        report.sections.graphics.as_ref(),
        render_graphics,
    );
    render_section(
        &mut output,
        "Network",
        report.sections.network.as_ref(),
        render_network,
    );
    render_section(
        &mut output,
        "Drives",
        report.sections.drives.as_ref(),
        render_drives,
    );
    render_section(
        &mut output,
        "Partitions",
        report.sections.partitions.as_ref(),
        render_partitions,
    );
    render_section(
        &mut output,
        "Swap",
        report.sections.swap.as_ref(),
        render_swap,
    );
    render_section(
        &mut output,
        "Info",
        report.sections.info.as_ref(),
        render_info,
    );

    if !report.warnings.is_empty() {
        push_header(&mut output, "Warnings");
        for warning in &report.warnings {
            let source = warning.source.as_deref().unwrap_or("n/a");
            output.push_str(&format!(
                "  - [{}] {} ({source})\n",
                warning.code, warning.message
            ));
        }
    }

    output
}

fn render_section<T>(
    output: &mut String,
    title: &str,
    section: Option<&SectionEnvelope<T>>,
    renderer: fn(&mut String, &T),
) {
    let Some(section) = section else {
        return;
    };

    // Missing sections are omitted entirely so the screen format reflects the
    // request shape instead of printing empty headings for every known domain.
    push_header(output, title);
    match &section.value {
        Some(value) => renderer(output, value),
        None => push_line(output, "State", state_label(&section.state)),
    }
}

fn render_system(output: &mut String, section: &SystemSection) {
    if let Some(pretty_name) = &section.distro.pretty_name {
        push_line(output, "Distro", pretty_name);
    }
    if let Some(release) = &section.kernel.release {
        push_line(output, "Kernel", release);
    }
    push_line(output, "Arch", &section.kernel.architecture);
    push_line(output, "Hostname", &section.hostname);

    if let Some(desktop) = &section.desktop {
        if let Some(session_type) = &desktop.session_type {
            push_line(output, "Session", session_type);
        }
        if let Some(current_desktop) = &desktop.current_desktop {
            push_line(output, "Desktop", current_desktop);
        }
    }
}

fn render_machine(output: &mut String, section: &MachineSection) {
    if let Some(vendor) = &section.vendor {
        push_line(output, "Vendor", vendor);
    }
    if let Some(product_name) = &section.product_name {
        let product = match &section.product_version {
            Some(version) => format!("{product_name} ({version})"),
            None => product_name.clone(),
        };
        push_line(output, "Product", &product);
    }
    if let Some(product_family) = &section.product_family {
        push_line(output, "Family", product_family);
    }
    if let Some(serial) = &section.serial {
        push_line(output, "Serial", serial);
    }
    if let Some(board_name) = &section.board.name {
        let board = match &section.board.vendor {
            Some(vendor) => format!("{vendor} {board_name}"),
            None => board_name.clone(),
        };
        push_line(output, "Board", &board);
    }
    if let Some(firmware_version) = &section.firmware.version {
        let firmware = match &section.firmware.vendor {
            Some(vendor) => format!("{vendor} {firmware_version}"),
            None => firmware_version.clone(),
        };
        push_line(output, "Firmware", &firmware);
    }
}

fn render_cpu(output: &mut String, section: &CpuSection) {
    if let Some(model_name) = &section.model_name {
        push_line(output, "Model", model_name);
    }
    if let Some(vendor) = &section.vendor {
        push_line(output, "Vendor", vendor);
    }

    push_line(
        output,
        "Topology",
        &format_topology(
            section.topology.logical_cpus,
            section.topology.physical_packages,
            section.topology.cores_per_package,
            section.topology.threads_per_core,
        ),
    );

    let speed = format_speed(
        section.speed.current_mhz,
        section.speed.min_mhz,
        section.speed.max_mhz,
    );
    if let Some(speed) = speed {
        push_line(output, "Speed", &speed);
    }
}

fn render_memory(output: &mut String, section: &MemorySection) {
    push_line(output, "Total", &format_bytes(section.total_bytes));
    if let Some(available_bytes) = section.available_bytes {
        push_line(output, "Available", &format_bytes(available_bytes));
    }
    if let Some(used_bytes) = section.used_bytes {
        push_line(output, "Used", &format_bytes(used_bytes));
    }
}

fn render_graphics(output: &mut String, section: &GraphicsSection) {
    for gpu in &section.gpus {
        let mut summary = String::new();
        if let Some(vendor) = &gpu.vendor {
            summary.push_str(vendor);
            summary.push(' ');
        }
        if let Some(device) = &gpu.device {
            summary.push_str(device);
        } else {
            summary.push_str(&gpu.class);
        }
        if let Some(driver) = &gpu.driver {
            summary.push_str(&format!(" [driver: {driver}]"));
        }
        push_line(output, "GPU", &summary);
    }

    for display in &section.displays {
        let mut summary = format!("{} {}", display.name, display.status);
        if display.primary {
            summary.push_str(" primary");
        }
        if let Some(resolution) = &display.resolution {
            summary.push_str(&format!(" {resolution}"));
        }
        push_line(output, "Display", &summary);
    }
}

fn render_network(output: &mut String, section: &NetworkSection) {
    for interface in &section.interfaces {
        let mut pieces = vec![interface.name.clone()];
        if let Some(link_kind) = &interface.link_kind {
            pieces.push(link_kind.clone());
        }
        if let Some(state) = &interface.state {
            pieces.push(state.clone());
        }
        if !interface.ipv4.is_empty() {
            pieces.push(format!("ipv4={}", interface.ipv4.join(",")));
        }
        if !interface.ipv6.is_empty() {
            pieces.push(format!("ipv6={}", interface.ipv6.join(",")));
        }
        push_line(output, "Iface", &pieces.join(" "));
    }
}

fn render_drives(output: &mut String, section: &DrivesSection) {
    for drive in &section.drives {
        let mut summary = drive.name.clone();
        if let Some(model) = &drive.model {
            summary.push_str(&format!(" {model}"));
        }
        if let Some(size_bytes) = drive.size_bytes {
            summary.push_str(&format!(" ({})", format_bytes(size_bytes)));
        }
        if let Some(rotational) = drive.rotational {
            summary.push_str(if rotational { " HDD" } else { " SSD" });
        }
        push_line(output, "Drive", &summary);
    }
}

fn render_partitions(output: &mut String, section: &PartitionsSection) {
    for partition in &section.partitions {
        let mut summary = partition.name.clone();
        if let Some(filesystem) = &partition.filesystem {
            summary.push_str(&format!(" {filesystem}"));
        }
        if let Some(mountpoint) = &partition.mountpoint {
            summary.push_str(&format!(" {mountpoint}"));
        }
        let total_bytes = partition.filesystem_size_bytes.or(partition.size_bytes);
        match (total_bytes, partition.used_bytes, partition.used_percent) {
            (Some(total_bytes), Some(used_bytes), Some(used_percent)) => {
                summary.push_str(&format!(
                    " total={} used={} ({}%)",
                    format_bytes(total_bytes),
                    format_bytes(used_bytes),
                    used_percent
                ))
            }
            (Some(total_bytes), Some(used_bytes), None) => summary.push_str(&format!(
                " total={} used={}",
                format_bytes(total_bytes),
                format_bytes(used_bytes)
            )),
            (Some(total_bytes), None, _) => {
                summary.push_str(&format!(" ({})", format_bytes(total_bytes)));
            }
            (None, _, _) => {}
        }
        push_line(output, "Part", &summary);
    }
}

fn render_swap(output: &mut String, section: &SwapSection) {
    for device in &section.devices {
        let used = device
            .used_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "unknown".to_string());
        let summary = format!(
            "{} total={}, used={}",
            device.path,
            format_bytes(device.size_bytes),
            used
        );
        push_line(output, "Swap", &summary);
    }
}

fn render_info(output: &mut String, section: &InfoSection) {
    if let Some(uptime_seconds) = section.uptime_seconds {
        push_line(output, "Uptime", &format_uptime(uptime_seconds));
    }
    if let Some(user) = &section.user {
        push_line(output, "User", user);
    }
    if let Some(shell) = &section.shell {
        push_line(output, "Shell", shell);
    }
    if let Some(terminal) = &section.terminal {
        push_line(output, "Terminal", terminal);
    }
    if let Some(locale) = &section.locale {
        push_line(output, "Locale", locale);
    }
    if let Some(process_count) = section.process_count {
        push_line(output, "Processes", &process_count.to_string());
    }
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

fn state_label(state: &crate::model::DataState) -> &'static str {
    match state {
        crate::model::DataState::Ok => "ok",
        crate::model::DataState::Missing => "missing",
        crate::model::DataState::PermissionRequired => "permission_required",
        crate::model::DataState::Unsupported => "unsupported",
        crate::model::DataState::Unknown => "unknown",
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut index = 0usize;

    while value >= 1024.0 && index < UNITS.len() - 1 {
        value /= 1024.0;
        index += 1;
    }

    format!("{value:.1} {}", UNITS[index])
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn format_topology(
    logical_cpus: u32,
    physical_packages: Option<u32>,
    cores_per_package: Option<u32>,
    threads_per_core: Option<u32>,
) -> String {
    // We keep topology phrasing intentionally explicit because it is easier to
    // compare across collectors and architectures than marketing-oriented CPU
    // labels such as MT/MCP.
    let mut pieces = vec![format!("{logical_cpus} logical")];
    if let Some(physical_packages) = physical_packages {
        pieces.push(format!("{physical_packages} package(s)"));
    }
    if let Some(cores_per_package) = cores_per_package {
        pieces.push(format!("{cores_per_package} core(s)/package"));
    }
    if let Some(threads_per_core) = threads_per_core {
        pieces.push(format!("{threads_per_core} thread(s)/core"));
    }
    pieces.join(", ")
}

fn format_speed(
    current_mhz: Option<f64>,
    min_mhz: Option<f64>,
    max_mhz: Option<f64>,
) -> Option<String> {
    let current = current_mhz.map(|value| format!("{value:.0} MHz"));
    let range = match (min_mhz, max_mhz) {
        (Some(min), Some(max)) => Some(format!("{min:.0}-{max:.0} MHz")),
        (Some(min), None) => Some(format!("min {min:.0} MHz")),
        (None, Some(max)) => Some(format!("max {max:.0} MHz")),
        (None, None) => None,
    };

    match (current, range) {
        (Some(current), Some(range)) => Some(format!("{current} ({range})")),
        (Some(current), None) => Some(current),
        (None, Some(range)) => Some(range),
        (None, None) => None,
    }
}
