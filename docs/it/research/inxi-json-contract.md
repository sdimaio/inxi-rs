# inxi-rs JSON Contract

Date: `2026-03-27`

This document defines the JSON stability rules for the first stable line.

## Primary report shape

Current top-level object:

- `meta`
- `sections`
- `warnings`
- `capabilities`
- `safety`

## Section envelope shape

Each collected section is stored under `sections.<name>` with this envelope:

- `state`
- `value` if available
- `sources`

Rationale:

- `state` explains whether the section is usable
- `value` carries the actual payload
- `sources` preserves provenance for debugging and trust

## Stability rules for `v0.1.0`

Allowed changes:

- adding new optional fields
- adding new optional sections
- adding new warning codes
- adding new source entries

Disallowed changes without a version bump:

- renaming top-level keys
- renaming current section keys
- removing current fields
- changing field types
- changing `snake_case` naming conventions

## Self-check JSON

`--self-check --output json` is a separate but related contract.

Current top-level keys:

- `meta`
- `request`
- `capabilities`
- `safety`
- `sections`
- `warnings`

This output is diagnostic and may grow faster than the primary report, but the
same discipline should apply to existing keys once `v0.1.0` is tagged.

## Interpretation rules

Consumers should treat:

- missing section keys as "not requested" or "not collected"
- `state != "ok"` as authoritative even if future payload fragments exist
- `warnings` as supplemental diagnostics, not as the sole source of failure state

## Design note

The JSON contract is intentionally not modeled after the textual `inxi` output.

Reason:

- screen output is optimized for humans
- JSON must be optimized for stable downstream consumption, tests, and future frontends
