---
format: aep.planning-md/1
id: story:media-session-domain
kind: story
status: implemented
title: Model the neutral media session
summary: softphone.media from the existing Rust port, plus softphone.local as the second carrier.
relations:
- decomposes: epic:browser-softphone
revision: 5
---
# Model the neutral media session

## Outcome

`softphone.media` states what a media session is without naming a protocol, and `softphone.local`
exists as a second carrier so that claim has evidence behind it.

## Acceptance

`ess validate --path ess` accepts the domain. **No construct in `softphone.media` references a
binding**, checked against the compiled IR rather than the source: `ess compile --format json`, then
search `domains."softphone.media"` for `softphone.sip.*` or `softphone.local.*` — it returns nothing.

That replaces the acceptance this story shipped with, which was a `grep` over the YAML for `Sip` and
a claim that it returned empty. It returned five lines. Two of them were the header comment asserting
the very thing, and the other three were an `owns` relation naming `softphone.sip.SipDialog` and the
prose explaining it. The relations moved to the binding side —
`architecture-decision-record:media-session-owns-nothing` — and the claim is now narrower and
mechanical.

Two bindings own a `MediaSession` reference, not one, so the neutrality has a second carrier behind
it. `MediaProfile`'s four numbers are checked invariants rather than a comment.

## What was built

`ess/domains/media.yaml`, modelled construct-for-type from the Rust port that already existed:
`connectors/crates/domain/src/voice.rs`.

| ESS construct | Read from |
|---|---|
| `MediaProfile` | `MediaDescriptor` (`:68`); the single accepted value `pcm_s16le`/8000/1/20 ms/320 B (`:78-95`), so `SampleFormat` has one member |
| `ChannelSignal` + `ChannelSignalKind` | `ChannelSignal::Dtmf { digits }` (`:114-135`); split in two because an ESS enum carries no fields |
| `TerminationReason` | `:137-147`, all eight variants unchanged |
| `Participant`, `ContextTrust` | `ParticipantContext` (`:57`), `ContextTrust::Untrusted` (`:49`) |
| `SessionRef` | `VoiceRef` (`:33`); opacity modelled, the 128-byte cap left to the host |
| the six commands | one per `TelephonySession` method that is not the frame path (`:163-229`) |

`softphone.local` is `ess/domains/local.yaml`, cited from `connectors/crates/voice-local-audio` —
the sibling of `driver-sip` behind the same port.

## One deliberate difference from the port it was read from

Upstream, establishment happens *before* a neutral session exists: `RuntimeError` is documented as
"Failure before a neutral telephony session exists"
(`connectors/crates/voice-runtime/src/lib.rs:307-309`). Here it is the `Requested -> Live` move, so
the ESS session starts one state earlier than the Rust port's. Written into the domain `summary` so
the next reader does not find a contradiction and assume one side is wrong.

## The limit that could not be modelled

A `MediaSession` owns a `SipDialog` and a `LoopbackDevice`, one each, and at runtime exactly one
exists. ESS cannot state that: `crates/specify/ess-domain/src/entity.rs:821-822` says an invariant
reads only its own entity's fields, identity and `state`, so a predicate over a related entity's
existence is refused. `binding: BindingKind` is the discriminator and the two `creates` outcomes —
`softphone.sip.OpenDialog` and `softphone.local.AttachLoopback` — are what hold it. No invariant
pretends otherwise.
