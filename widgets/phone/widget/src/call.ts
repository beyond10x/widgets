// One outbound call, in the order the specification already wrote down.
//
// This file issues no sequence of its own invention: it is `scenarios/bridge-carries-outbound-call.
// yaml` at run time, and that scenario is executed headlessly on every `task check`. Where the two
// disagree the scenario is right.
//
//   ConfigureEndpoint · OpenSession(Bridge) · ConnectBridge(offer) · Dial
//     → the server answers ConfirmBridge, then RingCall, then ConfirmAnswer
//     → this page issues AttachMedia and MediaConnected
//
// **Those last two are the page's and not the server's**, which is a decision recorded in
// `crates/phone-server/tests/wire.rs`: only the page watches its own `RTCPeerConnection`, so only
// the page can say media is up, and the page minted the media session before it said anything to
// the server, so it attaches its own. A server reporting either would be claiming to see something
// it cannot.

import { createPcmuFirstOffer } from "./offer.mjs";
import { unanswerable, uncheckable } from "./pcmu-first.mjs";
import { Kernel, published, type Observation } from "./kernel";
import { Link, type Inbound } from "./transport";

/** The audio profile, whose four numbers the model fixes by invariant. */
const PROFILE = {
  sample_format: "PcmS16Le",
  sample_rate_hz: 8000,
  channels: 1,
  packet_time_ms: 20,
  frame_bytes: 320,
} as const;

/** What the phone tells its screen. Every one is a fact that has already happened. */
export interface Screen {
  /** The store changed: new views, a new log. */
  observed: (observation: Observation) => void;
  /** Something a person needs told in words, because no view carries it. */
  said: (words: string) => void;
}

/** A call this page is holding up, and everything that has to be let go of when it ends. */
interface Holding {
  call_id: string;
  bridge_id: string;
  session_id: string;
  connection: RTCPeerConnection;
  stream: MediaStream;
  /** The far end answered — `Call` is in `Answering`, so `activate` is legal. */
  answered: boolean;
  /** ICE and DTLS are up, which only this page can observe. */
  media: boolean;
  /** `MediaConnected` has been issued; it moves `Answering` to `Active` and only once. */
  activated: boolean;
}

/**
 * The phone: one module, one control channel, one call at a time.
 *
 * One call because that is what `phone-server` serves per connection and what the ADR arranges —
 * a connection holds its own bridge and there is no map for a second phone's call to be found in.
 * The model permits two `Active` calls and that is its own change, listed as out of scope.
 */
export class Phone {
  #kernel: Kernel;
  #link: Link | null = null;
  #screen: Screen;
  #endpoint: string;
  #endpointId: string | null = null;
  #presenceId: string | null = null;
  #holding: Holding | null = null;

  constructor(kernel: Kernel, endpoint: string, screen: Screen) {
    this.#kernel = kernel;
    this.#endpoint = endpoint;
    this.#screen = screen;
  }

  /** What the module projects right now, for a screen that has just been mounted. */
  observe(): Observation {
    return this.#kernel.observe();
  }

  /** Whether a call is up, which is what a screen enables its buttons from. */
  get busy(): boolean {
    return this.#holding !== null;
  }

