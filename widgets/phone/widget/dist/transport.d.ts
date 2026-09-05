/**
 * Browser → server.
 *
 * One offer and one destination, then the four things a page can decide about a call — and, before
 * any of them, the handle this phone is claiming. Presence is not a property of a call and outlives
 * every one of them, which is why it is on this wire and not inside `open-bridge`.
 */
export type Leg = {
    leg: "open-bridge";
    bridge_id: string;
    session_id: string;
    call_id: string;
    destination: string;
    offer: string;
} | {
    leg: "announce";
    presence_id: string;
    handle: string;
    label: string;
} | {
    leg: "withdraw";
    presence_id: string;
} | {
    leg: "hangup";
    call_id: string;
} | {
    leg: "digits";
    call_id: string;
    digits: string;
} | {
    leg: "mute";
    call_id: string;
    muted: boolean;
} | {
    leg: "hold";
    call_id: string;
    held: boolean;
};
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
export declare class Link {
    #private;
    private constructor();
    /** Opens the channel and resolves once it is open, or rejects if it never opens. */
    static connect(endpoint: string, watcher: Watcher): Promise<Link>;
    /** Says one thing to the server. */
    send(leg: Leg): void;
    /** Drops the channel, which takes both legs of the call with it. */
    close(): void;
    get open(): boolean;
}
