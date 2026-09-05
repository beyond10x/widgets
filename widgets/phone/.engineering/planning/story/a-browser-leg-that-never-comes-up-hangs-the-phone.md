---
format: aep.planning-md/1
id: story:a-browser-leg-that-never-comes-up-hangs-the-phone
kind: story
status: draft
title: A browser leg that never comes up hangs the phone, and nobody is told
relations:
- decomposes: epic:browser-softphone
- informed_by: story:phone-server
- blocks: story:browser-media-adapter
revision: 1
---
# A browser leg that never comes up hangs the phone, and nobody is told

## Outcome

A bridge that cannot be brought up ends, within a bounded time, as one
`softphone.bridge.FailBridge` and one `softphone.control.FailCall`. Today it ends as nothing at all:
the page's bridge stays `Live`, its call stays `Requested`, the server logs nothing, and the media
sockets stay bound.

## What was measured

2026-09-05, driving headless Chrome 150 (fake capture device) at `phone-server` with `sipx answer`
as the far end, both on one machine.

The page gets as far as the answer and stops there. Its event log, read out of the panel:

```
control.EndpointConfigured | media.SessionOpened | bridge.BridgeConnecting |
control.CallDialled | presentation.CallShown | bridge.BridgeConfirmed | media.SessionActivated
```

So `answer_offer`, `codec()` and `bridgeable` all passed, `say(ToBrowser::ConfirmBridge)` went out,
the page applied the answer and the binding activated the session. Then, in 35 seconds of watching:

- no INVITE at the far end — `sipx answer` stayed `listening` and timed out with `no call arrived`;
- nothing in `phone-server`'s log at `RUST_LOG=debug`, neither a success nor a failure;
- three media sockets still bound on the server (`ss -lun` showed `:53077`, `:55390`, `:55391`),
  one per stuck attempt.

`Phone::open` is therefore still awaiting inside `leg.start(...)`, which is
`crates/phone-server/src/session.rs`. Everything after it — the SIP call, the bridge, the fact
channel — never happens, and `main.rs` logs only on `Err`, so a hang is silent by construction.

## The part that makes it a defect rather than a limitation

`browser.rs:387-417` already passes a budget:

```rust
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
```

> A budget rather than a retry count: a page whose handshake does not complete is a page that will
> not hear anything, and waiting longer only delays telling it so.

The await outlived it by more than 3×, so either `start_browser_audio` does not honour the timeout
it is given, or the wait is happening before the handshake — ICE nomination, which that budget does
not cover. Which of the two has **not** been established, and the fix differs:

- if the timeout is not honoured, it is upstream in `sipx-media`;
- if it is ICE nomination, `phone-server` needs its own bound around the whole bring-up, and
  `ProfileError::NoNominatedPair` already exists as the word for the outcome.

## What this story is not

It is not "the DTLS handshake with Chrome fails". Whether it would ever complete here is unknown
**because** the hang prevents finding out — no error, no log, no timeout. Establishing that is the
first step of this story, not its premise.

## First step

Bound the whole of `Phone::open`'s browser bring-up in `phone-server` and report the timeout as
`FailBridge` + `FailCall`. That turns an invisible hang into a stated failure, which is worth having
whichever of the two causes it is, and it is what makes the cause observable.

## Acceptance

A browser leg that does not come up produces both commands at the page within a stated bound, the
media sockets are released, and a case drives it — a peer that completes ICE and never finishes
DTLS is one `tests/` can write, since the browser leg's far side is a socket.
