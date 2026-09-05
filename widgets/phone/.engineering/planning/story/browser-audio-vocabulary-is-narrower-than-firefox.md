---
format: aep.planning-md/1
id: story:browser-audio-vocabulary-is-narrower-than-firefox
kind: story
status: draft
title: The browser-audio vocabulary is narrower than Firefox's offer
relations:
- decomposes: epic:browser-softphone
- informed_by: story:browser-media-adapter
scope:
- confidence: cited
  path: web/pcmu-first.mjs
- confidence: cited
  path: web/test/adversary-pcmu-first.test.mjs
revision: 2
---
# The browser-audio vocabulary is narrower than Firefox's offer

## What was measured

`adp:adversary`, attacking the page's offer, built a probe crate against
`sipx_sdp::browser_audio` at `5c302380` — the rev `crates/phone-server` pins — and ran the page's
own captured fixtures through it:

| offer | `browser_audio::validate` |
|---|---|
| Chrome 150's, PCMU first | `Ok`, selects payload 0 |
| **Firefox 153's, PCMU first** | **`Err(CodecSetIncomplete)`** |
| Firefox's plus `a=rtpmap:13 CN/8000` | still `Err(CodecSetIncomplete)` |
| Firefox's plus CN **and** `telephone-event/8000` without `/1` | `Ok`, selects payload 0 |

Two things in Firefox's offer are outside the profile's required vocabulary: it names **no comfort
noise at payload 13**, and it writes **`telephone-event/8000/1`**, whose channel count
`mapping_matches` (`browser_audio.rs:666-674`) rejects.

So the browser-audio profile, as `sipx` implements it, is answerable by Chromium and not by Firefox.

## Why this is upstream and not ours

The page cannot fix it by munging. Adding `a=rtpmap:13 CN/8000` to an offer would advertise a codec
Firefox did not offer, and there is no measurement here that Firefox would decode it. Dropping the
`/1` from a telephone-event Firefox wrote is a change to the browser's own vocabulary statement.
Both are the page asserting something about a stack it does not own.

`widgets/phone/web/pcmu-first.mjs`'s `unanswerable()` therefore refuses a Firefox-shaped offer on
the page, with the reason, rather than letting the server answer and then decline it. Firefox is
**explicitly unsupported** today, which is a fact somebody can read, rather than a call that fails
with no explanation.

## What would close it

RFC 7874 §4.2 makes comfort noise *recommended* rather than required, and RFC 4733's telephone-event
takes an optional channel count that means the same thing present or absent. So the narrowing looks
like sipx's rather than the web platform's, and the fix is one of:

- accept a missing CN, on the ground that RFC 7874 does not require it;
- accept `telephone-event/8000/1` as equal to `telephone-event/8000`, per RFC 4733.

Either is a change to `crates/sipx-sdp/src/browser_audio.rs` in `codewandler/sipx`, with its own
vector, and it makes every consumer's browser leg work with Firefox rather than this one.

## Acceptance

A Firefox offer reordered by `pcmuFirst` is answered by the server, `unanswerable()` returns `null`
for it, and the case in `web/test/adversary-pcmu-first.test.mjs` that asserts Firefox is refused is
inverted to assert it is answered.

## Scope

Derived by the coordinator from the adversarial pass, confidence **high** — every row above was
executed against the pinned crate.

- **`codewandler/sipx`**, `crates/sipx-sdp/src/browser_audio.rs` — the change itself, out of tree
- `web/pcmu-first.mjs`, `web/test/adversary-pcmu-first.test.mjs` — cited, the refusal and its case
- **Would collide with** any unit inside `web/`.
