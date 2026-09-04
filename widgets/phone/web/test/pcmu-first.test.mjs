// The one function under the page's offer, run without a browser.
//
// `pcmuFirst` is pure so this can be a plain node script: `node web/test/pcmu-first.test.mjs`
// exits non-zero on any failing case and prints the count it executed, which is what
// `tests/suite.mjs` does for the scenarios. The runner is `node:test`, which ships with node —
// the page has no package.json and this file does not change that.
//
// The two fixtures are real captures, not hand-written: an audio-only `createOffer()` from
// headless Chrome 150.0.7871.46 and headless Firefox 153.0, taken 2026-09-04 through
// `getUserMedia({audio: true})` against a fake device, and taken *before* `setLocalDescription` —
// which is exactly the string `offer.mjs` hands to `pcmuFirst`, so there is no ICE candidate in
// either. They differ where it matters: Chrome numbers Opus 111 and groups its `a=rtpmap` lines
// late; Firefox numbers Opus 109, sorts every attribute alphabetically, and writes `G722/8000/1`.
// A function tested against one of them is tested against a habit.

import assert from "node:assert/strict";
import { test } from "node:test";
import { pcmuFirst } from "../pcmu-first.mjs";

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
  "a=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level",
  "a=extmap:2 http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time",
  "a=extmap:3 http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01",
  "a=extmap:4 urn:ietf:params:rtp-hdrext:sdes:mid",
  "a=sendrecv",
  "a=msid:51901bac-29e3-4a4d-9662-fbe4ba34e7a4 75de9893-eae4-4e0d-8b86-1945be94c7cd",
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
  "a=ssrc:2418695681 cname:utKhhEjWWjzaVRGh",
  "a=ssrc:2418695681 msid:51901bac-29e3-4a4d-9662-fbe4ba34e7a4 75de9893-eae4-4e0d-8b86-1945be94c7cd",
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
  "a=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level",
  "a=extmap:2/recvonly urn:ietf:params:rtp-hdrext:csrc-audio-level",
  "a=extmap:3 urn:ietf:params:rtp-hdrext:sdes:mid",
  "a=extmap:7 http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01",
  "a=extmap-allow-mixed",
  "a=fmtp:109 maxplaybackrate=48000;stereo=1;useinbandfec=1",
  "a=fmtp:101 0-15",
  "a=ice-pwd:f3de5a638ac67074a709331c2918ac25",
  "a=ice-ufrag:ebe0533a",
  "a=mid:0",
  "a=msid:{5d7994fd-563d-4817-916a-fd385544b9a8} {7cbb231a-98a5-4c76-af37-a36a40205d23}",
  "a=rtcp-fb:109 transport-cc",
  "a=rtcp-fb:9 transport-cc",
  "a=rtcp-fb:0 transport-cc",
  "a=rtcp-fb:8 transport-cc",
  "a=rtcp-mux",
  "a=rtpmap:109 opus/48000/2",
  "a=rtpmap:9 G722/8000/1",
  "a=rtpmap:0 PCMU/8000",
  "a=rtpmap:8 PCMA/8000",
  "a=rtpmap:101 telephone-event/8000/1",
  "a=setup:actpass",
  "a=ssrc:1216609104 cname:{0b611577-b737-4961-8eaa-8305a1c2669e}",
];

/** An SDP as a browser writes it: every line CRLF-terminated, the last one included. */
const crlf = (lines) => lines.map((line) => `${line}\r\n`).join("");

/** The lines with their one `m=audio` replaced — the only difference `pcmuFirst` may make. */
const withMediaLine = (lines, mediaLine) =>
  lines.map((line) => (line.startsWith("m=audio ") ? mediaLine : line));

/** The session-level lines of a capture, to build a media section of one's own under. */
const SESSION = CHROME.slice(
  0,
  CHROME.findIndex((line) => line.startsWith("m=")),
);

// ── the reorder ─────────────────────────────────────────────────────────────

