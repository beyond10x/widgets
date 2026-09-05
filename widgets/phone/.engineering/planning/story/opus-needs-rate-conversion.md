---
format: aep.planning-md/1
id: story:opus-needs-rate-conversion
kind: story
status: draft
title: Opus on the browser leg, with the rate conversion it needs
relations:
- decomposes: epic:browser-softphone
- informed_by: review-result:adversary-phone-server-pass-1
- depends_on: story:phone-server
scope:
- confidence: cited
  path: crates/phone-server/src/bridge.rs
- confidence: cited
  path: crates/phone-server/tests/adversary_bridge.rs
revision: 2
---
# Opus on the browser leg, with the rate conversion it needs

## What was measured

`review-result:adversary-phone-server-pass-1`: 500 ms spoken on an Opus browser leg arrives at the
G.711 far end as **23,040 samples — 2,880 ms**. The far end hears the browser sped up six times.
`sipx_media::Bridge::spawn_leg` moves buffers between two legs with no rate conversion
(`sipx-media-1.0.1/src/bridge.rs:151-168`), and 48 kHz frames written into an 8 kHz session are
simply more samples than the session's clock expects.

`crates/phone-server`'s correction round takes the fail-closed way out: the browser leg answers
PCMU, or refuses an Opus acceptance with the reason named. That keeps both legs at 8 kHz and the
bridge a pass-through, and it costs the profile's primary codec.

## What this story is

Opus on the browser leg, rated correctly. Two ways, and choosing between them is the first act:

**Resample in `phone-server`.** A 48 kHz ↔ 8 kHz converter on the transcoding leg. It is ours, it is
testable here, and a naive one is an audible defect — a decimating resampler with no low-pass filter
aliases, and the artefact is exactly the kind nobody notices in a test and everybody notices on a
call.

**Resample in sipx.** `sipx-media`'s own bridge is where the rate is known on both sides, and sipx
already carries rate-converting PCM playback and recording, so the machinery exists on one side of
the crate. This is an upstream story in `codewandler/sipx`, not here, and it makes every consumer's
bridge correct rather than this one.

The second is the better answer and the slower one. Neither is a prototype's next step, which is why
`phone-server` shipped without it.

## Acceptance

500 ms spoken on an Opus browser leg is 500 ms at a G.711 far end, measured at the far end and not
at the seam — the case that found this is `crates/phone-server/tests/adversary_bridge.rs:71`, and it
asserts the sample count. Plus one that says an aliased conversion is distinguishable from a clean
one, or a statement of why that cannot be asserted here.

## Scope

Derived by the coordinator from the review, confidence **high** — the sample count was measured.

- `crates/phone-server/src/bridge.rs` — cited, if the conversion lands here
- `crates/phone-server/tests/adversary_bridge.rs` — cited, the case that measures it
- **Would collide with** any unit inside `crates/phone-server`.
