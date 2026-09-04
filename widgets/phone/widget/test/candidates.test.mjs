// The candidate half of the offer, and the defect that made every real call fail.
//
// The fixture is one capture, taken on 2026-09-05 from Chrome 150 on a machine with a WireGuard
// interface, four Docker bridges and a LAN address: `widget/src/dev.ts` dialled, and the offer was
// read off the control channel before `phone-server` saw it. Twelve candidates — six UDP and six
// ICE-TCP — and `phone-server` refused it with
//
//   the browser-audio profile requires a complete usable ICE generation
//
// three times running, which is what sent anyone looking. It is a capture and not a construction,
// so it may be recopied but not tidied.

import assert from "node:assert/strict";
import { test } from "node:test";
import { pcmuFirst, uncheckable, udpCandidatesOnly } from "../src/pcmu-first.mjs";

const GATHERED = [
  "v=0",
  "o=- 5298306660555186521 2 IN IP4 127.0.0.1",
  "s=-",
  "t=0 0",
  "a=group:BUNDLE 0",
  "a=extmap-allow-mixed",
  "a=msid-semantic: WMS 705badad-84f4-48db-af44-7b144eca7904",
  "m=audio 44228 UDP/TLS/RTP/SAVPF 0 111 63 9 8 13 110 126",
  "c=IN IP4 172.20.0.3",
  "a=rtcp:9 IN IP4 0.0.0.0",
  "a=candidate:1582201386 1 udp 2122260223 172.20.0.3 44228 typ host generation 0 network-id 6 network-cost 50",
  "a=candidate:802305441 1 udp 2122194687 172.18.0.1 55039 typ host generation 0 network-id 1",
  "a=candidate:2260749112 1 udp 2122129151 172.21.0.1 57606 typ host generation 0 network-id 2",
  "a=candidate:2149039948 1 udp 2122063615 172.25.0.1 46086 typ host generation 0 network-id 3",
  "a=candidate:645148186 1 udp 2121998079 172.17.0.1 53996 typ host generation 0 network-id 4",
  "a=candidate:2886965191 1 udp 2121932543 192.168.68.50 42369 typ host generation 0 network-id 5",
  "a=candidate:2699341502 1 tcp 1518280447 172.20.0.3 9 typ host tcptype active generation 0 network-id 6 network-cost 50",
  "a=candidate:3514360117 1 tcp 1518214911 172.18.0.1 9 typ host tcptype active generation 0 network-id 1",
  "a=candidate:2020253612 1 tcp 1518149375 172.21.0.1 9 typ host tcptype active generation 0 network-id 2",
  "a=candidate:2126343128 1 tcp 1518083839 172.25.0.1 9 typ host tcptype active generation 0 network-id 3",
  "a=candidate:3638491790 1 tcp 1518018303 172.17.0.1 9 typ host tcptype active generation 0 network-id 4",
  "a=candidate:1387873107 1 tcp 1517952767 192.168.68.50 9 typ host tcptype active generation 0 network-id 5",
  "a=ice-ufrag:+H21",
  "a=ice-pwd:43rspUJkvFAmiSYT3/AxhEG6",
  "a=ice-options:trickle",
  "a=fingerprint:sha-256 1E:4A:1C:4D:1A:E6:70:CD:29:D1:FC:44:BC:32:B8:69:A8:F6:AB:05:89:47:0F:A2:26:11:DE:06:0E:0F:3F:D0",
  "a=setup:actpass",
  "a=mid:0",
  "a=sendrecv",
  "a=rtcp-mux",
  "a=rtcp-rsize",
  "a=rtpmap:0 PCMU/8000",
  "a=rtpmap:111 opus/48000/2",
  "a=rtcp-fb:111 transport-cc",
  "a=fmtp:111 minptime=10;useinbandfec=1",
  "a=rtpmap:63 red/48000/2",
  "a=fmtp:63 111/111",
  "a=rtpmap:9 G722/8000",
  "a=rtpmap:8 PCMA/8000",
  "a=rtpmap:13 CN/8000",
  "a=rtpmap:110 telephone-event/48000",
  "a=rtpmap:126 telephone-event/8000",
];

const crlf = (lines) => lines.map((line) => `${line}\r\n`).join("");
const candidates = (sdp) =>
  sdp.split(/\r?\n/).filter((line) => line.startsWith("a=candidate:"));
