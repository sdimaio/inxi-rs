# V1 Scope

The first stable line of `inxi-rs` is intentionally narrower than the original
`inxi`.

That is a design decision, not a weakness in disguise.

## Supported In V1

- Linux-only
- CLI frontend
- `screen` output
- `json` output
- privacy filtering
- self-check diagnostics

Supported sections:

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

## Explicitly Deferred

- `Audio`
- `Battery`
- `USB`
- `Sensors`
- `Bluetooth`
- `RAID`
- `Repos`
- `Processes`
- TUI frontend
- BSD backends

## Why The Scope Is Narrower

The project is optimizing for:

- correctness
- inspectability
- maintainable growth

The alternative would be to claim wide surface coverage early and accumulate a
large amount of unstable behavior before the data model and safety story are
fully hardened.
