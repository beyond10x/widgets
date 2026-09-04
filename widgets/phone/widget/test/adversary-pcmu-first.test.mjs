// `pcmuFirst` against the consumer that decides whether its offer bridges, and against its own
// "no other byte changes" claim.
//
// `pcmu-first.test.mjs` asserts what the function does. Nothing asserts that what it does is an
// offer `crates/phone-server` can answer, and the whole reason this function exists is that one
// offer in three is refused at the media seam. The seam is `sipx_sdp::browser_audio`, and its
// profile is transcribed below from the source it is cited against rather than paraphrased —
// `/home/timo/projects/sipx` on `main`, `crates/sipx-sdp/src/browser_audio.rs`. Every rule here
// was also executed against that crate directly; see the adversarial pass's report.
//
// Added by the adversarial pass. It changes no existing case.

import assert from "node:assert/strict";
import { test } from "node:test";
import { pcmuFirst, unanswerable } from "../src/pcmu-first.mjs";

// ── the two captures, copied verbatim from `pcmu-first.test.mjs` ─────────────
//
// Copied rather than imported because that file exports nothing, and an adversarial pass does not
// edit the file it is attacking. If those fixtures change, these must be recopied.

const CHROME = [
  "v=0",
  "o=- 7912598068799701116 2 IN IP4 127.0.0.1",
  "s=-",
  "t=0 0",
  "a=group:BUNDLE 0",
  "a=extmap-allow-mixed",
  "a=msid-semantic: WMS 51901bac-29e3-4a4d-9662-fbe4ba34e7a4",
  "m=audio 9 UDP/TLS/RTP/SAVPF 111 63 9 0 8 13 110 126",
  "c=IN IP4 0.0.0.0",
  "a=rtcp:9 IN IP4 0.0.0.0",
  "a=ice-ufrag:Sj/d",
  "a=ice-pwd:UnLjTut8CfbIhJlaeHYOKbhC",
  "a=ice-options:trickle",
  "a=fingerprint:sha-256 60:04:1A:F9:DF:F1:E5:A7:7B:65:8A:21:3D:6F:D6:14:4C:C0:FE:DB:D0:37:9A:27:90:D8:17:F2:50:F1:A6:17",
  "a=setup:actpass",
  "a=mid:0",
  "a=sendrecv",
  "a=rtcp-mux",
  "a=rtcp-rsize",
  "a=rtpmap:111 opus/48000/2",
  "a=rtcp-fb:111 transport-cc",
  "a=fmtp:111 minptime=10;useinbandfec=1",
  "a=rtpmap:63 red/48000/2",
  "a=fmtp:63 111/111",
  "a=rtpmap:9 G722/8000",
  "a=rtpmap:0 PCMU/8000",
  "a=rtpmap:8 PCMA/8000",
  "a=rtpmap:13 CN/8000",
  "a=rtpmap:110 telephone-event/48000",
  "a=rtpmap:126 telephone-event/8000",
];

const FIREFOX = [
  "v=0",
  "o=mozilla...THIS_IS_SDPARTA-99.0 4663967860738290254 0 IN IP4 0.0.0.0",
  "s=-",
  "t=0 0",
  "a=fingerprint:sha-256 29:4F:4A:07:BC:10:0C:C9:DC:2F:8D:0E:B0:EB:8D:B5:8A:C6:78:21:74:3B:38:93:B1:D4:D8:07:F1:8C:AE:E1",
  "a=group:BUNDLE 0",
  "a=ice-options:trickle",
  "a=msid-semantic:WMS *",
  "m=audio 9 UDP/TLS/RTP/SAVPF 109 9 0 8 101",
  "c=IN IP4 0.0.0.0",
  "a=sendrecv",
  "a=fmtp:109 maxplaybackrate=48000;stereo=1;useinbandfec=1",
  "a=fmtp:101 0-15",
  "a=ice-pwd:f3de5a638ac67074a709331c2918ac25",
  "a=ice-ufrag:ebe0533a",
  "a=mid:0",
  "a=rtcp-fb:109 transport-cc",
  "a=rtcp-mux",
  "a=rtpmap:109 opus/48000/2",
  "a=rtpmap:9 G722/8000/1",
  "a=rtpmap:0 PCMU/8000",
  "a=rtpmap:8 PCMA/8000",
  "a=rtpmap:101 telephone-event/8000/1",
  "a=setup:actpass",
];

