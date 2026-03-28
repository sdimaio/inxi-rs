# inxi-rs Multi-Host Comparison Procedure

Date: `2026-03-27`

This procedure is the ready-to-run checklist for the next available Linux host.

## Goal

Validate that `inxi-rs` remains semantically coherent outside the current
developer machine and identify host-dependent regressions before the first
stable release.

## Preconditions

- second Linux machine or VM available
- local checkout of this repository
- local `inxi` binary available from the repository root
- Rust workspace buildable on that host

## Commands to run

From the repository root:

1. `./inxi -b -z`
2. `cargo run -q -p inxi-cli --manifest-path inxi-rs/Cargo.toml -- -b -z`
3. `./inxi -G -N -D -P -z`
4. `cargo run -q -p inxi-cli --manifest-path inxi-rs/Cargo.toml -- -G -N -D -P -z`
5. `./inxi -S -M -C -m -I -j -z`
6. `cargo run -q -p inxi-cli --manifest-path inxi-rs/Cargo.toml -- -S -M -C -m -I -j -z`
7. `cargo run -q -p inxi-cli --manifest-path inxi-rs/Cargo.toml -- --self-check`
8. `cargo run -q -p inxi-cli --manifest-path inxi-rs/Cargo.toml -- -m -P -z --output json`

## What to compare

Check these categories:

- `System`: distro, kernel, desktop/session
- `Machine`: vendor/product/board/firmware coverage
- `CPU`: topology and speed plausibility
- `Memory`: total/available/used plausibility
- `Graphics`: GPU identity and display state
- `Network`: interfaces, state, privacy filtering
- `Drives`: per-device identity and size
- `Partitions`: filesystem, mountpoint, total/used metrics
- `Swap`: path, size, used
- `Info`: uptime and process count

## Classification model

For each difference, assign one class:

- `bug`: wrong or misleading in `inxi-rs`
- `gap`: valid but narrower than desired for `v0.1.0`
- `accepted divergence`: intentionally different from `inxi`
- `environment artifact`: host-specific quirk not worth changing

## Output to save

Store the result under `ai/`:

- `ai/inxi-compatibility-review-<host>.md`

Minimum content:

- host summary
- commands executed
- section-by-section findings
- classification per finding
- follow-up actions

## Current blocking note

This procedure was prepared because no second Linux environment was available in
the current workspace context:

- `/run/systemd/machines` was empty
- Docker daemon was not running
