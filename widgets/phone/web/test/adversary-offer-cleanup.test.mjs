// `offer.mjs`'s own cleanup promise, held to under node.
//
// `offer.mjs` says it "cannot be tested under node" because `RTCPeerConnection` and
// `navigator.mediaDevices` are the browser's and nothing in this repository fakes either. That is
// true of the *media*: no peer connection here will gather a candidate or open a microphone. It is
// not true of the one guarantee that file makes in prose, `web/offer.mjs:29-32`:
//
//   > On any failure after the microphone opened — an offer `pcmuFirst` refuses, a browser that
//   > will not take the reordered description — the stream is stopped and the connection closed
//   > before the error is rethrown, so a page that failed to offer is not a page with a live
//   > microphone.
//
// Whether a rethrow is preceded by a `track.stop()` is decidable with two stubs and no media at
// all, which is what this file does. Added by the adversarial pass; it faked nothing that carries
// audio.

import assert from "node:assert/strict";
import { test } from "node:test";
import { createPcmuFirstOffer } from "../offer.mjs";

/** A microphone stream that records whether anybody stopped it. */
function fakeMicrophone() {
  const stopped = [];
  const track = {
    kind: "audio",
    stop() {
      stopped.push("audio");
    },
  };
  return {
    stopped,
    stream: { getAudioTracks: () => [track], getTracks: () => [track] },
  };
}

/** Installs the two browser globals `offer.mjs` reads, and returns them to what they were. */
function withBrowser({ getUserMedia, RTCPeerConnection }, body) {
  const navigatorWas = Object.getOwnPropertyDescriptor(globalThis, "navigator");
  const connectionWas = Object.getOwnPropertyDescriptor(globalThis, "RTCPeerConnection");
  Object.defineProperty(globalThis, "navigator", {
    value: { mediaDevices: { getUserMedia } },
    configurable: true,
    writable: true,
  });
  globalThis.RTCPeerConnection = RTCPeerConnection;
  return (async () => body())().finally(() => {
    Object.defineProperty(globalThis, "navigator", navigatorWas);
    if (connectionWas) Object.defineProperty(globalThis, "RTCPeerConnection", connectionWas);
    else delete globalThis.RTCPeerConnection;
  });
}

test("a connection that fails to construct still leaves the microphone stopped", async () => {
  // `configuration` is the caller's, and the doc comment invites `{iceServers}` for STUN. A
  // malformed or credential-less entry is what `RTCPeerConnection`'s constructor rejects — with a
  // synchronous `SyntaxError` for a URL it cannot parse and `InvalidAccessError` for a TURN URL
  // with no credential (WebRTC 1.0 §4.3.1, `setConfiguration` steps 4 and 5). That throw happens
  // *after* `getUserMedia` resolved, which is the case the doc comment names.
  const microphone = fakeMicrophone();
  await assert.rejects(
    withBrowser(
      {
        getUserMedia: async () => microphone.stream,
        RTCPeerConnection: class {
          constructor() {
            throw new SyntaxError("Failed to parse URL: stun:");
          }
        },
      },
      () => createPcmuFirstOffer({ iceServers: [{ urls: "stun:" }] }),
    ),
    { message: /Failed to parse URL/ },
  );
  assert.deepEqual(
    microphone.stopped,
    ["audio"],
    "the microphone was still live after the offer failed",
  );
});