test("chrome: PCMU moves to the head of the format list and no other byte changes", () => {
  assert.equal(
    pcmuFirst(crlf(CHROME)),
    crlf(withMediaLine(CHROME, "m=audio 9 UDP/TLS/RTP/SAVPF 0 111 63 9 8 13 110 126")),
  );
});

test("firefox: PCMU moves to the head of the format list and no other byte changes", () => {
  assert.equal(
    pcmuFirst(crlf(FIREFOX)),
    crlf(withMediaLine(FIREFOX, "m=audio 9 UDP/TLS/RTP/SAVPF 0 109 9 8 101")),
  );
});

test("the rtpmap, fmtp and rtcp-fb attributes stay on the lines they were on", () => {
  // An SDP is not required to order its attributes to match the format list, and a reordered
  // attribute block is a change nobody asked for. The byte-equal cases above already imply this;
  // this one says it, by line number.
  for (const capture of [CHROME, FIREFOX]) {
    const before = crlf(capture).split("\r\n");
    const after = pcmuFirst(crlf(capture)).split("\r\n");
    assert.equal(after.length, before.length);
    for (const [index, line] of before.entries()) {
      if (
        line.startsWith("a=rtpmap:") ||
        line.startsWith("a=fmtp:") ||
        line.startsWith("a=rtcp-fb:")
      ) {
        assert.equal(after[index], line, `line ${index + 1} moved`);
      }
    }
  }
});

// ── by name, not by number ──────────────────────────────────────────────────

test("PCMU named at a payload other than 0 is refused, because the server cannot resolve it", () => {
  // This case asserted the opposite — that PCMU is whatever `a=rtpmap` calls PCMU/8000 and payload
  // 97 would be moved to the head. The adversarial pass ran the consumer,
  // `sipx_sdp::browser_audio`, against exactly that offer: `Err(CodecSetIncomplete)`.
  // `static_or_mapping(media, 0, "PCMU", 8_000)` looks for the *number*, and consults an
  // `a=rtpmap` only to check it does not disagree. So moving 97 to the head produces an offer the
  // server answers and then refuses, and the refusal belongs here instead.
  const lines = [
    ...SESSION,
    "m=audio 9 UDP/TLS/RTP/SAVPF 96 97",
    "a=rtpmap:96 opus/48000/2",
    "a=rtpmap:97 PCMU/8000",
  ];
  assert.throws(() => pcmuFirst(crlf(lines)), { message: /payload 97/ });
});

test("payload 0 with no `a=rtpmap` is PCMU, because that is what the server resolves", () => {
  // This case asserted the opposite, on the ground that the offer's own `a=rtpmap` lines are its
  // vocabulary and reading 0 as PCMU would import the static table. Measured against the consumer:
  // payload 0 with no `a=rtpmap` is `Ok`, selecting 0. Refusing it here would mean the page
  // declining an offer the server would have answered — and RFC 3551 is why: 0 is static, so an
  // `a=rtpmap` for it is optional and only ever a contradiction to check.
  const lines = [...SESSION, "m=audio 9 UDP/TLS/RTP/SAVPF 111 0", "a=rtpmap:111 opus/48000/2"];
  assert.equal(
    pcmuFirst(crlf(lines)),
    crlf(withMediaLine(lines, "m=audio 9 UDP/TLS/RTP/SAVPF 0 111")),
  );
});

// ── the head of the list is the contract ────────────────────────────────────

test("PCMU goes first even when something other than Opus was ahead of it", () => {
  // The answer copies the offer's order and the bridge reads the head. Placing PCMU merely ahead
  // of Opus would leave G722 first here, and the call would fail at the media seam anyway.
  const lines = [
    ...SESSION,
    "m=audio 9 UDP/TLS/RTP/SAVPF 9 111 0 8",
    "a=rtpmap:9 G722/8000",
    "a=rtpmap:111 opus/48000/2",
    "a=rtpmap:0 PCMU/8000",
    "a=rtpmap:8 PCMA/8000",
  ];
  assert.equal(
    pcmuFirst(crlf(lines)),
    crlf(withMediaLine(lines, "m=audio 9 UDP/TLS/RTP/SAVPF 0 9 111 8")),
  );
});

