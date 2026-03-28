//! Safety policy primitives.
//!
//! This module is the narrow waist of the project: collectors and utilities
//! are expected to ask policy questions here instead of hardcoding path or
//! command trust rules in many places.

use std::path::{Path, PathBuf};

use crate::command::AuditedCommand;
use crate::model::{ExternalCommandPolicy, SafetyMode, SafetyReport};

const ALLOWED_READ_ROOTS: [&str; 4] = ["/etc", "/proc", "/sys", "/usr/lib"];
const TRUSTED_COMMAND_ROOTS: [&str; 4] = ["/usr/bin", "/usr/sbin", "/bin", "/sbin"];

/// Returns the static safety contract exposed in reports.
///
/// We serialize the policy into every report because "read-only by design"
/// should be inspectable by users and by future frontends, not merely assumed.
pub fn default_safety_report() -> SafetyReport {
    SafetyReport {
        mode: SafetyMode::ReadOnly,
        file_writes_allowed: false,
        shell_execution_allowed: false,
        network_access_allowed: false,
        privilege_escalation_allowed: false,
        external_commands_policy: ExternalCommandPolicy::WhitelistOnly,
        audited_commands: AuditedCommand::all()
            .iter()
            .map(|command| command.label().to_string())
            .collect(),
        allowed_read_roots: ALLOWED_READ_ROOTS.iter().map(ToString::to_string).collect(),
        trusted_command_roots: TRUSTED_COMMAND_ROOTS
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

/// Checks whether a path stays inside the read-only file roots allowed by the
/// project policy.
pub fn is_allowed_read_path(path: &Path) -> bool {
    let resolved = resolve_path_for_policy(path);
    is_absolute_without_traversal(path)
        && ALLOWED_READ_ROOTS
            .iter()
            .any(|root| resolved == Path::new(root) || resolved.starts_with(root))
}

/// Checks whether an executable path belongs to trusted system binary roots.
pub fn is_trusted_command_path(path: &Path) -> bool {
    let resolved = resolve_path_for_policy(path);
    is_absolute_without_traversal(path)
        && TRUSTED_COMMAND_ROOTS
            .iter()
            .any(|root| resolved == Path::new(root) || resolved.starts_with(root))
}

fn resolve_path_for_policy(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn is_absolute_without_traversal(path: &Path) -> bool {
    // We reject relative paths and parent traversal even before canonicalization
    // so callers cannot rely on ambiguous input that might resolve differently
    // across environments.
    path.is_absolute()
        && path
            .components()
            .all(|component| !matches!(component, std::path::Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_allowed_read_path, is_trusted_command_path};

    #[test]
    fn allows_expected_read_roots() {
        assert!(is_allowed_read_path(Path::new("/proc/cpuinfo")));
        assert!(is_allowed_read_path(Path::new(
            "/sys/class/dmi/id/product_name"
        )));
        assert!(is_allowed_read_path(Path::new("/etc/os-release")));
    }

    #[test]
    fn blocks_relative_and_device_paths() {
        assert!(!is_allowed_read_path(Path::new("proc/cpuinfo")));
        assert!(!is_allowed_read_path(Path::new("/dev/null")));
        assert!(!is_allowed_read_path(Path::new("/tmp/test.txt")));
    }

    #[test]
    fn trusts_only_system_command_roots() {
        assert!(is_trusted_command_path(Path::new("/usr/bin/ip")));
        assert!(is_trusted_command_path(Path::new("/bin/lsblk")));
        assert!(!is_trusted_command_path(Path::new("/home/user/bin/ip")));
    }
}
