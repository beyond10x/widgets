---
format: aep.planning-md/1
id: review-result:adversary-phone-server-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 — story:phone-server: the bridge does not resample, and mute and hold are one flag'
relations:
- reviews: story:phone-server
revision: 1
---
# Adversary pass 1 — `story:phone-server`

Worktree `~/.local/state/worktree/trees/b10x/widgets/wave-phone-server`, branch `impl/phone-server`
at `5115fa3`, plus three untracked test files the adversary added under
`crates/phone-server/tests/`. Agent `adp:adversary`.

```
verdict: NEEDS-CHANGE
cases: executed 15→22, red 6
origin: introduced 8 / pre-existing 0 / undecided 0
```

Every origin is `introduced` and that is checked rather than assumed: `eb40ddc` contains no
`crates/phone-server` at all.

## The two blockers

**The bridge does not resample, and the unit's audio proof ran on a codec pair the server never
negotiates.** 500 ms spoken on an Opus browser leg arrives at the G.711 far end as 23,040 samples —
2,880 ms. `Bridge::connect` sets `transcoding` and `spawn_leg` moves buffers with no rate
conversion (`sipx-media-1.0.1/src/bridge.rs:151-168`). The unit's own `browser_profile.rs:149` is the
proof that a browser offer negotiates Opus, and `sip.rs:193` places G.711, so this is every real
call. The three passing bridge cases used a PCMU pair.

**Mute and hold share one `AtomicBool`.** `set_held` and `set_browser_muted` swap the same flag, so
unholding clears a mute the page never lifted, and clearing mute resumes a leg that is still held.
`ess/domains/control.yaml` declares `muted` and `held` as two independent fields, and `sipx-call`'s
own `Call::mute` doc heads a section "**Mute is not hold**".

## Six more, each with what reaches it

- the crate doc at `src/lib.rs:12` enumerates `softphone.control.HangUp` among what this server
  sends, which no kernel actor may issue — **the same wrong claim the commit message says
  `tests/wire.rs` caught in the brief**, now in the prose `cargo doc` publishes;
- every bridge opens with the hard-coded SDP session id `1`, so two concurrent phones answer with
  byte-identical `o=` lines against RFC 4566 §5.2, and no existing case moves if the constant does;
- `src/browser.rs:193` claims the SDP boundary "refuses before anything is bound", and its only
  caller binds a media port and gathers ICE first. No leak — `BrowserLeg` drops on the `?` — so the
  cost is the claim;
- a refused offer, an undialable destination and a failed identity all reach the page as
  `FailCall{cause: Media}`, and `EndCause` has `Sip` and `Refused`;
- the `sipx_call::EndCause::Rejected` arm reads the variant backwards from sipx's own doc, and no
  path reaches it — `INFEASIBLE`, a dead arm;
- `tests/wire.rs:90` derives its variant set from the sample list it is checking, so a variant with
  a `command()` arm and no sample is checked by nothing. The adversary's `adversary_docs.rs:236`
  reads the `enum` declaration instead and closes it — that case is **green**, and it is the
  residual hole the implementor named and could not close.

## Attacked and not broken

No eleventh weakened offer: `sipx_sdp-1.0.1/src/browser_audio.rs::validate` was read line by line —
two audio sections, a rejected or port-0 section, `RTP/SAVP`, `a=crypto` at either level, a
non-mux `a=rtcp:`, a component-2 candidate, a relay candidate, missing `ice-options`, an
unspecified `c=`, and any offerer `a=setup` that is not `actpass` are each refused, and
`answer_offer` has one route to an `Accepted`. Duplicate `RingCall` is guarded. `EndCause` does not
collapse a clean hangup into a transport drop. No leaked socket or task: `sipx_media::Bridge` aborts
both legs on drop, a refused offer drops `BrowserLeg` through the `?`, and `SipLeg::watch` returns
on `Ended`. The audio proof's `loudest > 1_000` does separate a tone from silence and from comfort
noise.

## Findings

```findings
- file: widgets/phone/crates/phone-server/src/bridge.rs
  line: 37
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: on the only codec pair this server produces, an Opus browser leg against a G.711 SIP leg, the bridge delivers 500 ms of speech as 2880 ms of audio because the transcoding leg carries 48 kHz buffers into an 8 kHz session without resampling, and the unit's audio proof runs on a PCMU pair the server never negotiates
- file: widgets/phone/crates/phone-server/src/bridge.rs
  line: 76
  category: property
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: mute and hold share one AtomicBool on the browser leg, so unholding clears a mute the page never lifted and clearing mute resumes a leg that is still held, while the specification declares muted and held as two independent fields on softphone.control.Call
- file: widgets/phone/crates/phone-server/src/lib.rs
  line: 12
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the crate doc enumerates softphone.control.HangUp among the commands every fact leaves as, and the compiled specification grants HangUp to no kernel actor — the same wrong claim the commit message says tests/wire.rs caught in the brief
- file: widgets/phone/crates/phone-server/src/session.rs
  line: 145
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: every bridge is opened with the hard-coded SDP session id 1, so two concurrent phones answer with identical o= lines against RFC 4566 §5.2, and no existing case moves if the constant changes
- file: widgets/phone/crates/phone-server/src/browser.rs
  line: 193
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the doc claims the SDP boundary refuses before anything is bound, but the only caller binds a media port and gathers ICE before answer_offer runs
- file: widgets/phone/crates/phone-server/src/main.rs
  line: 198
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: an undialable destination and a failed DTLS identity both reach the page as FailCall with cause Media, which is neither what happened nor a class the page can act on
- file: widgets/phone/crates/phone-server/src/sip.rs
  line: 208
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the sipx_call::EndCause::Rejected arm reads the variant as the far end refusing our attempt when sipx documents it as this side refusing an invitation with no producer at this layer, and I could show no path that reaches it
- file: widgets/phone/crates/phone-server/tests/wire.rs
  line: 90
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: every_message derives its variant set from the sample list it is checking, so a ToBrowser variant with a command() arm and no sample is checked by nothing; tests/adversary_docs.rs now reads the enum declaration instead and closes it
```
