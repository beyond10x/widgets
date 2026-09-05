/** Whether this server's audio profile can answer the offer, and why not when it cannot. */
export function unanswerable(sdp: string): string | null;

/** The same offer with PCMU's payload type at the head of the `m=audio` format list. */
export function pcmuFirst(sdp: string): string;

/** The same description with every candidate the server cannot check over UDP removed. */
export function udpCandidatesOnly(sdp: string): string;

/** Why the server could not check this offer's ICE, or `null` if it can. */
export function uncheckable(sdp: string): string | null;
