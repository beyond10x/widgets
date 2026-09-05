/** An audio-only offer with PCMU first and its ICE candidates already gathered. */
export function createPcmuFirstOffer(configuration?: RTCConfiguration): Promise<{
  connection: RTCPeerConnection;
  stream: MediaStream;
  sdp: string;
}>;
