# Publishing To GitHub

This project is currently prepared as a standalone repository inside a larger
workspace that also contains the original upstream Perl source tree.

Do **not** publish the parent repository as-is if your goal is to present
`inxi-rs` as an independent Rust project.

## Recommended Repository Name

- `inxi-rs`

## Recommended Description

- `A safety-first, read-only Rust system information tool for Linux, inspired by inxi.`

## Recommended About Box

Use these values in the GitHub `About` panel.

Description:

- `A safety-first, read-only Rust system information tool for Linux, inspired by inxi.`

Website:

- leave empty for now

Rationale:

- a duplicated GitHub repository URL adds no value as a website entry
- it is better to leave the field empty until there is a real documentation site,
  project page, or demo destination

Topics:

- `rust`
- `linux`
- `system-information`
- `sysinfo`
- `cli`
- `json`
- `diagnostics`
- `hardware`
- `observability`
- `tui`

## Recommended Visibility

- public

## Recommended Initial Release Label

- `0.1.0-alpha.1`

## What To Publish

Publish the contents of this directory as the repository root:

- `inxi-rs/`

Do not publish:

- the parent Perl repository
- local IDE metadata
- build artifacts

## Suggested Workflow

From the parent directory:

```bash
cp -a inxi-rs /tmp/inxi-rs-publish
cd /tmp/inxi-rs-publish
git init
git add .
git commit -m "Initial public alpha"
git branch -M main
git remote add origin git@github.com:sdimaio/inxi-rs.git
git push -u origin main
```

## Before The First Push

Run:

```bash
cargo fmt --all --check
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

## After The First Push

Recommended follow-up actions:

1. enable GitHub Actions
2. create the first tag: `0.1.0-alpha.1`
3. open a short pinned issue listing the current non-goals
4. keep issue reports focused on Linux

## Create The First Tag

From the repository root:

```bash
git tag -a 0.1.0-alpha.1 -m "0.1.0-alpha.1"
git push origin 0.1.0-alpha.1
```

## First Release Notes

The release notes are already prepared in:

- `docs/en/releases/0.1.0-alpha.1.md`
- `docs/it/releases/0.1.0-alpha.1.md`

If `gh` is authenticated, the release can be created with:

```bash
gh release create 0.1.0-alpha.1 --title "0.1.0-alpha.1" --notes-file docs/en/releases/0.1.0-alpha.1.md
```

If `gh` is not authenticated, create the release in the GitHub web UI and paste
the English release text from the file above.
