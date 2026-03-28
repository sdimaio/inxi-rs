//! Audited external command execution.
//!
//! External commands are treated as an exception, not the default. This module
//! exists to make every allowed command explicit, reproducible, and bounded by
//! timeouts and a minimal environment.

use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::safety::is_trusted_command_path;
use crate::util::trusted_command_path;

#[derive(Debug, Clone, Copy)]
pub enum AuditedCommand {
    IpBriefLink,
    IpBriefAddress,
    LsblkJson,
    LspciMachine,
    XrandrQuery,
}

impl AuditedCommand {
    /// Returns the full audited command set.
    ///
    /// Keeping this list centralized makes the safety report and tests derive
    /// from the same source of truth used at runtime.
    pub const fn all() -> &'static [AuditedCommand] {
        &[
            AuditedCommand::IpBriefLink,
            AuditedCommand::IpBriefAddress,
            AuditedCommand::LsblkJson,
            AuditedCommand::LspciMachine,
            AuditedCommand::XrandrQuery,
        ]
    }

    /// Human-readable command label shown in diagnostics and safety reports.
    pub const fn label(self) -> &'static str {
        match self {
            AuditedCommand::IpBriefLink => "ip -brief link",
            AuditedCommand::IpBriefAddress => "ip -brief address",
            AuditedCommand::LsblkJson => "lsblk --json --bytes",
            AuditedCommand::LspciMachine => "lspci -mm",
            AuditedCommand::XrandrQuery => "xrandr --query",
        }
    }

    /// Program name before path resolution and trust checks.
    pub const fn program(self) -> &'static str {
        match self {
            AuditedCommand::IpBriefLink | AuditedCommand::IpBriefAddress => "ip",
            AuditedCommand::LsblkJson => "lsblk",
            AuditedCommand::LspciMachine => "lspci",
            AuditedCommand::XrandrQuery => "xrandr",
        }
    }

    /// Fixed argument vector for the audited command.
    ///
    /// The arguments are immutable by design so callers cannot accidentally
    /// widen the command surface through ad hoc parameters.
    pub const fn args(self) -> &'static [&'static str] {
        match self {
            AuditedCommand::IpBriefLink => &["-brief", "link"],
            AuditedCommand::IpBriefAddress => &["-brief", "address"],
            AuditedCommand::LsblkJson => &[
                "--json",
                "--bytes",
                "--output",
                "NAME,KNAME,TYPE,SIZE,FSTYPE,MOUNTPOINT,UUID,MODEL,VENDOR,FSAVAIL,FSSIZE,FSUSED,FSUSE%",
            ],
            AuditedCommand::LspciMachine => &["-mm"],
            AuditedCommand::XrandrQuery => &["--query"],
        }
    }

    /// Command-specific timeout chosen to favor responsiveness over exhaustive
    /// probing on production hosts.
    pub const fn timeout(self) -> Duration {
        match self {
            AuditedCommand::XrandrQuery => Duration::from_secs(2),
            _ => Duration::from_secs(3),
        }
    }

    /// Environment variables allowed to survive `env_clear()`.
    ///
    /// Most commands run with no inherited environment. `xrandr` is the only
    /// current exception because display discovery is not meaningful without
    /// `DISPLAY` and sometimes `XAUTHORITY`.
    pub const fn allowed_env(self) -> &'static [&'static str] {
        match self {
            AuditedCommand::XrandrQuery => &["DISPLAY", "XAUTHORITY"],
            _ => &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub enum CommandError {
    NotInstalled(&'static str),
    PolicyViolation(&'static str),
    SpawnFailed(String),
    WaitFailed(String),
    TimedOut(&'static str),
}

/// Runs a command that has been explicitly audited and whitelisted.
///
/// The function rejects shell execution, clears the environment, forces a safe
/// working directory, and enforces a timeout. Those constraints are not
/// incidental; they are the mechanism that keeps the clone observational.
pub fn run_audited_command(command: AuditedCommand) -> Result<CommandOutput, CommandError> {
    let executable = trusted_command_path(command.program())
        .ok_or(CommandError::NotInstalled(command.program()))?;
    if !is_trusted_command_path(&executable) {
        return Err(CommandError::PolicyViolation(
            "command is outside trusted system binary roots",
        ));
    }

    let mut builder = Command::new(executable);
    builder
        .args(command.args())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .current_dir("/");

    for name in command.allowed_env() {
        if let Ok(value) = std::env::var(name) {
            builder.env(name, value);
        }
    }

    let mut child = builder
        .spawn()
        .map_err(|error| CommandError::SpawnFailed(error.to_string()))?;

    let start = Instant::now();
    while start.elapsed() < command.timeout() {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| CommandError::WaitFailed(error.to_string()))?;
                return Ok(CommandOutput {
                    status_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                });
            }
            // `try_wait` avoids blocking the entire process indefinitely while
            // still letting us keep command execution simple and portable.
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => return Err(CommandError::WaitFailed(error.to_string())),
        }
    }

    let _ = child.kill();
    let _ = child.wait();
    Err(CommandError::TimedOut(command.label()))
}

#[cfg(test)]
mod tests {
    use super::AuditedCommand;

    #[test]
    fn audited_commands_are_fixed_and_non_empty() {
        for command in AuditedCommand::all() {
            assert!(!command.label().is_empty());
            assert!(!command.program().is_empty());
            assert!(!command.args().is_empty());
        }
    }
}
