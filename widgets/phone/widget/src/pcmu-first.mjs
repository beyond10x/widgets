// PCMU first in an offer's `m=audio` format list, and a refusal for an offer the server cannot
// answer at all.
//
// The server bridges one WebRTC leg to one SIP leg, and its bridge moves buffers between the two
// with no rate conversion — an Opus leg at 48 kHz against a G.711 leg at 8 kHz delivered 500 ms
// of speech as 2,880 ms — so it refuses any pair whose rates differ, at the media seam, before
// ICE and before the SIP call is placed. Which rate the browser leg has is decided by the offer:
// an answer that reorders the offer's formats is `Err(CodecSetIncomplete)`, so the answerer has
// no say.
//
// **Everything below is what `sipx_sdp::browser_audio` does, executed against it rather than read
// off it.** The adversarial pass ran thirteen hand-built offers through that crate at the rev
// `crates/phone-server` pins, `5c302380`, and three of its results are why this file is shaped as
// it is:
//
// | offer | `browser_audio` |
// | --- | --- |
// | PCMU first, Chrome's vocabulary | `Ok`, selects payload 0 |
// | PCMU at payload 97 | **`Err(CodecSetIncomplete)`** |
// | payload 0 with no `a=rtpmap` | `Ok`, selects payload 0 |
// | `PCMU/8000/1` — a channel count | **`Err(CodecSetIncomplete)`** |
// | Firefox's vocabulary, PCMU first | **`Err(CodecSetIncomplete)`** — no CN, and `telephone-event/8000/1` |
// | `9 0 111` — G722 first, PCMU second | `Ok`, selects payload 0 |
//
// Two consequences, and each corrects something an earlier version of this file asserted.
//
// **PCMU is payload 0 and nothing else.** `static_or_mapping(media, 0, "PCMU", 8_000)` looks for
// the number, and an `a=rtpmap` is consulted only to check it does not *disagree*. So an offer
// naming PCMU at 97 cannot be made to bridge by putting 97 at the head, and payload 0 with no
// `a=rtpmap` is PCMU — the opposite of what this file said when it looked PCMU up by name.
//
// **Head-of-list is not what selection depends on.** The selection skips every format outside
// {opus, pcmu, pcma}, so `9 0 111` selects PCMU from the second slot. The head is still where
// this function puts it, because it is unambiguous and it is what the measured Chrome path did —
// but the earlier claim that a G722-first offer would otherwise fail at the same seam was false.

/** What `browser_audio` requires of an audio section beyond PCMU itself. */
const REQUIRED = [
  { name: "CN", clock: "8000", why: "comfort noise" },
  { name: "telephone-event", clock: "8000", why: "RFC 4733 digits" },
];

/** Every `a=rtpmap` of the section starting at `at`, parsed. */
function vocabulary(lines, at) {
  return lines.slice(at + 1).flatMap((line) => {
    const named = line.match(/^a=rtpmap:(\d+) ([^/\r\n]+)\/(\d+)(\/(\d+))?/);
    return named
      ? [{ payload: named[1], name: named[2], clock: named[3], channels: named[5] }]
      : [];
  });
}

/** The one `m=` line's index, refusing anything that is not exactly one audio section. */
function audioSection(lines) {
  const media = lines.flatMap((line, index) => (line.startsWith("m=") ? [index] : []));
  if (media.length === 0) {
    throw new Error("the offer has no m=audio section, and the page's offer is exactly one");
  }
  if (media.length > 1) {
    const kinds = media.map((index) => lines[index].split(" ")[0].slice(2));
    throw new Error(
      `the offer has ${media.length} media sections (${kinds.join(", ")}); ` +
        "the page's offer is exactly one, m=audio",
    );
  }
  const [at] = media;
  if (!lines[at].startsWith("m=audio ")) {
    const kind = lines[at].split(" ")[0].slice(2);
    throw new Error(
      `the offer's one media section is ${kind}; the page's offer is m=audio`,
    );
  }
  return at;
}

/**
 * Why the server's audio profile would decline this offer, or `null` if it would not.
 *
 * A second question from the reorder, and asked separately because it is about vocabulary rather
 * than order: `pcmuFirst` decides which format the server selects, and this decides whether the
 * section is one it will answer at all. `offer.mjs` asks both, in that order, before the offer
 * leaves the page — so a page hears the reason here instead of after a handshake the server then
 * abandons.
 *
 * Every rule is `sipx_sdp::browser_audio`'s, executed against it. **Firefox's offer fails this
 * one**: it names no CN and writes `telephone-event/8000/1`. See
 * `story:browser-audio-vocabulary-is-narrower-than-firefox`.
 */
