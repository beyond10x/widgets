---
format: aep.planning-md/1
id: story:fill-the-obligations
kind: story
status: implemented
title: Fill every obligation so the specified phone runs
relations:
- decomposes: epic:browser-softphone
scope:
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/softphone-behaviour
- confidence: cited
  path: crates/softphone-shell
revision: 5
---
# Fill every obligation so the specified phone runs

## Outcome

The specification executes. Every command has behaviour, every declared view answers with rows, and
the eleven authored scenarios pass against the filled system rather than being replayed by a page.

`docs/obligations.md` counts 344 capabilities: 248 generated, **75 obligations**, 21 refused. This
story is the 75.

## Scope

`widgets/phone` becomes one cargo workspace, `members = ["crates/*"]`, `exclude = [".build"]`.

- `crates/softphone-behaviour` — one module per domain implementing every `*Behavior` and `*Query`
  in `softphone_types::<domain>::obligations` over an in-memory store.
- `crates/softphone-shell` — `softphone_system::System<…>` over those bundles, exporting
  `ess_input_reserve`, `ess_dispatch` and `ess_output_len`, the three symbols the generated
  `bridge.js` calls (`.build/synth/web/crates/softphone-web/src/lib.rs:1741-1768`).

Nothing generated is edited. The generated `Bound` trait is implemented for the *generic* `System`
(`same file:169`), so the whole JSON wire is reused; only `serve()` hard-wires `Unimplemented`
(`:1216`), which is why the exports are ours.

## Acceptance

`task scenarios` writes `.build/suite.json`; a test in `softphone-behaviour` reads it and drives
every step through `Bound::run`, asserting each declared outcome and each declared event. Green
means the specification is executable, not that any deployment works.

Loopback audio — `softphone.local.LoopbackDevice`, `BindingKind::Loopback` — is what makes the
result usable with no server: microphone to speaker, in a browser, with dialling, answering, mute,
hold, the keypad, the log and the phonebook all working.
