# inxi vs inxi-rs Compatibility Review

Date: `2026-03-27`

Platform used for the comparison:

- Linux laptop
- Ubuntu MATE `20.04.6 LTS`
- Kernel `5.4.0-216-generic`

Commands used:

- `./inxi -b -z`
- `cargo run -q -p inxi-cli -- -b -z`
- `./inxi -G -N -D -P -z`
- `cargo run -q -p inxi-cli -- -G -N -D -P -z`
- `./inxi -S -M -C -m -I -j -z`
- `cargo run -q -p inxi-cli -- -S -M -C -m -I -j -z`

## Overall conclusion

`inxi-rs` is already semantically credible for the current v1 scope.

It is **not** output-compatible with `inxi`, and that is acceptable. The goal of
this review was to verify whether the Rust clone reports the same machine facts
with a coherent, safe, and inspectable model.

Conclusion by category:

- `System`: good
- `Machine`: good, but narrower
- `CPU`: good, different presentation
- `Memory`: good baseline, less rich
- `Graphics`: useful but clearly narrower
- `Network`: useful but intentionally interface-centric
- `Drives`: good per-device data, missing aggregate summary
- `Partitions`: good v1 baseline
- `Swap`: good
- `Info`: good

## Accepted design differences

These are differences from `inxi` that are acceptable for `v0.1.0`:

- `inxi-rs` shows `Meta` and `Safety` blocks. `inxi` does not. This is intentional.
- `inxi-rs` uses explicit topology terms for CPU instead of labels like `MT MCP`.
- `inxi-rs` models network by interface state and addresses, not by PCI marketing names.
- `inxi-rs` keeps a machine-readable `JSON` contract and source traces. `inxi` is primarily a screen tool.

## Section-by-section notes

### System

Observed parity:

- kernel release
- architecture
- distro identity
- desktop/session

Assessment:

- no blocking gap found

### Machine

`inxi` currently reports more motherboard and firmware flavor details:

- machine type
- firmware style like `UEFI`
- firmware date
- some board serials

`inxi-rs` currently reports:

- vendor
- product
- family
- board
- firmware vendor/version

Assessment:

- acceptable for `v0.1.0`
- firmware date is a good follow-up improvement
- machine type is useful but not blocking

### CPU

`inxi` presents:

- human-friendly CPU classification
- cache details
- per-core current speeds

`inxi-rs` presents:

- vendor and model
- logical/package/core/thread topology
- current/min/max frequency

Assessment:

- acceptable difference
- our presentation is less familiar but architecturally clearer

### Memory

`inxi` presents:

- total
- available
- used
- RAM report diagnostics

`inxi-rs` presents:

- total
- available
- used

Assessment:

- good baseline
- the core `used` metric is now present and materially closer to `inxi`

### Graphics

`inxi` presents richer session/display metadata:

- X/Wayland server details
- display stack details
- API/tool diagnostics

`inxi-rs` currently presents:

- GPU identity
- driver
- connectors and display resolutions

Assessment:

- usable but narrower
- not blocking for `v0.1.0`
- likely post-stable or late pre-stable enhancement

### Network

`inxi` focuses on network devices and kernel drivers.

`inxi-rs` focuses on:

- interfaces
- inferred link kind
- state
- addresses

Assessment:

- intentional divergence
- acceptable for `v0.1.0`
- if we later want closer parity, the next step is adding driver names per interface

### Drives

`inxi` presents:

- aggregate local storage total
- aggregate used
- per-device summaries

`inxi-rs` presents:

- per-device summaries with type hints like `SSD`/`HDD`

Assessment:

- good baseline
- aggregate total/used is still missing and would add value

### Partitions

`inxi` highlights mounted partitions with usage.

`inxi-rs` currently shows:

- partition path/name
- filesystem
- mountpoint
- size
- filesystem total/used percentage when available
- parent device

Assessment:

- acceptable for `v0.1.0`
- the main parity gap is now narrower because mounted partition usage is present

### Swap

Observed parity:

- swap path
- size
- used

Assessment:

- no blocking gap found

### Info

Observed parity:

- uptime
- process count

Deliberate differences:

- `inxi-rs` shows shell, user, terminal, locale
- `inxi` shows client/inxi version in a different style

Assessment:

- no blocking gap found

## Fixes applied after this review

The review already led to concrete improvements:

- storage warnings are now attributed to `drives` or `partitions` instead of a generic `storage`
- graphics now emits an explicit warning when `xrandr` is unavailable on a display-capable session
- CLI behavior is now covered by focused tests
- memory now exposes `used`
- partitions now expose filesystem usage when `lsblk` provides it

## Pre-stable priorities that remain

Recommended before calling the first stable version:

1. freeze CLI contract
2. freeze JSON contract
3. run the same comparison on at least one more Linux host

## Second-host status

I attempted to discover a second local Linux environment that could be used
without leaving the current safety perimeter.

Observed state:

- `/run/systemd/machines` exists but is empty
- no active local systemd machine was discoverable
- `docker` is installed but the daemon is not running

Conclusion:

- a second-host comparison could not be executed from the current environment
- the procedure for that comparison is documented separately