export function unanswerable(sdp) {
  const lines = sdp.split(/(?<=\n)/);
  const at = audioSection(lines);
  const terminator = lines[at].match(/\r?\n$/)?.[0] ?? "";
  const formats = lines[at].slice(0, -terminator.length || undefined).split(" ").slice(3);
  const mappings = vocabulary(lines, at);
  const at_payload = (payload) => mappings.find((entry) => entry.payload === payload);
  const matches = (entry, name, clock, channels) =>
    entry !== undefined &&
    entry.name.toLowerCase() === name.toLowerCase() &&
    entry.clock === String(clock) &&
    entry.channels === channels;

  // `payloads()` at `browser_audio.rs:591-633`, one line per element of the required vocabulary.
  // Payload 0's own mapping is `pcmuFirst`'s to refuse, because a head the server cannot resolve is
  // not PCMU first; what is left here is the rest.
  const required = {
    "opus/48000/2": formats.filter((payload) =>
      matches(at_payload(payload), "opus", 48000, "2"),
    ).length === 1,
    "PCMU at payload 0": formats.includes("0"),
    "PCMA at payload 8": formats.includes("8") && matches(at_payload("8"), "PCMA", 8000, undefined),
    "CN at payload 13": formats.includes("13") && matches(at_payload("13"), "CN", 8000, undefined),
    "telephone-event/8000 with no channel count": formats.filter((payload) =>
      matches(at_payload(payload), "telephone-event", 8000, undefined),
    ).length === 1,
  };
  const missing = Object.entries(required).flatMap(([what, held]) => (held ? [] : [what]));
  if (missing.length > 0) {
    return `the server's audio profile requires, and this offer has no ${missing.join(", no ")}`;
  }
  if (formats.length < 5) {
    return `the offer names ${formats.length} formats and the profile requires all five`;
  }
  if (new Set(formats).size !== formats.length) {
    return "the offer lists a payload number twice, which the profile refuses";
  }
  return null;
}

export function pcmuFirst(sdp) {
  // Each line keeps its own terminator, so joining them back gives the input, and whatever the
  // terminator was — CRLF from every browser, bare LF from a hand-written one — stays what it was.
  const lines = sdp.split(/(?<=\n)/);
  const at = audioSection(lines);
  const terminator = lines[at].match(/\r?\n$/)?.[0] ?? "";
  // `m=<media> <port> <proto> <fmt> ...`, single spaces between them (RFC 8866 §5.14).
  const [kind, port, proto, ...formats] = lines[at]
    .slice(0, -terminator.length || undefined)
    .split(" ");

  if (!formats.includes("0")) {
    const elsewhere = vocabulary(lines, at).find((entry) => entry.name.toLowerCase() === "pcmu");
    throw new Error(
      elsewhere
        ? `the offer names PCMU at payload ${elsewhere.payload}, and the server resolves PCMU only ` +
          "at payload 0, so putting it first would be answered and then refused"
        : "the offer's m=audio has no payload 0, so there is no PCMU for the server to select",
    );
  }
  // Whether the head is PCMU *to the server* is part of putting PCMU first, so this sits here and
  // not with the vocabulary check below: `mapping_matches` rejects a channel count, so an
  // `a=rtpmap:0 PCMU/8000/1` offer has a 0 at the head that the server resolves as nothing.
  const zero = vocabulary(lines, at).find((entry) => entry.payload === "0");
  if (zero && (zero.name.toLowerCase() !== "pcmu" || zero.clock !== "8000" || zero.channels)) {
    throw new Error(
      `the offer maps payload 0 to ${zero.name}/${zero.clock}${zero.channels ? `/${zero.channels}` : ""}` +
        ", and the server resolves payload 0 as PCMU only when it is PCMU/8000 with no channel count",
    );
  }
  if (formats[0] === "0") return sdp;
  // One occurrence removed, not every one: a list naming 0 twice keeps both, because this moves a
  // token and a filter would delete one.
  const rest = [...formats];
  rest.splice(rest.indexOf("0"), 1);
  lines[at] = [kind, port, proto, "0", ...rest].join(" ") + terminator;
  return lines.join("");
}
