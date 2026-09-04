---
format: aep.planning-md/1
id: epic:browser-softphone
kind: epic
status: draft
title: Browser softphone
summary: Register over WSS and carry audio calls in a browser page, with media owned by RTCPeerConnection.
relations:
- derived_from: initiative:browser-softphone
revision: 6
---
# Browser softphone

## Scope

Register one SIP account over secure WebSocket, place and answer audio calls, keep a log of them and
a phonebook, and show it all in a browser page. Signalling and dialog state come from the sipx
WebAssembly session kernel (`crates/sipx-wasm`, story S-41) driven over the WebSocket binding
(`browser/src/`, story T-33), both present at the pinned revision. Media is negotiated by the
browser's own `RTCPeerConnection`.

Seven domains, one per concern, in system `softphone` v1:

| Domain | Concern |
|---|---|
| `softphone.media` | the media session; references no binding, checked against the compiled IR |
| `softphone.control` | the surface a human or an agent drives, identically |
| `softphone.history` | one durable record per finished call |
| `softphone.directory` | contacts and the addresses they are reachable at |
| `softphone.presentation` | the keypad, one tile per call, and which mode the screen is in |
| `softphone.sip` | the SIP binding, including the offer/answer exchange |
| `softphone.local` | the local-audio binding, whose existence makes the media layer's neutrality checkable |

They are joined by **eight `relations:`** entries, seven crossing a domain boundary, and by **seven
`bindings:`** — one event causing one command — which is what makes a layer react to another.

Counts, from `ess compile`: 52 commands, 52 events, 16 views, 12 entities, 7 invariants, 7 workloads,
13 `refs`. `ess validate --path ess` reports `softphone v1 — 10 file(s), valid`.
`ess generate synthesize --target rust` reports `307 capabilities: 221 generated, 68 obligation(s),
18 refused` and the emitted crates build clean.

`architecture-decision-record:layered-softphone-domains` holds why this shape rather than three
systems; `architecture-decision-record:media-session-owns-nothing` holds why bridges 2 and 3 point at
the media session rather than away from it.

## What upstream has not shipped

At the pinned revision two of M15's stories are pending, and both sit on the path to a working
softphone:

| Upstream story | What it would deliver | Cited |
|---|---|---|
| `M-52` | audio-only `RTCPeerConnection` adapter | `docs/specs/browser-sdk.md:854`, `browser/README.md` |
| `A-17` | the installable `@sipx/browser` package and the `SipxClient` lifecycle layer | `docs/specs/browser-sdk.md:855`, `docs/specs/browser-signalling.md:212` |

`docs/roadmap.md` states M15 "is tracked but is **not** part of the selected M13 wave", so neither is
scheduled. This project therefore writes the media adapter against the normative contract in
`docs/specs/browser-sdk.md` §5.4 and §6.2 rather than consuming a package, and vendors the
`browser/src/` binding rather than installing it.

That the adapter lands here rather than upstream is decided:
`architecture-decision-record:media-adapter-lands-here`.

## Done When

A browser page registers the configured address-of-record over WSS, places and answers an audio call
against a SIP endpoint, carries non-silent audio in both directions, and ends every call with every
socket, timer and media track observed closed.

## Prior art

This organisation has already shipped a browser phone, on a different protocol. It is worth reading
before this one is built, and it is cited rather than copied.

`~/babelforce/projects/ai-agent-platform/docs/designs/browser-voice.md` — status *Implemented*,
backing stories F-07, E-09, F-83. Its phone speaks RTVBP over WebSocket/WebRTC, not SIP, so none of
its protocol work transfers. Its shape does:

| What it settled | Where | Bearing on this epic |
|---|---|---|
| A thin product adapter over an authoritative SDK; the console holds no protocol, envelope, codec, resampler or worklet code | §1, §2 | the same split is `story:signalling-over-wss` (transport, vendored) against `story:call-lifecycle` (product) |
| Browsers cannot set an `Authorization` header on a WebSocket upgrade; the token rides the negotiated subprotocol list | §4 | a gap in `story:signalling-over-wss` until this was read |
| ICE/TURN reachability is the deployment's; the SDK ships no relay | §4, §7 | agrees with sipx `README.md`, which offers host and STUN-derived candidates and no TURN relay |
| One idempotent teardown path that every failure converges on, ignoring callbacks from an obsolete connection generation | §5 | `story:browser-media-adapter` and `story:call-lifecycle` |
| Microphone permission and audio autoplay need an explicit user gesture and a secure context | §3, §7 | `story:softphone-page` |
| Evidence shape: clean registry install, a static test that the adapter holds no protocol code, an in-memory session test, a real-browser non-silent echo, one bounded device smoke | §6 | the *Done When* above |

The code is at `sdk/typescript/src/rtvbp.ts` and
`web/packages/console/src/composables/useVoiceSession.ts` in that repository.

**One rule of theirs this epic cannot follow.** §1 says generated SDK output is never copied and the
lock must resolve the exact public npm version. sipx has no published browser package — `A-17` is
unshipped — so `story:signalling-over-wss` vendors `browser/src/` from a pinned Git revision instead.
That is the same substitution
`architecture-decision-record:media-adapter-lands-here` records for the media adapter, and it retires
the same way, when upstream ships.

## What is out of scope, and what merely is not built

Two kinds of absence, and they are not the same. The first was never in scope. The second is absent
because the **target contract** excludes it, and until now nothing in this store said so — the only
recorded exclusion was "video and data channels".

## Excluded by the sipx browser SDK v1 contract

`docs/specs/browser-sdk.md:830` §10, prefaced "this contract does not include, and completing the
epic MUST NOT imply": hold and resume re-INVITEs, transfer (REFER), **DTMF sending**, MESSAGE,
SUBSCRIBE/NOTIFY, PUBLISH.

Four of those the specification does not model at all. **DTMF it does** — six constructs, kept
deliberately: `softphone.media` is modelled from a Rust port where DTMF is real, and the next
binding may carry it. What that means concretely is that `softphone.control.SendDigits` has no SIP
path until the contract changes, and `domains/sip.yaml` says so where somebody implementing it will
read it.

sipx's *native* stack ships hold, resume and transfer (`README.md`, "Does it fit?"), so their absence
here is a browser-SDK constraint and not a limit of the stack.

## Not in scope for 0.1, and nothing pretends otherwise

Conference, DND, voicemail and MWI, duration, audio-device selection for a SIP call (`DeviceRef`
exists only in the loopback binding), volume, ringtone and ringback, call quality, redial, park,
presence, outbound proxy, a call limit, call recording. A service, an event log, multi-device sync.
Contact import or export. Video, data channels, TURN relay — sipx offers host and STUN-derived ICE
candidates and no relay.

## In scope and not built: two concurrent calls

sipx allows eight concurrent calls (`browser-sdk.md:281`). `softphone.presentation.Console` now holds
one active tile and four modes, which is the beginning of call focus, and there is **no held state**:
two calls can both be `Active` with both capture paths open. Local media hold and call focus are not
excluded by the contract and two concurrent calls need them. That is the next story, not a gap in
this one.
