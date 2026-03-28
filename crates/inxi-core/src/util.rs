//! Shared utility helpers.
//!
//! The helpers in this module intentionally return structured outcomes instead
//! of panicking or silently swallowing errors. For a diagnostics-oriented tool,
//! "why data is missing" matters almost as much as the data itself.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::safety::{is_allowed_read_path, is_trusted_command_path};

#[derive(Debug, Clone)]
pub enum ReadOutcome {
    Present(String),
    Missing,
    PermissionDenied,
    Unreadable,
    BlockedByPolicy,
}

impl ReadOutcome {
    /// Converts a structured read result into an optional string when callers
    /// explicitly choose to ignore the distinction between missing and blocked.
    pub fn into_option(self) -> Option<String> {
        match self {
            ReadOutcome::Present(value) => Some(value),
            ReadOutcome::Missing
            | ReadOutcome::PermissionDenied
            | ReadOutcome::Unreadable
            | ReadOutcome::BlockedByPolicy => None,
        }
    }
}

/// Reads a UTF-8 text file and trims surrounding whitespace.
///
/// The trim is deliberate for procfs/sysfs scalar values where trailing newlines
/// are transport noise rather than meaningful payload.
pub fn read_trimmed(path: impl AsRef<Path>) -> ReadOutcome {
    let path = path.as_ref();
    if !is_allowed_read_path(path) {
        return ReadOutcome::BlockedByPolicy;
    }

    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => return ReadOutcome::Unreadable,
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return ReadOutcome::Missing,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return ReadOutcome::PermissionDenied;
        }
        Err(_) => return ReadOutcome::Unreadable,
    }

    match fs::read_to_string(path) {
        Ok(contents) => {
            let trimmed = contents.trim().to_string();
            if trimmed.is_empty() {
                ReadOutcome::Missing
            } else {
                ReadOutcome::Present(trimmed)
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::Missing,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            ReadOutcome::PermissionDenied
        }
        Err(_) => ReadOutcome::Unreadable,
    }
}

/// Reads a UTF-8 text file without trimming.
///
/// This variant exists because multiline pseudo-files such as `/proc/cpuinfo`
/// and `/proc/mounts` would lose structure if normalized too aggressively.
pub fn read_text(path: impl AsRef<Path>) -> ReadOutcome {
    let path = path.as_ref();
    if !is_allowed_read_path(path) {
        return ReadOutcome::BlockedByPolicy;
    }

    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => ReadOutcome::Unreadable,
        Ok(_) => match fs::read_to_string(path) {
            Ok(contents) => ReadOutcome::Present(contents),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::Missing,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                ReadOutcome::PermissionDenied
            }
            Err(_) => ReadOutcome::Unreadable,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::Missing,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            ReadOutcome::PermissionDenied
        }
        Err(_) => ReadOutcome::Unreadable,
    }
}

/// Opens a directory only if it is allowed by the safety policy.
pub fn read_dir(path: impl AsRef<Path>) -> Result<fs::ReadDir, ReadOutcome> {
    let path = path.as_ref();
    if !is_allowed_read_path(path) {
        return Err(ReadOutcome::BlockedByPolicy);
    }

    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_dir() => Err(ReadOutcome::Unreadable),
        Ok(_) => fs::read_dir(path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => ReadOutcome::Missing,
            std::io::ErrorKind::PermissionDenied => ReadOutcome::PermissionDenied,
            _ => ReadOutcome::Unreadable,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(ReadOutcome::Missing),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(ReadOutcome::PermissionDenied)
        }
        Err(_) => Err(ReadOutcome::Unreadable),
    }
}

/// Reads the final path component of a symlink target.
///
/// Collectors usually only care about the symbolic name of a driver, not the
/// full path, so returning the basename keeps the model compact and stable.
pub fn read_link_name(path: impl AsRef<Path>) -> ReadOutcome {
    let path = path.as_ref();
    if !is_allowed_read_path(path) {
        return ReadOutcome::BlockedByPolicy;
    }

    match fs::read_link(path) {
        Ok(target) => target
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| ReadOutcome::Present(name.to_string()))
            .unwrap_or(ReadOutcome::Unreadable),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ReadOutcome::Missing,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            ReadOutcome::PermissionDenied
        }
        Err(_) => ReadOutcome::Unreadable,
    }
}

/// Checks for path existence through the same read policy used by file access.
pub fn path_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    is_allowed_read_path(path) && path.exists()
}

/// Parses `KEY=VALUE` content commonly found in `/etc/os-release` and similar
/// files.
pub fn parse_key_value_lines(input: &str) -> BTreeMap<String, String> {
    input
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), unquote(value.trim())))
        })
        .collect()
}

/// Removes a single layer of simple shell-style quoting.
pub fn unquote(value: &str) -> String {
    value
        .trim_matches('"')
        .trim_matches('\'')
        .replace("\\\"", "\"")
}

/// Returns whether the command resolves to a trusted executable.
pub fn find_command_in_path(command: &str) -> bool {
    trusted_command_path(command).is_some()
}

/// Resolves a command name to a trusted executable path.
///
/// The helper intentionally searches `PATH` instead of hardcoding one location,
/// but still re-applies trust checks so an unexpected user directory cannot
/// shadow a system binary.
pub fn trusted_command_path(command: &str) -> Option<PathBuf> {
    let path_os = std::env::var_os("PATH")?;

    std::env::split_paths(&path_os).find_map(|entry| {
        let candidate = entry.join(command);
        let resolved = fs::canonicalize(&candidate).ok()?;
        if resolved.is_file()
            && is_executable(resolved.clone())
            && is_trusted_command_path(&resolved)
        {
            Some(resolved)
        } else {
            None
        }
    })
}

fn is_executable(path: PathBuf) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                metadata.is_file()
            }
        }
        Err(_) => false,
    }
}

/// Reads a non-empty environment variable.
pub fn env_var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Reads a non-empty OS environment variable.
pub fn env_os_var(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

/// Returns the final path component, preserving the original string on failure.
pub fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or(path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{ReadOutcome, read_text, read_trimmed};

    #[test]
    fn blocks_reads_outside_allowed_roots() {
        assert!(matches!(
            read_text("/dev/null"),
            ReadOutcome::BlockedByPolicy
        ));
        assert!(matches!(
            read_trimmed("/tmp/example"),
            ReadOutcome::BlockedByPolicy
        ));
    }
}
