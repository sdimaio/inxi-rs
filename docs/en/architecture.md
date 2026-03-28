# Architecture

`inxi-rs` is intentionally not designed as a single CLI binary with embedded
collector logic.

The architecture follows a stricter split:

- `inxi-core`
- `inxi-cli`
- future TUI frontend

## Design Goals

The architecture is optimized for:

- read-only system inspection
- deterministic output
- modular collectors
- stable JSON
- future reuse by a TUI

## Current Structure

`inxi-core` contains:

- runtime capability scanning
- request and planning types
- data model
- collectors
- safety policy
- audited command execution
- screen and JSON rendering
- self-check rendering

`inxi-cli` contains:

- CLI parsing
- translation from flags to a stable request model
- frontend selection between normal rendering and self-check

## Why This Split Matters

This split is not aesthetic.

It exists because the project intends to support a future text UI without
rebuilding the collection pipeline. If collectors or planners knew too much
about terminal rendering, that future path would become expensive immediately.

## Data Flow

The current pipeline is:

1. parse CLI flags into `Request`
2. scan runtime `CapabilityReport`
3. build an `ExecutionPlan`
4. collect section data
5. aggregate warnings and source traces
6. render to `screen`, `json`, or `self-check`

## Collector Philosophy

Collectors are expected to:

- stay read-only
- prefer safe local files first
- use audited commands only when justified
- surface degraded behavior through warnings
- preserve provenance through source tracing

## Rendering Philosophy

The textual renderer is intentionally not trying to mimic the original `inxi`
format field by field.

The project favors:

- stable output
- readability
- low noise
- alignment with the underlying data model

## Future TUI

The TUI is a planned frontend, not a sidecar feature.

That is why the project is already being designed around:

- frontend-neutral requests
- structured section envelopes
- explicit safety metadata
- self-check diagnostics

The architectural requirement is simple: the TUI must be able to sit on top of
the same core used by the CLI.
