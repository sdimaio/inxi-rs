# Contributing

Thank you for considering a contribution to `inxi-rs`.

This project is still in an early public phase, but it is intentionally being
built with long-term maintainability in mind. Contributions are welcome when
they improve the tool without weakening its safety, clarity, or architecture.

## Project Priorities

The current priorities are:

- correctness on Linux
- explicit safety boundaries
- stable internal modeling
- useful diagnostics
- predictable screen and JSON output

The project is not optimizing for quick feature accumulation at any cost.

## Before Opening A PR

For anything beyond a trivial fix, please open an issue first.

This is especially important for:

- new collectors
- CLI surface changes
- JSON schema changes
- safety model changes
- architectural refactors

## Safety Expectations

`inxi-rs` is designed to remain observational.

Contributions must not introduce:

- filesystem writes
- shell-based command execution
- network I/O
- privilege escalation
- arbitrary external command execution

If a collector needs an external tool, the command must be:

- explicitly audited
- fixed-argument
- trusted-path only
- covered by timeout and tests

## Code Style

The codebase favors:

- small focused modules
- explicit data modeling
- structured warnings instead of hidden behavior
- deterministic output

## Rustdoc And Comments

Use Rustdoc and comments sparingly but intentionally.

The standard for project documentation is:

- explain why a design exists
- explain requirements and constraints
- explain trade-offs
- do not restate obvious code line by line

This applies to both public API docs and internal comments.

## Tests

All non-trivial contributions should include appropriate tests.

Depending on the change, this may mean:

- unit tests
- parser fixture tests
- golden output tests
- CLI behavior tests

Run before submitting:

```bash
cargo fmt --all --check
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

## Documentation

If you change any of the following, update documentation in the same PR:

- supported sections
- CLI behavior
- JSON contract
- safety policy
- roadmap assumptions

English documentation is the main public reference. Italian documentation is
maintained where it adds value for project context and origin.
