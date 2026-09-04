---
format: aep.planning-md/1
id: story:signalling-over-wss
kind: story
status: draft
title: Register over secure WebSocket
summary: Load the WASM kernel, open WSS, and drive REGISTER to a terminal state.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:media-session-domain
revision: 5
---
# Register over secure WebSocket

## Outcome

The page loads the sipx WebAssembly session kernel, opens a WSS connection to the SIP provider, and
drives `softphone.sip.Register` through to `Registered` or `Failed` — and back out of `Failed`, which
it could not do before: that state was terminal, so one transient WSS drop killed the registration
permanently.

## Acceptance

Loading the page with a configured address-of-record, a WSS URL and an expiry produces a
`softphone.sip.RegistrationConfirmed` fact, and `softphone.sip.RegistrationById` reports
`Registered`. A rejected REGISTER produces `RegistrationFailed` and no retry loop —
`RetryRegistration` is a command somebody issues, not a loop the model implies. `RefreshRegistration`
extends the lifetime without a state change.

## Scope

This story is now scoped to `softphone.sip` — `ess/domains/sip.yaml` — and nothing else. Registration
and the dialog live there; the call lifecycle moved to `softphone.control` and the media session to
`softphone.media`.

- `Registration` — `aor`, `transport`, four states, four commands. `SignallingTransport` has one
  member, `SecureWebSocket`, because a browser can open no other and the page is served over HTTPS.
- `SipDialog` — `session_id` typed `softphone.media.MediaSessionId`, which is the carrier of bridge 2.
  `OpenDialog` is the only outcome that creates one, and that is what holds the binding
  discriminator on `MediaSession` honest, since no invariant can.
- `MediaSecurity` has one member, `DtlsSrtp`: the browser negotiates media and that is the only
  keying `RTCPeerConnection` offers.
- `SipCause` stays text. Inventing a closed set of SIP causes here would be inventing the protocol's
  vocabulary; the call log records `softphone.media.TerminationReason` instead.

Implementation, unchanged from before: the vendored `browser/src/` binding from the pinned sipx
revision, the WASM module from `crates/sipx-wasm` and `wasm/`, and the hand-written ABI glue that
`A-17` has not generated.

## Cited from

- `browser/README.md` — the binding's four platform facilities and its constructor-injected socket,
  clock, entropy and connectivity monitor.
- `docs/specs/browser-signalling.md` — the contract the binding implements.
- `docs/specs/browser-sdk.md` §4 — the kernel ABI.

## Authentication

A browser cannot set an `Authorization` header on a WebSocket upgrade. The access token rides the
negotiated WebSocket subprotocol list, the way
`~/babelforce/projects/ai-agent-platform/docs/designs/browser-voice.md` §4 records for the console's
RTVBP phone. Whether the SIP provider's WSS endpoint accepts a token that way, or wants SIP digest
authentication in the REGISTER instead, is not established here and is the first thing to check
against the chosen provider.

Origin allowlisting and a TLS endpoint reachable from the browser are the deployment's, not this
story's.
