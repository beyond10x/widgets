---
format: aep.planning-md/1
id: review-result:adversary-pcmu-offer-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 — the page''s PCMU-first offer: Firefox does not bridge, and the microphone could outlive a failed offer'
relations:
- reviews: story:browser-media-adapter
revision: 1
---
# Adversary pass 1 — unit 3, the page's PCMU-first offer

Worktree `~/.local/state/worktree/trees/b10x/widgets/wave-pcmu-offer` at `91521e4`, plus two added
test files under `web/test/`. No implementation file touched. Agent `adp:adversary`.

```
verdict: NEEDS-CHANGE
cases: executed 14→18, red 4
origin: introduced 9 / pre-existing 0 / undecided 0
```

It built a probe crate in its scratch root and ran `sipx_sdp::browser_audio` — the consumer that
decides whether an offer bridges — against thirteen hand-built offers at `5c302380`, the rev
`crates/phone-server` pins. Six of those rows are now in `web/pcmu-first.mjs`'s own header.

## What the measurement settled

| finding | outcome |
|---|---|
| `new RTCPeerConnection(configuration)` outside the `try`, so a throwing configuration leaves the microphone live — the one thing the doc above it promised cannot happen | **fixed** |
| the reordered **Firefox** offer is `Err(CodecSetIncomplete)`: no CN at 13, and `telephone-event/8000/1` | refused on the page now, with the reason; upstream as `story:browser-audio-vocabulary-is-narrower-than-firefox` |
| the page found PCMU by `a=rtpmap` name while the consumer requires payload **0** | **fixed** — a PCMU at 97 is refused rather than moved to the head |
| the page refused payload 0 with no `a=rtpmap`, which the consumer resolves as PCMU | **fixed** — it is PCMU here too now |
| head-of-list was justified by a false claim: selection skips every format outside {opus, pcmu, pcma}, so `9 0 111` selects PCMU from the second slot | comment corrected; the head is still where the token goes, for being unambiguous |
| `filter` deleted every occurrence of PCMU's payload, so a list naming it twice lost a token | **fixed** — one occurrence moves |
| the Firefox fixture's provenance prose claims alphabetical attribute sorting; its own fixture contradicts it | corrected |
| the commit body's "11 of them red against an identity stub" does not add up; measured, it is **12 red, 2 green** | corrected here, since the commit is written |
| `gathered()` has no timeout, so a gathering that never completes leaves the offer pending with the microphone open | `INFEASIBLE` — no browser shown that never completes |

## Attacked and could not break

Bare LF, CRLF/LF mix, no final terminator, `m=` as the last line, a trailing blank line: byte
preserved. `PCMU/8000/1`, mixed-case `pcmu`, PCMU named at a payload absent from the format list,
two PCMU rtpmaps. Refusal ordering — section count, then kind, then PCMU. A session-level
`a=rtpmap:` before any `m=` correctly excluded from the media-level lookup. The fixed point, on the
function's own output.

## The coordinator's own note on the fix

`unanswerable()` is a second statement of the profile's codec half, written from the same source as
the adversary's transcription rather than from it, and the case that holds them together asserts
both agree. Two independently written readings of one document agreeing is the property that made
this worth doing here rather than deferring it.
