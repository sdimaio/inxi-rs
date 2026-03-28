# Publishing To GitHub

This project is currently prepared as a standalone repository inside a larger
workspace that also contains the original upstream Perl source tree.

Do **not** publish the parent repository as-is if your goal is to present
`inxi-rs` as an independent Rust project.

## Recommended Repository Name

- `inxi-rs`

## Recommended Description

- `A safety-first, read-only Rust system information tool for Linux, inspired by inxi.`

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
