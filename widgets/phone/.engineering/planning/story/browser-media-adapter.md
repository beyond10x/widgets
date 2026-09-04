---
format: aep.planning-md/1
id: story:browser-media-adapter
kind: story
status: active
title: Adapt browser-native audio
summary: Audio-only RTCPeerConnection adapter; media failures never present as SIP failures.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:signalling-over-wss
scope:
- confidence: inferred
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/softphone-shell
- confidence: inferred
  path: docs
- confidence: inferred
  path: ess/domains/bridge.yaml
- confidence: inferred
  path: web
- confidence: cited
  path: widget
- confidence: cited
  path: widget/src
revision: 18
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

## What `phone-server` requires of this adapter, measured

`crates/phone-server` refuses to bridge a browser leg whose audio rate differs from the SIP leg's,
at the media seam, before ICE and before the SIP call is placed. That refusal is not a limitation of
the SDP: `sipx_sdp::browser_audio::answer` **can** select PCMU, and the measurement that decided it
is in `review-result:adversary-phone-server-pass-1`'s correction —

| offer | answer | `validate_answer` |
|---|---|---|
| Opus first | Opus first, payload 111 both sides | `Ok` |
| **PCMU first** | **PCMU first, payload 0 both sides**, Opus still in the vocabulary | `Ok` |
| Opus first, answer hand-reordered to PCMU first | — | **`Err(CodecSetIncomplete)`** |

The answer must preserve the offer's format order, so the answerer has no say. **This adapter has
it.** Offering PCMU ahead of Opus is what makes a call bridge today, and it is one line in the
`RTCPeerConnection` setup rather than a negotiation.

Until it does, the server answers and then declines, with a refusal naming the pair and the way
through. `crates/phone-server/tests/browser_profile.rs`'s
`a_pcmu_first_offer_is_the_shape_this_server_can_bridge` is the checkable target: it proves the
PCMU-first path works end to end.

The alternative is `story:opus-needs-rate-conversion`, which is Opus with the rate conversion it
needs and belongs upstream in sipx, where both rates are known. This story does not wait for it.

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

## What landed 2026-09-05, and what is still unproven

The page half now exists: `widget/` is a pnpm package `@b10x/phone-widget` holding the module, one
control channel and one `RTCPeerConnection` together.

| file | what it owns |
|---|---|
| `widget/src/kernel.ts` | the realized module through the emitted `bridge.js`'s own `open()`; every command answers the whole observation, so nothing here shadows the module's state |
| `widget/src/transport.ts` | the WebSocket. `phone_server::wire::FromBrowser` out; inbound frames are already the module's own request shape and go in verbatim |
| `widget/src/call.ts` | `scenarios/bridge-carries-outbound-call.yaml`'s sequence at run time |
| `widget/src/PhonePanel.vue` | the screen, rendered from the projection and never from a copy |
| `widget/index.html`, `widget/src/dev.ts` | a harness, so the page runs with no devcenter |

**`AttachMedia` and `MediaConnected` are issued here and not by the server**, which is what
`crates/phone-server/tests/wire.rs:33-44` already said: only the page watches its own
`RTCPeerConnection`, and the page minted the media session before it said anything to the server.
Both are gated on *two* conditions rather than on the peer connection alone — `Call.activate` has
`Answering` as its only legal `from`, and DTLS completes as soon as the answer is applied, well
before a person picks up. Issuing `MediaConnected` on `connected` alone earns a `CallStateConflict`
and the call never becomes `Active`.

Measured against the realized module, 2026-09-05: the sequence walks `Requested → Ringing →
Answering → Active`, `ActiveCalls` gains its row only after `MediaConnected`, and `RecentCalls`
records the call on hang-up. The harness mounts in Brave headless, instantiates the module and logs
`softphone.control.EndpointConfigured`.

Two things the panel reads that are worth writing down. It renders
`softphone.control.CallById` rather than `ActiveCalls`, because `ActiveCalls` filters
`state == Active` and a ringing call is not in it — an `observe` that binds no parameter answers a
parameterised view over its whole source. And it draws no timestamps:
`story:the-module-has-no-clock` records why.

**This story's acceptance is not met.** It asks for non-silent audio in both directions and for a
denied microphone to end the call as `EndCause::Media`. Both need a browser with a microphone, and
neither has been observed. `widget/src/offer.mjs`, `call.ts` and `PhonePanel.vue` are unexercised by
any automated case for exactly that reason, and `task widget-check` says so in its own comment
rather than implying coverage it does not have.
