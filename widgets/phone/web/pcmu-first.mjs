// PCMU ahead of everything else in an offer's `m=audio` format list, and no other change.
//
// The server bridges one WebRTC leg to one SIP leg, and its bridge moves buffers between the two
// with no rate conversion — an Opus leg at 48 kHz against a G.711 leg at 8 kHz delivered 500 ms
// of speech as 2,880 ms — so it refuses any pair whose rates differ, at the media seam, before
// ICE and before the SIP call is placed.
//
// Which rate the browser leg has is decided by the offer, and only by the offer. Measured against
// `sipx_sdp::browser_audio`: an answer must keep the offer's format order — an answer hand-
// reordered to put PCMU first is `Err(CodecSetIncomplete)` — so the answerer has no say. Every
// browser offers Opus first by default, which is why the default offer is answered and then
// declined. This puts PCMU at the head, where the answer copies it from and the bridge reads it.
//
// The head of the list is the contract, not Opus's slot: PCMU goes first whatever was ahead of
// it. Placing it merely ahead of Opus would leave a G722-first offer failing at the same seam.
//
// Pure, over the SDP string, so `web/test/pcmu-first.test.mjs` can decide it without a browser.

/**
 * `sdp` with PCMU/8000 first in its one `m=audio` format list.
 *
 * Every other byte is unchanged: every other line, every terminator, and every `a=rtpmap`,
 * `a=fmtp` and `a=rtcp-fb` in the place the browser put it — an SDP is not required to order its
 * attributes to match the format list, and moving them would be a change nobody asked for. An
 * offer that already has PCMU first is returned as it is.
 *
 * PCMU is found by its `a=rtpmap` name, `PCMU/8000`, not by payload type 0. RFC 3551 makes 0
 * static and browsers do use it, but the `a=rtpmap` lines are the offer's own vocabulary, and a
 * function that assumed the number would be wrong for a reason its next reader cannot see. So a
 * format the offer does not name is not PCMU, even when it is 0.
 *
 * Refused, with the reason in the message: an offer naming no PCMU/8000 — returned unchanged it
 * would make a call that fails later, at the media seam, for a reason the page then has to
 * explain — and an offer with anything other than exactly one media section, audio. The server's
 * profile permits one audio section and `offer.mjs` builds an audio-only connection, so a second
 * `m=` of any kind is the page's own bug, and the message says how many there were.
 */
export function pcmuFirst(sdp) {
  // Each line keeps its own terminator, so joining them back gives the input, and whatever the
  // terminator was — CRLF from every browser, bare LF from a hand-written one — stays what it was.
  const lines = sdp.split(/(?<=\n)/);
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
  const terminator = lines[at].match(/\r?\n$/)?.[0] ?? "";
  // `m=<media> <port> <proto> <fmt> ...`, single spaces between them (RFC 8866 §5.14).
  const [kind, port, proto, ...formats] = lines[at]
    .slice(0, -terminator.length || undefined)
    .split(" ");
  if (kind !== "m=audio") {
    throw new Error(
      `the offer's one media section is ${kind.slice(2)}; the page's offer is m=audio`,
    );
  }
  // The section's own vocabulary. `a=rtpmap` is media-level, so only the lines after the `m=`.
  const pcmu = lines
    .slice(at + 1)
    .flatMap((line) => {
      const named = line.match(/^a=rtpmap:(\d+) ([^/\r\n]+)\/(\d+)/);
      return named && named[2].toLowerCase() === "pcmu" && named[3] === "8000" ? [named[1]] : [];
    })
    .find((type) => formats.includes(type));
  if (pcmu === undefined) {
    throw new Error(
      "the offer's m=audio names no PCMU/8000 in an a=rtpmap line, so there is nothing to put first",
    );
  }
  if (formats[0] === pcmu) return sdp;
  lines[at] =
    [kind, port, proto, pcmu, ...formats.filter((type) => type !== pcmu)].join(" ") + terminator;
  return lines.join("");
}
