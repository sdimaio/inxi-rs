# Why Rust

`inxi-rs` uses Rust because Rust is a good fit for this exact problem, not
because it is fashionable.

## Technical Reasons

This project has a domain full of:

- optional fields
- partial data
- fallback paths
- explicit warning states
- strongly structured JSON

Rust is particularly strong at modeling these constraints without collapsing
everything into loosely typed maps or ad hoc string-based state.

It also encourages a clean split between:

- raw source access
- normalized models
- rendering

That matters a lot for a tool that wants to be reliable and extensible.

## Project Reasons

The project is not only a CLI utility. It is also a systems-oriented codebase
that wants:

- a reusable core
- disciplined error handling
- testable behavior
- a future path toward a TUI

Rust supports that trajectory better than a quick throwaway implementation.

## Maintainer Reasons

This repository also reflects a deliberate personal direction by the maintainer,
Simmaco Di Maio.

The maintainer comes from a long background across:

- Pascal / Object Pascal
- C and C++
- Delphi and Borland C++ Builder
- Java as the primary professional language for many years

Rust is being treated here as the systems language to invest in for the next
phase of work, especially where strong modeling and operational safety matter.

## Why Not Treat This As A Perl Rewrite

The original `inxi` is valuable, but copying its implementation style would not
serve the goals of this project.

Rust was chosen to support a redesign, not only a migration.
