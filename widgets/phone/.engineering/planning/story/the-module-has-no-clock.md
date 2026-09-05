---
format: aep.planning-md/1
id: story:the-module-has-no-clock
kind: story
status: draft
title: The module has no clock, so the phone cannot say when a call happened
relations:
- decomposes: epic:browser-softphone
- informed_by: story:browser-media-adapter
revision: 1
---
# The module has no clock, so the phone cannot say when a call happened

## Outcome

A person looking at this phone's call history sees when each call started, was answered and ended —
or sees no time at all. What it must not do is show `1970-01-01T00:00:03Z` as though that were a
time, which is what every row carries today.

## What is actually true

`softphone.history.CallRecordRow` declares `started_at`, `answered_at` and `ended_at`, and
`crates/softphone-behaviour/src/lib.rs:130-139` fills all three from a counter:

```rust
Timestamp(format!("1970-01-01T{:02}:{:02}:{:02}Z", total / 3600 % 24, …))
```

That is not laziness in the behaviour crate. **The module has no clock to read.** The emitted
bridge instantiates it with an empty import object — `.build/synth/web/bridge.js`,
`WebAssembly.instantiate(source, {})` — so there is no host function the wasm could call for the
time, and `wasm32-unknown-unknown` has no other route to one. A counter is the only monotonic thing
available inside that boundary, and it is also what makes the 21 authored scenarios reproducible.

Measured 2026-09-05, driving the outbound sequence against the realized module: a completed call
records `started_at: 1970-01-01T00:00:01Z`, `answered_at: …:02Z`, `ended_at: …:03Z`.

## What this story is not

It is not "make `tick()` call `SystemTime::now()`". That does not compile to this target, and if it
did it would take the suite's determinism with it.

## The two ways out, both upstream in `ess`

1. **The host passes the instant in.** Every command already arrives as
   `{"request":"command","command":…,"input":{…}}`, and every authored scenario already carries an
   `at:` for each step — so the wire has a place for an instant and the scenario format has the
   word for it. The emitter would have to accept one and hand it to the obligation.
2. **The emitted bridge takes an import object.** `open(source)` would instantiate with
   `{ess: {now}}` and the generated system would call it. This is the larger change and it makes
   every host supply a clock.

Either settles it for every ESS web target, not just this phone, which is why neither belongs in
this repository.

## Meanwhile

`widget/src/PhonePanel.vue` renders the history rows without their timestamps, and says why in a
comment. Direction, remote address and termination are real; the times are not, so they are not
drawn.

## Acceptance

Either the history panel shows times a person can trust — sourced from something that knows the
time — or this repository carries a written statement that it cannot, with the upstream issue
linked. A screen showing 1970 satisfies neither.