const crlf = (lines) => lines.map((line) => `${line}\r\n`).join("");
const SESSION = CHROME.slice(
  0,
  CHROME.findIndex((line) => line.startsWith("m=")),
);

// ── the server's profile, transcribed ───────────────────────────────────────

/** The one `m=audio` line's fields, and the media-level `a=` lines under it. */
function section(sdp) {
  const lines = sdp.split(/\r?\n/);
  const at = lines.findIndex((line) => line.startsWith("m="));
  const [, , , ...formats] = lines[at].split(" ");
  return { formats, attributes: lines.slice(at + 1) };
}

/**
 * The `a=rtpmap` value for one payload, or `undefined` when the offer names none.
 *
 * `mapping_matches` (`browser_audio.rs:695-703`) splits on `/` and requires the channel field to
 * be **absent** when the profile asks for no channel count, so `PCMU/8000/1` is not `PCMU/8000`.
 */
function rtpmap({ attributes }, payload) {
  const line = attributes.find((value) => value.startsWith(`a=rtpmap:${payload} `));
  return line?.slice(`a=rtpmap:${payload} `.length);
}

const mappingMatches = (mapping, encoding, clock, channels) =>
  mapping !== undefined &&
  mapping.toLowerCase() === [encoding.toLowerCase(), clock, ...(channels ? [channels] : [])].join("/");

/**
 * The payload the server resolves as PCMU, or `null` when it refuses the offer's PCMU.
 *
 * `static_or_mapping(media, 0, "PCMU", 8_000)` — `browser_audio.rs:611-621`. PCMU is payload 0 and
 * only payload 0: the number must be in the format list, and its `a=rtpmap`, **if it has one at
 * all**, must read exactly `PCMU/8000`. A payload 0 with no `a=rtpmap` is PCMU to the server, and
 * a `PCMU/8000` named at any other number is not PCMU to it.
 */
function serverPcmu(sdp) {
  const media = section(sdp);
  if (!media.formats.includes("0")) return null;
  const mapping = rtpmap(media, "0");
  if (mapping === undefined) return "0";
  return mappingMatches(mapping, "PCMU", 8000) ? "0" : null;
}

/**
 * Why `sipx_sdp::browser_audio::validate` would refuse this offer, or `null` if it would not.
 *
 * `payloads()` — `browser_audio.rs:591-633` — is a fixed required vocabulary, and every element of
 * it is a refusal on its own: Opus at 48 kHz stereo, PCMU at 0, PCMA at 8, comfort noise at 13
 * with an `a=rtpmap` of its own, and RFC 4733 telephone events at 8 kHz with no channel count.
 * Only the codec half is transcribed here; ICE, DTLS, `rtcp-mux` and the rest of the profile are
 * not this unit's business and are not asserted.
 */
function profileRefusal(sdp) {
  const media = section(sdp);
  const single = (encoding, clock, channels) => {
    const found = media.formats.filter((payload) =>
      mappingMatches(rtpmap(media, payload), encoding, clock, channels),
    );
    return found.length === 1 ? found[0] : null;
  };
  const opus = single("opus", 48000, 2);
  const pcmu = serverPcmu(sdp);
  const pcma = media.formats.includes("8") && mappingMatches(rtpmap(media, "8"), "PCMA", 8000) ? "8" : null;
  const cn = media.formats.includes("13") && mappingMatches(rtpmap(media, "13"), "CN", 8000) ? "13" : null;
  const telephoneEvent = single("telephone-event", 8000, undefined);
  const missing = Object.entries({ opus, pcmu, pcma, cn, telephoneEvent })
    .flatMap(([name, payload]) => (payload === null ? [name] : []));
  if (missing.length > 0) return `CodecSetIncomplete: no ${missing.join(", no ")}`;
  if (media.formats.length < 5) return "CodecSetIncomplete: fewer than five formats";
  if (new Set(media.formats).size !== media.formats.length) {
    return "CodecSetIncomplete: a payload number listed twice";
  }
  return null;
}

/** The head of the reordered format list. */
const head = (sdp) => section(sdp).formats[0];

// ── the acceptance statement, as the commit words it ────────────────────────