test("an offer with no Opus at all is still reordered", () => {
  const lines = [
    ...SESSION,
    "m=audio 9 UDP/TLS/RTP/SAVPF 9 0 8",
    "a=rtpmap:9 G722/8000",
    "a=rtpmap:0 PCMU/8000",
    "a=rtpmap:8 PCMA/8000",
  ];
  assert.equal(
    pcmuFirst(crlf(lines)),
    crlf(withMediaLine(lines, "m=audio 9 UDP/TLS/RTP/SAVPF 0 9 8")),
  );
});

// ── unchanged, and refused ──────────────────────────────────────────────────

test("an offer that already has PCMU first comes back byte-identical", () => {
  const already = crlf(
    withMediaLine(CHROME, "m=audio 9 UDP/TLS/RTP/SAVPF 0 111 63 9 8 13 110 126"),
  );
  assert.equal(pcmuFirst(already), already);
  // And the function's own output is its own fixed point.
  const once = pcmuFirst(crlf(FIREFOX));
  assert.equal(pcmuFirst(once), once);
});

test("an offer with no PCMU is refused, and the refusal says what is missing", () => {
  // Returning it unchanged would produce a call that fails later, at the media seam, for a reason
  // the page would then have to explain.
  // The payload, not the mapping: removing `a=rtpmap:0 PCMU/8000` and leaving 0 in the format list
  // is still an offer the server answers, so what has to be missing is the 0 itself.
  const without = withMediaLine(
    CHROME.filter((line) => line !== "a=rtpmap:0 PCMU/8000"),
    "m=audio 9 UDP/TLS/RTP/SAVPF 111 63 9 8 13 110 126",
  );
  assert.throws(() => pcmuFirst(crlf(without)), { message: /no payload 0/ });
});

// ── one audio section, exactly ──────────────────────────────────────────────
//
// The server's profile permits exactly one audio section and `offer.mjs` builds an audio-only
// connection, so anything else is the page's own bug. Refused now, with the count, rather than
// sent to a server that will refuse it later.

test("an offer with no m=audio is refused", () => {
  assert.throws(() => pcmuFirst(crlf(SESSION)), { message: /m=audio/ });
});

test("an offer whose only section is m=video is refused", () => {
  const lines = [...SESSION, "m=video 9 UDP/TLS/RTP/SAVPF 96", "a=rtpmap:96 VP8/90000"];
  assert.throws(() => pcmuFirst(crlf(lines)), { message: /video.*m=audio/ });
});

test("an offer with a second m=audio is refused", () => {
  const media = CHROME.slice(SESSION.length);
  assert.throws(() => pcmuFirst(crlf([...CHROME, ...media])), { message: /2 media sections/ });
});

test("an offer with an m=video beside the audio is refused", () => {
  const lines = [...CHROME, "m=video 9 UDP/TLS/RTP/SAVPF 96", "a=rtpmap:96 VP8/90000"];
  assert.throws(() => pcmuFirst(crlf(lines)), { message: /2 media sections/ });
});

// ── line endings ────────────────────────────────────────────────────────────

test("bare-LF line endings, and a missing final one, are kept as they are", () => {
  // Browsers write CRLF; RFC 8866 §5 says a parser should accept LF too. Whatever came in goes
  // out, because "every other byte identical" includes the terminators.
  const lf = (lines) => lines.join("\n");
  assert.equal(
    pcmuFirst(lf(FIREFOX)),
    lf(withMediaLine(FIREFOX, "m=audio 9 UDP/TLS/RTP/SAVPF 0 109 9 8 101")),
  );
});
