---
format: aep.planning-md/1
id: story:browser-media-adapter
kind: story
status: draft
title: Adapt browser-native audio
summary: Audio-only RTCPeerConnection adapter; media failures never present as SIP failures.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:signalling-over-wss
scope:
- confidence: inferred
  path: Cargo.toml
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/softphone-shell
- confidence: inferred
  path: docs
- confidence: inferred
  path: ess/domains/bridge.yaml
revision: 10
---
# Adapt browser-native audio

## Outcome

An audio-only `RTCPeerConnection` adapter turns the kernel's negotiated-media report into a live
media session, and turns browser media failures into `softphone.control.FailCall` with
`EndCause::Media` rather than into a SIP cause.

## Acceptance

An established call carries non-silent audio in both directions in both SIP roles. A denied
microphone permission ends the call with `softphone.control.EndCause::Media` and never presents it as
connected — and the model now holds that rather than hoping for it: a call reaches `Active` only
through `MediaConnected`, which only `Kernel` may issue, so `Answering` is where a call whose media
failed stops.

## Scope

Derived 2026-09-04 by `story-scoper`, confidence **medium** — high that work remains and that it is
page-side, **low on the exact file**, because no adapter file exists anywhere in the tree and the
story names none in this repository. Paths are relative to `widgets/phone`.

**No line is cited.** Every path the story names belongs to the pinned `codewandler/sipx` checkout;
`docs/` here holds only generated ESS documentation.

| path | mark | why |
|---|---|---|
| `crates/softphone-shell` | inferred | the adapter is an effectful driver and this is the only hand-written thing the browser loads. The page is emitted into gitignored `.build/synth/web` and stays as emitted, and the one hand-written-JS seam in the tree belongs to the scenario player — so `RTCPeerConnection` would be reached through `web-sys` from here |
| `Cargo.toml` | inferred | `members` is an explicit list, so a separate adapter crate edits it |
| `Taskfile.yml` | inferred | only if the adapter ships a file the emitted page must load |
| `ess/domains/bridge.yaml` | inferred, low | only if the five browser failure kinds must be distinguishable in the model. `BridgeCause` is `Local | Remote | Transport | Refused` and carries none of them; `EndCause::Media` is one word |

**Not in scope, against the story's own text.** `crates/softphone-behaviour` — the bridge
obligations are filled and that crate is pure. `ess/domains/sip.yaml` — the four SDP commands the
story names are the server's leg now, which `ess/topology.yaml` states.

**The story is stale and should be rewritten before it is worked.** Its offer/answer paragraph puts
the browser's exchange in `softphone.sip`; under the accepted ADR it is
`softphone.bridge.ConnectBridge`/`ConfirmBridge` carrying `softphone.bridge.SessionDescription`,
declared separately on purpose. Two things it claims as work are already done: the four
`MediaProfile` invariants exist at `ess/domains/media.yaml:79-83`, and the model gate — a call
reaches `Active` only through `MediaConnected` — is proved by `scenarios/media-fails-after-answer.yaml`.

**No harness for its acceptance.** "Non-silent audio in both directions" cannot be asserted by
anything in the gate: `tests/suite.mjs` runs under node and `task check` never opens a browser.

**Would collide with** any unit touching `crates/softphone-shell`, `Cargo.toml` or `Taskfile.yml`.

## Cited from

- `docs/specs/browser-sdk.md:854` — "§5.4 media flow over `RTCPeerConnection`, §6.2 established
  gate, §8.2/§8.5 at the media edge | browser media adapter | `M-52`".
- `docs/specs/browser-sdk.md:539` — `SipxMediaError` kinds: permission, device, autoplay,
  negotiation, track-ended; "browser media failures never masquerade as SIP failures".
- `docs/specs/browser-signalling.md:215` — "Anything to do with media. `RTCPeerConnection` is
  `M-52`'s."
- `README.md` — "The browser owns `RTCPeerConnection`, ICE, DTLS-SRTP, capture and render"; no TURN
  relay, host and STUN-derived ICE candidates only.

## Teardown

Microphone denial, a missing media API, negotiation failure, connection loss and a remote BYE all
converge on one idempotent teardown: stop source tracks, disconnect the graph, close the
`AudioContext`, clear queued playback, and ignore callbacks arriving from a superseded connection
generation.

Read from `~/babelforce/projects/ai-agent-platform/docs/designs/browser-voice.md` §5, which is the
implemented version of this for RTVBP, and whose §7 records that jitter behaviour is bounded by a ring
buffer policy rather than solved.
