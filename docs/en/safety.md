# Safety Model

`inxi-rs` is intentionally designed as an observational tool.

This is not just a promise in the README. The safety model is implemented in
the codebase and exposed in the report itself.

## Safety Rules

Current rules:

- no filesystem writes
- no writes to `/proc`, `/sys`, or `/dev`
- no shell execution
- no network I/O
- no privilege escalation
- no arbitrary external commands

## Allowed Read Roots

Reads are intentionally constrained to explicit roots:

- `/etc`
- `/proc`
- `/sys`
- `/usr/lib`

This is also how the project avoids accidental interaction with device nodes
such as `/dev/*`.

## External Commands

External commands are allowed only when all of the following are true:

- the command is explicitly whitelisted
- the argument vector is fixed
- the executable resolves under trusted system roots
- the process runs without a shell
- the environment is minimized
- the process has a short timeout

## Why This Matters

System information tools are often assumed to be harmless until they grow a
long tail of edge-case behaviors.

`inxi-rs` takes the opposite approach: the safety perimeter is part of the
design and should stay visible and testable as the project evolves.
