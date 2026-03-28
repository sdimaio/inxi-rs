# Compatibility Review

`inxi-rs` is inspired by `inxi`, but it does not aim for textual or internal
implementation parity.

## Compatibility Philosophy

The project tries to preserve what is valuable:

- section semantics
- operational usefulness
- privacy filtering
- robust handling of partial data

It intentionally does not preserve:

- identical wording
- identical layout
- layout-encoded internal structures
- every historical option of upstream `inxi`

## Current Assessment

The current Linux implementation is already semantically credible for:

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

Current meaningful differences remain mostly in presentation depth and in some
advanced metadata that upstream `inxi` reports.

## Documents

The detailed review lives in the Italian research notes and in the compatibility
work carried out during project bootstrap.

For the current public repository, the practical conclusion is simple:

- `inxi-rs` is publishable as an alpha
- it is not ready to claim feature parity
- it is already serious enough to invite technical review