  /**
   * Names this phone to the model, once.
   *
   * `default_binding: Bridge` is the whole of what makes this a bridged phone rather than a
   * loopback one, and it is a field of the endpoint rather than a flag of this class.
   */
  configure(label: string): void {
    if (this.#endpointId) return;
    const observation = this.#kernel.run("softphone.control.ConfigureEndpoint", {
      label,
      default_binding: "Bridge",
    });
    this.#endpointId =
      published(observation, "softphone.control.EndpointConfigured", "endpoint_id") ?? null;
    this.#screen.observed(observation);
  }

  /**
   * Claims a handle, so other phones can be told this one is here.
   *
   * The channel is opened for this rather than for a call: a phone is in the server's table from
   * the moment it announces until its connection goes, whether or not it ever dials. Whether the
   * handle was accepted is not known when this resolves — the server answers with
   * `softphone.presence.ConfirmPresence` or `FailPresence`, and until one arrives the model holds
   * this phone in `Announcing`, reachable by nobody.
   *
   * @throws when there is no server to announce to.
   */
  async announce(handle: string, label: string): Promise<void> {
    if (this.#presenceId) throw new Error("this phone has already claimed a handle");
    const link = await this.#connect();
    const announcing = this.#kernel.run("softphone.presence.AnnouncePresence", { handle, label });
    const presence_id = published(
      announcing,
      "softphone.presence.PresenceAnnounced",
      "presence_id",
    );
    this.#screen.observed(announcing);
    if (!presence_id) throw new Error("the module refused the announcement; the log says why");
    this.#presenceId = presence_id;
    link.send({ leg: "announce", presence_id, handle, label });
  }

  /** Gives the handle back, keeping the channel. */
  withdraw(): void {
    const presence_id = this.#presenceId;
    if (!presence_id) return;
    this.#presenceId = null;
    this.#screen.observed(
      this.#kernel.run("softphone.presence.WithdrawPresence", { presence_id }),
    );
    this.#link?.send({ leg: "withdraw", presence_id });
  }

  /**
   * Places one call.
   *
   * The offer exists before anything is told about it, and it is checked against the server's own
   * audio profile here — `unanswerable` mirrors `sipx_sdp::browser_audio` — so an offer this
   * server would decline costs no round trip and no half-open bridge. Firefox's offer is refused
   * by that check today, with the reason; `story:browser-audio-vocabulary-is-narrower-than-firefox`
   * is the fix and it is upstream.
   *
   * @throws when the microphone is refused, or the offer is one the far side cannot answer.
   */
  async dial(destination: string, configuration: RTCConfiguration = {}): Promise<void> {
    if (!this.#endpointId) throw new Error("configure the phone before dialling");
    if (this.#holding) throw new Error("this phone is already holding a call");

    const { connection, stream, sdp } = await createPcmuFirstOffer(configuration);
    // Both questions, because they fail differently: a vocabulary this server has no answer for
    // is the browser's, and an ICE generation it cannot check is the offer's own shape.
    const refusal = unanswerable(sdp) ?? uncheckable(sdp);
    if (refusal) {
      this.#letGo(connection, stream);
      throw new Error(`this browser's offer cannot be answered: ${refusal}`);
    }

    const link = await this.#connect().catch((error: unknown) => {
      this.#letGo(connection, stream);
      throw error;
    });

    // The three identifiers, each minted by the module and read back out of its own event. The
    // session is opened first because the bridge references it, and the call is dialled last
    // because a `Requested` call with nowhere to go is the state nothing can move out of.
    const opened = this.#kernel.run("softphone.media.OpenSession", {
      session_ref: `bridge-${Date.now().toString(36)}`,
      binding: "Bridge",
      profile: PROFILE,
      participant: { reference: destination, trust: "Untrusted" },
    });
    const session_id = published(opened, "softphone.media.SessionOpened", "session_id");
    this.#screen.observed(opened);

    const connecting = this.#kernel.run("softphone.bridge.ConnectBridge", {
      session_id,
      endpoint: this.#endpoint,
      offer: sdp,
    });
    const bridge_id = published(connecting, "softphone.bridge.BridgeConnecting", "bridge_id");
    this.#screen.observed(connecting);

    const dialled = this.#kernel.run("softphone.control.Dial", {
      endpoint_id: this.#endpointId,
      remote: destination,
    });
    const call_id = published(dialled, "softphone.control.CallDialled", "call_id");
    this.#screen.observed(dialled);

    if (!session_id || !bridge_id || !call_id) {
      this.#letGo(connection, stream);
      throw new Error("the module refused part of the sequence; the log says which");
    }

    const holding: Holding = {
      call_id,
      bridge_id,
      session_id,
      connection,
      stream,
      answered: false,
      media: false,
      activated: false,
    };
    this.#holding = holding;

    // Registered before the offer goes out, because the answer can come back before the next line
    // runs and a connection whose state changed unobserved is a call that never becomes `Active`.
    connection.addEventListener("connectionstatechange", () => this.#stateChanged(holding));
    connection.addEventListener("track", (event) => this.#screenAudio(event));

    link.send({
      leg: "open-bridge",
      bridge_id,
      session_id,
      call_id,
      destination,
      offer: sdp,
    });
  }

  /** Ends the call from this side: the model, the server, and this page's own media. */
  hangUp(): void {
    const holding = this.#holding;
    if (!holding) return;
    this.#screen.observed(
      this.#kernel.run("softphone.control.HangUp", { call_id: holding.call_id }),
    );
    if (this.#link?.open) this.#link.send({ leg: "hangup", call_id: holding.call_id });
    this.#close(holding, "Local", "Completed");
  }

  /** Gates this side's outbound audio. Local to the browser leg; the far end is told nothing. */
  setMuted(muted: boolean): void {
    const holding = this.#holding;
    if (!holding) return;
    this.#screen.observed(
      this.#kernel.run("softphone.control.SetMuted", { call_id: holding.call_id, muted }),
    );
    this.#link?.send({ leg: "mute", call_id: holding.call_id, muted });
  }

  /** Holds the call. A held call is still `Active` — `held` is a field, not a state. */
  setHeld(held: boolean): void {
    const holding = this.#holding;
    if (!holding) return;
    this.#screen.observed(
      this.#kernel.run("softphone.control.SetHeld", { call_id: holding.call_id, held }),
    );
    this.#link?.send({ leg: "hold", call_id: holding.call_id, held });
  }

  /** Sends keys on the SIP leg, as RFC 4733 events and never as audio. */
  sendDigits(digits: string): void {
    const holding = this.#holding;
    if (!holding) return;
    this.#screen.observed(
      this.#kernel.run("softphone.control.SendDigits", { call_id: holding.call_id, digits }),
    );
    this.#link?.send({ leg: "digits", call_id: holding.call_id, digits });
  }

  /** The channel, opened once and kept for as long as it lasts. */
  async #connect(): Promise<Link> {
    if (this.#link?.open) return this.#link;
    this.#link = await Link.connect(this.#endpoint, {
      command: (inbound) => this.#told(inbound),
      closed: (reason) => this.#channelClosed(reason),
    });
    return this.#link;
  }

  /**
   * One command the server issued, applied.
   *
   * Forwarded verbatim, with one interception: `ConfirmBridge` carries the answer this page has to
   * install, and installing it before dispatching keeps the module's `Live` bridge and the
   * browser's remote description in the same order the server put them in.
   */
  #told(inbound: Inbound): void {
    const holding = this.#holding;
    if (inbound.command === "softphone.bridge.ConfirmBridge" && holding) {
      const answer = inbound.input["answer"];
      if (typeof answer !== "string") {
        this.#screen.said("the server confirmed a bridge with no answer in it");
        return;
      }
      void holding.connection
        .setRemoteDescription({ type: "answer", sdp: answer })
        .then(() => this.#screen.observed(this.#kernel.run(inbound.command, inbound.input)))
        .catch((error: unknown) => {
          this.#screen.said(`this browser would not take the answer: ${String(error)}`);
          this.hangUp();
        });
      return;
    }

    // Everything else goes in verbatim, which includes all four of `softphone.presence`'s: a
    // roster is a projection of facts about other phones, and none of them needs a peer connection,
    // a track or a socket of its own.
    this.#screen.observed(this.#kernel.run(inbound.command, inbound.input));

    if (!holding) return;
    if (inbound.command === "softphone.control.ConfirmAnswer") {
      holding.answered = true;
      this.#activate(holding);
    }
    if (
      inbound.command === "softphone.control.FailCall" ||
      inbound.command === "softphone.bridge.FailBridge"
    ) {
      // The far leg or the bridge is gone, and the model has already recorded why. What is left is
      // this page's own media, which nothing else releases.
      this.#letGo(holding.connection, holding.stream);
      this.#holding = null;
    }
  }

  /** ICE and DTLS, which only this page can see. */
  #stateChanged(holding: Holding): void {
    if (this.#holding !== holding) return;
    const state = holding.connection.connectionState;
    if (state === "connected") {
      holding.media = true;
      this.#activate(holding);
      return;
    }
    if (state === "failed" || state === "closed") {
      // Reported, not asserted into the model: `softphone.control.FailCall` is the server's to
      // issue, and the server sees the same bridge stop within `BRIDGE_POLL`. A page that raced it
      // would be two authorities on one call's ending.
      this.#screen.said(`the media path is ${state}; the server will report the call`);
    }
  }

  /**
   * Attaches the media and says it is up — once, and only when both halves are true.
   *
   * The gate is the whole point. `MediaConnected` takes `activate`, whose only legal `from` is
   * `Answering`, and DTLS completes as soon as the answer is applied — which is well before a
   * person picks up. Issuing it on `connected` alone refuses with a `CallStateConflict` and the
   * call never becomes `Active`.
   */
  #activate(holding: Holding): void {
    if (holding.activated || !holding.answered || !holding.media) return;
    holding.activated = true;
    this.#screen.observed(
      this.#kernel.run("softphone.control.AttachMedia", {
        call_id: holding.call_id,
        session_id: holding.session_id,
      }),
    );
    this.#screen.observed(
      this.#kernel.run("softphone.control.MediaConnected", { call_id: holding.call_id }),
    );
  }

  /** The far end's audio, rendered. Without this a call connects and nobody hears anything. */
  #screenAudio(event: RTCTrackEvent): void {
    const [stream] = event.streams;
    if (!stream) return;
    let audio = document.getElementById("phone-far-end") as HTMLAudioElement | null;
    if (!audio) {
      audio = document.createElement("audio");
      audio.id = "phone-far-end";
      audio.autoplay = true;
      document.body.append(audio);
    }
    audio.srcObject = stream;
  }

  /** Closes the bridge in the model, then lets this page's media go. */
  #close(holding: Holding, cause: string, reason: string): void {
    this.#screen.observed(
      this.#kernel.run("softphone.bridge.CloseBridge", {
        bridge_id: holding.bridge_id,
        session_id: holding.session_id,
        cause,
        reason,
      }),
    );
    this.#letGo(holding.connection, holding.stream);
    this.#holding = null;
  }

  /** The microphone stops and the connection closes. Neither outlives the call. */
  #letGo(connection: RTCPeerConnection, stream: MediaStream): void {
    for (const track of stream.getTracks()) track.stop();
    connection.close();
  }

  #channelClosed(reason: string): void {
    this.#screen.said(reason);
    const holding = this.#holding;
    if (!holding) return;
    // A phone with no channel has no far leg. The server drops the bridge when the connection
    // goes, so the model is told the same thing the server did.
    this.#close(holding, "Transport", "TransportLost");
  }
}