test("the page refuses exactly the offers the server's profile would refuse", () => {
  // This case asserted that both captures survive the profile, and Firefox's does not — no CN, and
  // `telephone-event/8000/1` — which is the finding that made the split below exist. The claim it
  // was making is still the right one, and it is asked of the page's whole guarantee rather than of
  // the reorder alone: `pcmuFirst` decides which format the server selects, `unanswerable` decides
  // whether the section is one it answers at all, and `offer.mjs` asks both before the offer leaves.
  //
  // So Chrome bridges and Firefox is refused **here**, on the page, with the reason — instead of
  // after a handshake the server abandons. `story:browser-audio-vocabulary-is-narrower-than-firefox`
  // is the way out of that, and it is upstream: the vocabulary is sipx's.
  assert.equal(unanswerable(pcmuFirst(crlf(CHROME))), null, "chrome's offer is refused");
  assert.equal(profileRefusal(pcmuFirst(crlf(CHROME))), null, "and the transcribed profile agrees");

  const firefox = unanswerable(pcmuFirst(crlf(FIREFOX)));
  assert.match(firefox, /CN at payload 13/, "firefox is refused, and for the reason the profile gives");
  assert.ok(profileRefusal(pcmuFirst(crlf(FIREFOX))), "which the transcribed profile also refuses");
});

// ── the head has to be a payload the server calls PCMU ──────────────────────

test("the payload pcmuFirst puts first is the payload the server resolves as PCMU", () => {
  // The function is documented to find PCMU by its `a=rtpmap` name and not by payload type,
  // "because a function that hard-codes it is wrong for a reason the next reader cannot see"
  // (`pcmu-first.mjs:27-30`). The consumer hard-codes it: `static_or_mapping(media, 0, "PCMU", …)`
  // requires the number 0, and requires the mapping to carry no channel count. So an offer whose
  // PCMU is not payload 0, or whose PCMU mapping is `PCMU/8000/1`, is one the server refuses —
  // and putting it at the head of the list does not make it answerable. Either refusal is a fine
  // answer here; producing a head the server will not read as PCMU is not.
  const offers = {
    "PCMU named only at payload 97": [
      ...SESSION,
      "m=audio 9 UDP/TLS/RTP/SAVPF 111 97 8 13 126",
      "a=rtpmap:111 opus/48000/2",
      "a=rtpmap:97 PCMU/8000",
      "a=rtpmap:8 PCMA/8000",
      "a=rtpmap:13 CN/8000",
      "a=rtpmap:126 telephone-event/8000",
    ],
    "PCMU written with a channel count": [
      ...SESSION,
      "m=audio 9 UDP/TLS/RTP/SAVPF 111 0 8 13 126",
      "a=rtpmap:111 opus/48000/2",
      "a=rtpmap:0 PCMU/8000/1",
      "a=rtpmap:8 PCMA/8000",
      "a=rtpmap:13 CN/8000",
      "a=rtpmap:126 telephone-event/8000",
    ],
  };
  for (const [why, lines] of Object.entries(offers)) {
    const sdp = crlf(lines);
    let returned;
    try {
      returned = pcmuFirst(sdp);
    } catch {
      continue; // A refusal is one of the two acceptable outcomes.
    }
    assert.equal(
      head(returned),
      serverPcmu(sdp),
      `${why}: the head of the returned list is not what the server reads as PCMU`,
    );
  }
});

// ── "changes no other byte" ─────────────────────────────────────────────────

test("a format list whose payload appears twice keeps both of them", () => {
  // The claim is that the format list is reordered and no other byte changes. `formats.filter`
  // (`pcmu-first.mjs:79`) removes *every* occurrence of PCMU's number and then prepends one, so a
  // list that named it twice comes back one element shorter — a deletion, not a reorder. Whether
  // a duplicate is legal SDP is beside the point: the function's contract is that it moves a
  // token, and it drops one.
  const lines = [
    ...SESSION,
    "m=audio 9 UDP/TLS/RTP/SAVPF 111 0 8 0",
    "a=rtpmap:111 opus/48000/2",
    "a=rtpmap:0 PCMU/8000",
    "a=rtpmap:8 PCMA/8000",
  ];
  const before = section(crlf(lines)).formats;
  const after = section(pcmuFirst(crlf(lines))).formats;
  assert.deepEqual([...after].sort(), [...before].sort(), "the format list lost a payload number");
});
