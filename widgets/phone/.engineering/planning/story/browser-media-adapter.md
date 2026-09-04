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
revision: 7
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

This is upstream story `M-52`, unshipped at the pinned revision. Written here against
`docs/specs/browser-sdk.md` §5.4, §6.2 and §8.2/§8.5, which are normative and complete enough to
implement without the package. Where it lands is settled:
`architecture-decision-record:media-adapter-lands-here`.

The adapter does not implement a domain. It **fills obligations** — 68 of them across the
specification, 52 command behaviours and 16 view queries, listed on the generated *Obligations* page.
This story fills the SIP binding's share with a real `RTCPeerConnection`.

The neutral port is what it satisfies: `MediaProfile` is `pcm_s16le`/8000/1/20 ms/320 B and now says
so as four checked invariants rather than as a comment; signals are DTMF only; and a call ends with
one of the six `softphone.control.EndCause` words, none of which is SIP's.

The offer/answer exchange it drives lives in `softphone.sip` as four commands —
`RequestLocalMedia`, `OfferLocalMedia`, `ApplyRemoteMedia`, `FailLocalMedia` — and a dialog carrying
`local_sdp` and `remote_sdp`. None of that existed when this story was first written.

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
