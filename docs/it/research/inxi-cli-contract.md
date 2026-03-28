# inxi-rs CLI Contract

Date: `2026-03-27`

This document defines the CLI behavior we consider stable enough to protect for
the first release line.

## Supported flags in scope

Section selection:

- `-b`, `--basic`
- `-S`, `--system`
- `-M`, `--machine`
- `-C`, `--cpu`
- `-m`, `--memory`
- `-G`, `--graphics`
- `-N`, `--network`
- `-D`, `--disk`
- `-P`, `--partition`
- `-j`, `--swap`
- `-I`, `--info`

Detail and privacy:

- `-x`
- `-xx`
- `-a`, `--admin`
- `-z`, `--filter`
- `-Z`, `--no-filter`

Output and diagnostics:

- `--output screen`
- `--output json`
- `--self-check`

## Semantics

### Default request

No section flags means:

- same behavior as `-b`
- output format `screen`
- privacy filter disabled

### `-b`

`-b` expands to the project-defined basic set:

- `System`
- `Machine`
- `CPU`
- `Memory`
- `Graphics`
- `Network`
- `Drives`
- `Swap`
- `Info`

Important:

- this is **our** stable basic set
- it is not a promise to match the original `inxi -b`

### Detail flags

Current mapping:

- default with explicit non-basic sections: `normal`
- default with `-b` or implicit basic mode: `basic`
- `-x`: `extended`
- `-xx` or more: `full`
- `-a`: `admin`

`-a` does not relax safety policy and does not imply privilege escalation.
It only changes requested detail level.

### Privacy flags

`-z` enables filtering for fields currently considered sensitive:

- IP addresses
- MAC addresses
- UUIDs
- user-identifying mount path fragments

`-Z` explicitly disables filtering.

When both would be present, the CLI rejects the combination.

### `--self-check`

`--self-check` does not collect different data.

It renders a diagnostic view of the same request, exposing:

- requested sections
- capabilities
- safety contract
- section state
- sources used
- warnings
- fallback usage

## Explicit non-goals for `v0.1.0`

The following should not be assumed stable yet:

- compatibility with the broader original `inxi` option surface
- exact screen formatting parity with `inxi`
- support for unsupported sections such as `Audio`, `Battery`, `USB`, `Sensors`

## Contract discipline

For `v0.1.0`, we should avoid:

- reusing existing flags with different meaning
- silently changing what `-b` expands to
- renaming current long options
- changing privacy semantics without updating this file and tests
