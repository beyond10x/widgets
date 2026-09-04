---
format: aep.planning-md/1
id: story:signalling-over-wss
kind: story
status: draft
title: Register the SIP leg, on the server that holds it
summary: 'Registration moved off the browser: phone-server drives REGISTER to a terminal state.'
relations:
- decomposes: epic:browser-softphone
- depends_on: story:media-session-domain
- informed_by: architecture-decision-record:browser-holds-no-sip-stack
revision: 7
---
# Register the SIP leg, on the server that holds it

## What changed about this story

It was "Register over secure WebSocket": the page would load the sipx WebAssembly session kernel,
open WSS to a SIP provider and drive REGISTER itself.
`architecture-decision-record:browser-holds-no-sip-stack` moved that off the browser. Registration
is still specified — `softphone.sip` is unchanged — and it is now `phone-server`'s, which
`ess/topology.yaml` states: `sip-binding` requires `network: sip-signalling` and `network: rtp-media`
rather than two browser facilities.

The kernel this story was built on is not the reason it moved, but it is why it could not have
shipped as written: `sipx` `A-17` (package the browser SDK) and `M-52` (browser-native WebRTC audio)
are both `status: ready`.

## Outcome

`phone-server` drives `softphone.sip.Register` to `Registered` or `Failed`, and back out of `Failed`
— that state stopped being terminal for exactly this reason: one transient drop must not kill a
registration permanently.

## Acceptance

Against the SIP service the deployment names, a configured address-of-record and expiry produce a
`softphone.sip.RegistrationConfirmed` fact and `softphone.sip.RegistrationById` reports `Registered`.
A rejected REGISTER produces `RegistrationFailed` and no retry loop — `RetryRegistration` is a
command somebody issues, not a loop the model implies. `RefreshRegistration` extends the lifetime
with no state change.

## Scope

`softphone.sip` only. `Registration`, `SipDialog`, the four SDP commands, and the server-side code
that drives them through `sipx-call`.

Two type members were narrowed to what a browser could do and now describe the server instead, so
each is a question this story answers rather than an assumption it keeps:

- `SignallingTransport` has one member, `SecureWebSocket`. A server can use UDP, TCP or TLS, so
  either the member set grows or the deployment is stated to be WSS-only. Decide it here.
- `MediaSecurity` has one member, `DtlsSrtp`. The browser leg keeps DTLS-SRTP; the SIP leg to a
  dev-cluster Asterisk will most likely be plain RTP or SDES-keyed SRTP, which is the same question
  with the same two answers.

## What is no longer in scope

The vendored `browser/src/` binding, the `crates/sipx-wasm` module, the hand-written ABI glue, and
the WebSocket-subprotocol token trick a browser needs because it cannot set an `Authorization`
header on an upgrade. A server sets the header.
