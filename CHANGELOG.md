# Changelog

All notable changes to this project will be documented in this file.

The project follows a pragmatic pre-1.0 approach: stability claims are made
explicitly in documentation rather than assumed from version numbers alone.

## [0.1.0-alpha.1] - 2026-03-28

Initial public alpha for the standalone GitHub repository.

### Added

- reusable `inxi-core` collection library
- `inxi-cli` frontend with section flags and `screen`/`json` output
- Linux collectors for `System`, `Machine`, `CPU`, `Memory`, `Graphics`, `Network`, `Drives`, `Partitions`, `Swap`, and `Info`
- privacy filtering with `-z`
- `--self-check` diagnostic mode
- safety policy with trusted read roots and audited external commands
- parser fixture tests
- golden tests for normal output and self-check output
- project documentation in English and Italian

### Changed

- project licensing aligned to `GPL-3.0-or-later`
- repository prepared as a standalone public Rust project

### Notes

- status remains `alpha`
- Linux is the only supported platform
- future TUI support is an architectural goal, not a delivered feature
