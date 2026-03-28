# inxi-rs

`inxi-rs` is a safety-first, read-only Rust system information tool for Linux,
inspired by [`inxi`](https://github.com/smxi/inxi).

It is an independent Rust reimplementation focused on:

- a reusable headless core
- deterministic `screen` and `json` output
- explicit source tracing and warnings
- a future TUI-friendly architecture

This project is maintained by Simmaco Di Maio and reflects a deliberate
long-term investment in Rust as a systems language.

`inxi-rs` is not an official port of `inxi` and is not affiliated with
upstream.

Italian documentation is available in [README.it.md](README.it.md).

## Status

Current status: `0.1.0-alpha.1`

What this means:

- the architecture is already serious enough for public review
- the project is useful today on Linux
- the output and supported sections are still being stabilized
- the repository is not yet claiming feature parity with `inxi`

## Why This Exists

The goal is not to produce a line-by-line translation of the original Perl code.

The goal is to build a modern Rust system information tool that keeps what is
valuable in `inxi`:

- pragmatic section-oriented output
- robust fallback thinking
- privacy filters
- operational usefulness on real Linux systems

while replacing what would be costly to preserve:

- layout-driven internal data structures
- globally coupled state
- CLI-specific logic mixed with collection logic

## Why Rust

Rust was chosen for both technical and personal reasons:

- it is strong at modeling partial, structured, fallible system data
- it supports a disciplined separation between collectors, models, and renderers
- it is a good foundation for a future TUI without rewriting the core
- it is the systems language the maintainer is actively investing in for the long term

More detail is available in [docs/en/why-rust.md](docs/en/why-rust.md).

## Current Scope

Supported today:

- Linux-only
- `screen` and `json` output
- `System`
- `Machine`
- `CPU`
- `Memory`
- `Graphics`
- `Network`
- `Drives`
- `Partitions`
- `Swap`
- `Info`

Important non-goals for the current release line:

- full feature parity with upstream `inxi`
- non-Linux platforms
- TUI implementation
- broad legacy option compatibility

See [docs/en/v1-scope.md](docs/en/v1-scope.md).

## Safety Model

`inxi-rs` is designed to stay observational.

Current safety rules:

- no filesystem writes
- no writes to `/proc`, `/sys`, or `/dev`
- no shell execution
- no privilege escalation
- no network access
- external commands only when audited, fixed-argument, and trusted

The project exposes this policy in the report itself so users can inspect it
instead of trusting undocumented behavior.

See [docs/en/safety.md](docs/en/safety.md).

## Quick Start

Build and run:

```bash
cargo run -p inxi-cli -- -b
```

JSON output:

```bash
cargo run -p inxi-cli -- -b --output json
```

Collector diagnostics:

```bash
cargo run -p inxi-cli -- --self-check
```

Quality checks:

```bash
cargo fmt --all --check
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

## Documentation

English:

- [Documentation Index](docs/en/index.md)
- [Architecture](docs/en/architecture.md)
- [Safety Model](docs/en/safety.md)
- [Compatibility Review](docs/en/compatibility.md)
- [Roadmap](docs/en/roadmap.md)

Italian:

- [Indice Documentazione](docs/it/index.md)
- [Panoramica](docs/it/panoramica.md)
- [Sicurezza](docs/it/sicurezza.md)
- [Roadmap](docs/it/roadmap.md)

## Inspiration And Attribution

`inxi-rs` is freely inspired by `inxi`, originally developed by Harald Hope and
contributors.

This repository does not attempt to present itself as upstream, as a drop-in
replacement, or as an official port. The intent is to learn from the design
strengths of `inxi` while building a Rust-first implementation with different
internal architecture and project goals.

## License

This project is licensed under `GPL-3.0-or-later`.

See [LICENSE](LICENSE).
