---
format: aep.planning-md/1
id: story:the-profile-refuses-an-offer-carrying-an-ice-tcp-candidate
kind: story
status: draft
title: The browser-audio profile refuses an offer carrying an ICE-TCP candidate
relations:
- decomposes: epic:browser-softphone
- informed_by: story:browser-media-adapter
revision: 1
---
# The browser-audio profile refuses an offer carrying an ICE-TCP candidate

## Outcome

An offer that names ICE-TCP candidates beside its UDP ones is answered, using the UDP ones. Today
it is refused whole, which means **no offer any current browser produces can be answered without
the page rewriting it first.**

## Where it is

`crates/sipx-sdp/src/browser_audio.rs`, `profile_candidates`:

```rust
let candidates: Vec<Candidate> = component_one
    .into_iter()
    .map(Candidate::parse)
    .collect::<Option<_>>()
    .ok_or(ProfileError::IceRequired)?;
```

`collect::<Option<Vec<_>>>()` is `None` if any single element is `None`, so one unparseable
candidate refuses the description. `Candidate::parse` answers `None` for a transport other than
UDP — `Transport::parse` has one variant.

## The specification for the fix is sipx's own doc

`ice::Transport`, in the same crate:

> One variant, deliberately. RFC 8839 §5.1's grammar is `"UDP" / transport-extension`, and sipx
> checks candidates over UDP only — so a candidate naming anything else parses as far as this type
> and is then **dropped** by `Candidate::parse`, rather than failing the description. A peer
> offering an ICE-TCP candidate alongside UDP ones is offering something usable.

That is the intended behaviour, stated where the type is defined, and `profile_candidates` does not
implement it. `filter_map` in place of `map(...).collect::<Option<_>>()`, then the existing
emptiness check, is the whole change.

## How it was found

Measured 2026-09-05. A Chrome 150 offer, captured off the control channel before `phone-server` saw
it, carried 12 candidates on a machine with a WireGuard interface, four Docker bridges and a LAN
address: 6 `udp` and 6 `tcp ... typ host tcptype active`. `phone-server` refused it three times
with

```
the browser-audio profile requires a complete usable ICE generation
```

The same offer with the six TCP lines removed was answered, and the page recorded
`softphone.bridge.BridgeConfirmed`.

Every browser does this by default, so the refusal is not an edge case: it is every call.

## Worked around, not waiting on

`widget/src/pcmu-first.mjs`'s `udpCandidatesOnly` strips them on the way out, and
`widget/test/candidates.test.mjs` pins both halves — the raw capture refused, the filtered one
accepted. The workaround is independently correct, since a server that checks over UDP only has no
use for an ICE-TCP candidate, so it stays after this is fixed. What changes is that a page which
does *not* strip them stops being broken.

## Acceptance

`profile_candidates` answers `Ok` for a description carrying both, with the TCP candidates absent
from the result, and `Err(IceRequired)` only when nothing usable is left. A case over the capture
in `widget/test/candidates.test.mjs` — a real browser's offer, not a constructed one.
