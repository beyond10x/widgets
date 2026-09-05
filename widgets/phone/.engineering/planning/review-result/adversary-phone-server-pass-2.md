---
format: aep.planning-md/1
id: review-result:adversary-phone-server-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 — story:phone-server: mute gates the wrong direction, and the refusal is late'
relations:
- reviews: story:phone-server
revision: 1
---
# Adversary pass 2 — `story:phone-server`

Worktree `~/.local/state/worktree/trees/b10x/widgets/wave-phone-server`, branch `impl/phone-server`
at `35f55cb` (pass 1 attacked `5115fa3`), plus one added test file,
`crates/phone-server/tests/adversary_pass2.rs`. No tracked file modified. Agent `adp:adversary`.

```
verdict: NEEDS-CHANGE
cases: executed 26→29, red 3
origin: introduced 3 / pre-existing 5 / undecided 0
```

## The largest finding in this wave: mute gates the wrong direction

`sipx_media::MediaSession::set_muted` gates a session's **outbound** audio
(`sipx-media-1.0.1/src/session.rs:2676-2692`). The browser leg's outbound is the audio going **to the
page**; the page's microphone arrives on that session's *reception*, which `set_muted` does not
touch. So `regate()`'s disjunction is applied to the render direction:

- **a page that sent `mute` is still heard by the far end at full amplitude** — 4,000 samples,
  loudest 7,932;
- **and the page stops hearing the far end** — loudest 0, against 7,932 before the mute.

`set_browser_muted`'s own doc says "what the browser hears is unaffected", and the browser leg's
outbound audio is exactly what the browser hears. Hold is unaffected, because `set_held`
additionally calls `call.mute()` and so gates both directions.

**Why three cases passed over it.** Every existing case about these flags asserts
`MediaSession::is_muted` — the value the gate was set to — rather than what is audible. An inverted
gate satisfies all of them.

The fix the pass names and does not apply: keep the SIP session (`call.media_handle()`, which
`connect` already calls) and make `regate()` set `sip.set_muted(muted || held)` and
`browser.set_muted(held)`, with `set_held` no longer touching the call directly.

## The refusal is fail-closed and late

`Phone::open` sends `softphone.bridge.ConfirmBridge` with the answer at `session.rs:181` and asks
`browser::bridgeable` at `browser.rs:392`. So `browser.rs:246`'s "before ICE, before DTLS, before
any key exists" is true of this server and false of the system: on every Opus-first offer — Chrome's
and Firefox's default — the page has installed a remote description, been told the bridge is `Live`,
and started ICE and DTLS against a `MediaPort` that has already been dropped. `close` is legal from
`Live`, so the page recovers; it is a wasted handshake, not a stuck machine. Fix: ask `bridgeable`
above the `say`.

## Five more

A bridge this server refused reaches the page as `EndCause::Refused`, whose own doc says "the far end
refused the attempt", so the page cannot tell a PBX rejecting a call from this server declining a
codec pair the far end never saw. `Bridged::is_connected` has no caller outside tests and
`fail_bridge` is reachable only from `OpenFailed`, so **`FailBridge` is never sent for a bridge that
dies after coming up** — the brief's "the bridge is gone" row is unimplemented. A control frame that
fails `deny_unknown_fields` is warned and dropped, and a second `OpenBridge` is dropped the same
way, so a page whose message the server will not parse waits for an answer no command brings. Two
`INFEASIBLE`: the setters' non-atomic `swap`/`load`/`store` (unreachable — `main::serve` awaits
`apply` one message at a time per connection), and `fresh_session_id`'s seed-plus-counter collision
across two processes sharing one advertised media address (no such deployment shown).

## Attacked and could not break

The rate gate's arithmetic: all five `Codec` variants classify as RTP does — 8 000 / 8 000 / 16 000 /
44 100 / 48 000 — so the `clock_rate`-only trap is really closed. A dynamic payload type slipping
past a codec-only gate: dead, because `browser_audio` pins PCMU to 0 and PCMA to 8, though the relay
would silently drop a dynamic one. The SIP leg's own rate cannot be wrong today
(`Codecs::G711`), but nothing pins it, so changing `SipLeg::options` would go unnoticed — a missing
guard rather than a defect. The kept ≈6× measurement cannot pass on padding: `record_at_least`
returns exactly what arrived. A second `open`, a leaked fd or an INVITE escaping the refusal: none.
All five `fail_call` classes are what the correction claims.

## Findings

```findings
- file: crates/phone-server/src/bridge.rs
  line: 127
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: regate() applies muted||held to the browser leg's outbound audio, which is the direction carrying the far end's voice to the page, so a page that sent mute is still heard by the far end at full amplitude while nothing gates the microphone crossing the bridge.
- file: crates/phone-server/src/bridge.rs
  line: 88
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: set_browser_muted's doc says "What the browser hears is unaffected — muting gates this side's outbound audio and nothing else", and the browser leg's outbound audio is exactly what the browser hears, so muting deafens the page instead of muting it.
- file: crates/phone-server/src/session.rs
  line: 181
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Phone::open sends softphone.bridge.ConfirmBridge with the SDP answer before it asks browser::bridgeable, so on every Opus-first browser offer the page is told the bridge is Live and starts ICE and DTLS against a media port this server then drops, contradicting browser.rs:246's "before ICE, before DTLS, before any key exists".
- file: crates/phone-server/src/session.rs
  line: 106
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a bridge this server refused reaches the page as EndCause::Refused, the variant wire.rs:41 documents as "the far end refused the attempt", so the page cannot tell a PBX rejecting the call from this server declining a codec pair the far end never saw.
- file: crates/phone-server/src/bridge.rs
  line: 60
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: Bridged::is_connected has no caller outside tests and fail_bridge is only reachable from OpenFailed, so softphone.bridge.FailBridge is never sent for a bridge that dies after coming up — the unit brief's "the bridge is gone" row is unimplemented.
- file: crates/phone-server/src/main.rs
  line: 121
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: a control frame that fails serde's deny_unknown_fields is warned and dropped, and session.rs:236 drops a second OpenBridge the same way, so a page whose message this server will not parse waits for an answer no command ever brings.
- file: crates/phone-server/src/bridge.rs
  line: 128
  category: concurrency
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: the swap in each setter and regate's two loads plus store are not one atomic step, so a delayed regate can publish a gate computed before the other flag moved; no caller reaches it because main::serve awaits apply one message at a time per connection.
- file: crates/phone-server/src/session.rs
  line: 272
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: fresh_session_id issues seed+n from one clock seed, so two processes advertising the same media address collide once the earlier has issued as many bridges as the nanoseconds between their starts, and a pre-epoch clock collapses the seed to 0; I could not show a deployment running two processes on one advertised address.
```