const replace = (lines, predicate, replacement) =>
  lines.flatMap((line) => (predicate(line) ? replacement : [line]));

test("what a browser really offers is refused, and for its candidates", () => {
  // The whole reason this file exists. The vocabulary is fine — Chrome names all five formats and
  // `pcmuFirst` has nothing to move, since payload 0 is already at the head of this capture.
  const refusal = uncheckable(crlf(GATHERED));
  assert.match(refusal, /6 candidate\(s\) over tcp/, "the six ICE-TCP lines are the reason");

  // And it is fatal upstream rather than ignorable, which is the part worth pinning: one
  // unparseable candidate refuses the description, because `profile_candidates` collects into
  // `Option<Vec<_>>`. sipx's own `ice::Transport` says the opposite should happen — "a peer
  // offering an ICE-TCP candidate alongside UDP ones is offering something usable" — so this
  // assertion is a statement about today's code and is expected to change when that is fixed.
  // `story:the-profile-refuses-an-offer-carrying-an-ice-tcp-candidate`.
  assert.equal(candidates(crlf(GATHERED)).length, 12, "twelve, as captured");
});

test("filtered, the same offer is one the server can check", () => {
  const filtered = udpCandidatesOnly(pcmuFirst(crlf(GATHERED)));
  assert.equal(uncheckable(filtered), null, "nothing left for the profile to refuse");
  assert.equal(candidates(filtered).length, 6, "the six UDP candidates survive");
  assert.ok(
    candidates(filtered).every((line) => line.includes(" udp ")),
    "and only those",
  );
});

test("the filter touches the candidate lines and nothing else", () => {
  const before = crlf(GATHERED);
  const after = udpCandidatesOnly(before);
  const other = (sdp) =>
    sdp.split(/(?<=\n)/).filter((line) => !line.startsWith("a=candidate:"));
  assert.deepEqual(other(after), other(before), "every other line is carried verbatim");
  // Including the terminators: this is what makes the result an SDP the browser and the server
  // read the same way, and `pcmu-first.mjs` makes the same promise about the reorder.
  assert.ok(after.endsWith("\r\n"), "the last line keeps its terminator");
});

test("an offer with no UDP candidate is refused rather than sent empty", () => {
  const tcpOnly = replace(
    GATHERED,
    (line) => line.startsWith("a=candidate:") && line.includes(" udp "),
    [],
  );
  assert.throws(
    () => udpCandidatesOnly(crlf(tcpOnly)),
    /no UDP candidate/,
    "stripping everything would leave an offer with no address at all",
  );
});

test("a candidate kind the profile does not take is refused", () => {
  // `profile_candidates` takes `Host` and `ServerReflexive` and no others, so a relayed candidate
  // — which a page configured with a TURN server would gather — is refused. This phone configures
  // none, and the case is here so that adding one is a decision somebody takes deliberately.
  const relayed = replace(
    GATHERED,
    (line) => line.startsWith("a=candidate:1582201386"),
    [
      "a=candidate:1582201386 1 udp 2122260223 172.20.0.3 44228 typ relay raddr 0.0.0.0 rport 0 generation 0",
    ],
  );
  const refusal = uncheckable(udpCandidatesOnly(crlf(relayed)));
  assert.match(refusal, /relay candidate\(s\)/, "and it says which kind");
});

test("more candidates than the profile counts is refused", () => {
  const many = [
    ...GATHERED,
    ...Array.from(
      { length: 30 },
      (_, n) => `a=candidate:${900000 + n} 1 udp ${2122260000 - n} 10.0.0.${n} ${40000 + n} typ host generation 0`,
    ),
  ];
  const refusal = uncheckable(udpCandidatesOnly(crlf(many)));
  assert.match(refusal, /36 candidates and the profile takes at most 32/);
});

test("a pre-gathering offer has no candidates, and that is not a defect in it", () => {
  // Why this is a separate question from `unanswerable`. `createOffer` returns a description with
  // `a=ice-options:trickle` and no candidate lines at all; the vocabulary in it is already
  // decidable and the ICE is not. One function asked of both would call every fresh offer broken.
  const pregathering = GATHERED.filter((line) => !line.startsWith("a=candidate:"));
  assert.match(uncheckable(crlf(pregathering)), /no ICE candidate/);
});
