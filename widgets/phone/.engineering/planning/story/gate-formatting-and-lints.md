---
format: aep.planning-md/1
id: story:gate-formatting-and-lints
kind: story
status: draft
title: Gate the formatting and the lints this application has never run
relations:
- decomposes: epic:browser-softphone
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/softphone-behaviour/src
- confidence: inferred
  path: crates/softphone-shell
revision: 3
---
# Gate the formatting and the lints this application has never run

## Outcome

`task check` runs `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`, and both are
green.

## What was measured

`widgets/phone/Taskfile.yml`'s `check` is validate, compile, scenarios, drift, test. **Neither
`cargo fmt` nor `cargo clippy` has ever run in this application**, and both are red on
`wave/phone-usable` at `eb40ddc`:

| command | exit | what it says |
|---|---|---|
| `cargo fmt --check` | 1 | rustfmt 1.9.0-stable disagrees with the committed formatting of 7 files under `crates/softphone-behaviour/src/` |
| `cargo clippy -p softphone-behaviour --all-targets -- -D warnings` | 101 | 26 lints, `clone_on_copy`-class |
| `cargo clippy -p softphone-shell --all-targets -- -D warnings` | 101 | the same, plus one in the generated `softphone-web` |

There is no `rustfmt.toml`, so the formatting disagreement is with rustfmt's defaults.

`cargo test -p softphone-shell` also exits 101, and that one is **not** a defect: the crate carries
`compile_error!` for any target that is not `wasm32-unknown-unknown`, which is the generated web
bridge refusing to produce a module nobody can run. A gate that adds clippy has to scope it to the
wasm target for that crate or exclude it by name, and say which.

Read as a pair with `story:prove-every-refusal-branch`: both are gates this application declares
nowhere and therefore never fails.

## Why it is filed rather than fixed in place

It was found by `story:call-lifecycle`'s implementor, whose surface is `scenarios/` and
`tests/suite.mjs`; running `cargo fmt` would have rewritten seven files in another unit's surface.
`origin: pre-existing` — the code is `story:fill-the-obligations`', and that story never gated it.

## Acceptance

`cargo fmt --check` exits 0, `cargo clippy --all-targets -- -D warnings` exits 0 with the
`softphone-shell` case handled explicitly rather than by a blanket allow, and both are steps of
`task check`. No lint is silenced with an `#[allow]` that carries no reason.

## Scope

Derived by the coordinator from measured exit codes, confidence **high**.

- `crates/softphone-behaviour/src` — cited, the 7 unformatted files and the 26 lints
- `Taskfile.yml` — cited, `check` gains two steps
- `crates/softphone-shell` — inferred, only if the clippy invocation has to name it
- **Would collide with** any unit touching `Taskfile.yml` or `crates/**`.
