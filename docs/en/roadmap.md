# Roadmap

The near-term goal is not feature explosion.

The near-term goal is a disciplined first stable release.

## Before 0.1 Stable

The main work items are:

1. multi-host comparison against upstream `inxi`
2. further warning and fallback hardening
3. freeze of CLI semantics
4. freeze of JSON semantics
5. documentation refinement

## After 0.1 Stable

Good candidates for post-stable work:

- richer graphics metadata
- richer drive summaries
- additional sections such as `Audio`, `Battery`, `USB`, and `Sensors`
- BSD backend exploration
- TUI frontend design and evaluation

## TUI Direction

The TUI is a future frontend, not a replacement for the CLI.

The repository is being shaped so that a text UI can later reuse:

- the request model
- the collectors
- the JSON-friendly section model
- the self-check diagnostics
