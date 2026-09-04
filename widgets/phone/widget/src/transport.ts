// The control channel: one WebSocket, JSON, and every message an ESS command.
//
// Nothing here invents a protocol. What arrives is the request the module's own bridge dispatches
// — `{"request":"command","command":…,"input":{…}}` — so an inbound frame is forwarded verbatim and
// this file needs no table of server messages. What leaves is `phone_server::wire::FromBrowser`,
// whose spellings are serde's: the variant tag is `leg`, kebab-cased, and the fields keep the
// specification's own snake_case.

/** Browser → server. One offer and one destination, then the four things a page can decide. */
export type Leg =
  | {
      leg: "open-bridge";
      bridge_id: string;
      session_id: string;
      call_id: string;
      destination: string;
      offer: string;
    }
  | { leg: "hangup"; call_id: string }
  | { leg: "digits"; call_id: string; digits: string }
  | { leg: "mute"; call_id: string; muted: boolean }
  | { leg: "hold"; call_id: string; held: boolean };

/** Server → browser: one ESS command per frame, in the module's own request shape. */
export interface Inbound {
  request: "command";
  command: string;
  input: Record<string, unknown>;
}

/** What a link tells its holder. Every one of these is a fact, never a request for a decision. */
export interface Watcher {
  /** A command the server issued as a kernel actor. */
  command: (inbound: Inbound) => void;
  /** The channel is gone. A phone with no channel has no far leg, whatever its screen says. */
  closed: (reason: string) => void;
}

/**
 * One control channel, for as long as it lasts.
 *
 * A frame this page cannot read is dropped and reported rather than guessed at, the same way
 * `phone-server` treats a frame it cannot parse. There is no message on this wire for "your frame
 * was malformed", and inventing one would be a message the specification does not name.
 */
export class Link {
  #socket: WebSocket;
  #watcher: Watcher;

  private constructor(socket: WebSocket, watcher: Watcher) {
    this.#socket = socket;
    this.#watcher = watcher;
    socket.addEventListener("message", (event) => this.#arrived(event));
    socket.addEventListener("close", (event) =>
      watcher.closed(event.reason || `the channel closed (${event.code})`),
    );
    socket.addEventListener("error", () => watcher.closed("the channel failed"));
  }

  /** Opens the channel and resolves once it is open, or rejects if it never opens. */
  static connect(endpoint: string, watcher: Watcher): Promise<Link> {
    return new Promise((resolve, reject) => {
      const socket = new WebSocket(endpoint);
      socket.addEventListener("open", () => resolve(new Link(socket, watcher)), { once: true });
      socket.addEventListener(
        "error",
        () => reject(new Error(`no control channel at ${endpoint}`)),
        { once: true },
      );
    });
  }

  /** Says one thing to the server. */
  send(leg: Leg): void {
    this.#socket.send(JSON.stringify(leg));
  }

  /** Drops the channel, which takes both legs of the call with it. */
  close(): void {
    this.#socket.close();
  }

  get open(): boolean {
    return this.#socket.readyState === WebSocket.OPEN;
  }

  #arrived(event: MessageEvent): void {
    if (typeof event.data !== "string") return;
    let body: unknown;
    try {
      body = JSON.parse(event.data);
    } catch {
      this.#watcher.closed("a frame on the control channel was not JSON");
      return;
    }
    const inbound = body as Partial<Inbound>;
    if (inbound.request !== "command" || typeof inbound.command !== "string") {
      this.#watcher.closed(`a frame this page has no message for: ${event.data.slice(0, 120)}`);
      return;
    }
    this.#watcher.command({
      request: "command",
      command: inbound.command,
      input: (inbound.input ?? {}) as Record<string, unknown>,
    });
  }
}
