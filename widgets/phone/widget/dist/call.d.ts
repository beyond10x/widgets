import { Kernel, type Observation } from "./kernel";
/** What the phone tells its screen. Every one is a fact that has already happened. */
export interface Screen {
    /** The store changed: new views, a new log. */
    observed: (observation: Observation) => void;
    /** Something a person needs told in words, because no view carries it. */
    said: (words: string) => void;
}
/**
 * The phone: one module, one control channel, one call at a time.
 *
 * One call because that is what `phone-server` serves per connection and what the ADR arranges —
 * a connection holds its own bridge and there is no map for a second phone's call to be found in.
 * The model permits two `Active` calls and that is its own change, listed as out of scope.
 */
export declare class Phone {
    #private;
    constructor(kernel: Kernel, endpoint: string, screen: Screen);
    /** What the module projects right now, for a screen that has just been mounted. */
    observe(): Observation;
    /** Whether a call is up, which is what a screen enables its buttons from. */
    get busy(): boolean;
    /**
     * Names this phone to the model, once.
     *
     * `default_binding: Bridge` is the whole of what makes this a bridged phone rather than a
     * loopback one, and it is a field of the endpoint rather than a flag of this class.
     */
    configure(label: string): void;
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
    announce(handle: string, label: string): Promise<void>;
    /** Gives the handle back, keeping the channel. */
    withdraw(): void;
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
    dial(destination: string, configuration?: RTCConfiguration): Promise<void>;
    /** Ends the call from this side: the model, the server, and this page's own media. */
    hangUp(): void;
    /** Gates this side's outbound audio. Local to the browser leg; the far end is told nothing. */
    setMuted(muted: boolean): void;
    /** Holds the call. A held call is still `Active` — `held` is a field, not a state. */
    setHeld(held: boolean): void;
    /** Sends keys on the SIP leg, as RFC 4733 events and never as audio. */
    sendDigits(digits: string): void;
}
