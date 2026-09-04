// The offer the page sends: audio only, PCMU ahead of Opus, every ICE candidate already in it.
//
// This file cannot be tested under node, and does not pretend to be: `RTCPeerConnection` and
// `navigator.mediaDevices` are the browser's, and nothing in this repository fakes either.
// Everything that can be decided without a browser — the reorder — is `pcmu-first.mjs`, and
// `test/pcmu-first.test.mjs` decides it. What is left here is the browser calls in the order
// they have to happen, and the order is the point:
//
//   1. `getUserMedia({audio: true})` — the track exists before the offer describes it;
//   2. `createOffer()` — Opus first, which is what every browser does by default;
//   3. `pcmuFirst` — the one change, on the string, before the browser commits to it;
//   4. `setLocalDescription` with that string — a reordered format list is a change the browser
//      accepts: measured 2026-09-04, headless Chrome 150.0.7871.46 and Firefox 153.0 both took it
//      and both carried PCMU at the head of `localDescription` through gathering;
//   5. wait for ICE gathering to complete — non-trickle, because the server answers one offer
//      once, and an offer that leaves before its candidates do is answered without them.
//
// Browser-targeted ESM with no build step, the same as `player/skin.js`.

import { pcmuFirst } from "./pcmu-first.mjs";

/**
 * An audio-only offer with PCMU first and its candidates gathered — with the connection it
 * belongs to, which the caller needs to apply the server's answer, and the microphone stream it
 * carries, which the caller needs to stop when the call ends.
 *
 * `configuration` is the `RTCPeerConnection`'s own; pass `{iceServers}` for STUN. The default,
 * none, gathers host candidates only, which is what the README says this phone does.
 *
 * On any failure after the microphone opened — an offer `pcmuFirst` refuses, a browser that will
 * not take the reordered description — the stream is stopped and the connection closed before
 * the error is rethrown, so a page that failed to offer is not a page with a live microphone.
 */
export async function createPcmuFirstOffer(configuration = {}) {
  const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  // Inside the `try`, because `RTCPeerConnection` throws synchronously on a configuration it will
  // not take — a malformed URL, a TURN server with no credential — and the microphone is already
  // open by then. Constructed outside it, that throw left a live microphone behind, which is the
  // one thing the paragraph above promises cannot happen.
  let connection;
  try {
    connection = new RTCPeerConnection(configuration);
    for (const track of stream.getAudioTracks()) connection.addTrack(track, stream);
    const { sdp } = await connection.createOffer();
    await connection.setLocalDescription({ type: "offer", sdp: pcmuFirst(sdp) });
    await gathered(connection);
    return { connection, stream, sdp: connection.localDescription.sdp };
  } catch (error) {
    for (const track of stream.getTracks()) track.stop();
    connection?.close();
    throw error;
  }
}

/** Resolves once the connection's ICE gathering is complete. */
function gathered(connection) {
  if (connection.iceGatheringState === "complete") return Promise.resolve();
  return new Promise((resolve) => {
    connection.addEventListener("icegatheringstatechange", function done() {
      if (connection.iceGatheringState !== "complete") return;
      connection.removeEventListener("icegatheringstatechange", done);
      resolve();
    });
  });
}
