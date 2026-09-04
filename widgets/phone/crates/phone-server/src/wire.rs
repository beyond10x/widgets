//! The control channel's vocabulary: one WebSocket per phone, JSON, and every message an ESS
//! command.
//!
//! Nothing here is a protocol of this crate's invention. What the server sends the page is the
//! request the page's *own* generated bridge dispatches —
//! `{"request":"command","command":<name>,"input":{…}}`, which is
//! `.build/synth/web/crates/softphone-web/src/lib.rs`'s `fn answer` reading `request`, `command`
//! and `input` in exactly those words. So a fact about a leg arrives at the page as a command it
//! already accepts, and this crate adds no message the specification does not name.
//!
//! **Every command this server sends is one the specification grants to a *kernel* actor**, and
//! that is a real constraint rather than a description. It is why there is no `HangUp` here: the
//! far leg going away is `softphone.control.FailCall` with a cause, because
//! `softphone.control.HangUp` is granted to `Human` and `Agent` and to no kernel — hanging up is a
//! local decision and this process makes none. The ADR's own list of the facts a server reports
//! says the same six words.
//!
//! `tests/wire.rs` holds all of that to the compiled specification rather than to this comment: it
//! reads `ess compile --format json` and fails if a command named here is absent from the model,
//! is not in a kernel's `may:` list, or carries an input field the model does not declare — and it
//! fails the other way too, if the model grows a kernel command nobody has classified.

use serde::{Deserialize, Serialize};
use serde_json::json;

/// An identifier the specification declares as a `Uuid` newtype — a `BridgeId`, a
/// `MediaSessionId`, a `CallId`.
///
/// Carried as text and never parsed. This server mints none of them: the page has already run
/// `softphone.control.Dial` and `softphone.bridge.ConnectBridge` before it says anything here, so
/// every identifier on this channel is one the page chose and this process echoes.
pub type Id = String;

/// `softphone.control.EndCause` — why a call ended, in the six classes the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndCause {
    /// This side hung up.
    Local,
    /// The far end did.
    Remote,
    /// The far end refused the attempt.
    Refused,
    /// Signalling failed.
    Sip,
    /// Media failed.
    Media,
    /// The far end stopped answering.
    Timeout,
}

/// `softphone.bridge.BridgeCause` — why the bridge is gone, in four classes, none of them SIP's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeCause {
    /// The page closed it.
    Local,
    /// The server closed it.
    Remote,
    /// The connection died.
    Transport,
    /// The server would not have it.
    Refused,
}

/// `softphone.media.TerminationReason` — all eight variants, unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationReason {
    /// The session ran to its end.
    Completed,
    /// It was called off before it started.
    Cancelled,
    /// The far end hung up.
    RemoteHangup,
    /// Authority over it was withdrawn.
    AuthorityRevoked,
    /// Its lease ran out.
    LeaseExpired,
    /// The media path was overloaded.
    MediaOverload,
    /// The transport went away.
    TransportLost,
    /// The peer broke the protocol.
    ProtocolError,
}

/// Browser → server.
///
/// One offer and one destination, and then only the four things a page can decide. Non-trickle:
/// every candidate the page gathered is already inside `offer`, so no later message adds one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "leg", rename_all = "kebab-case", deny_unknown_fields)]
pub enum FromBrowser {
    /// Bridge this offer to this destination. The page has already created the call and the
    /// bridge session, so it names both.
    OpenBridge {
        /// The `softphone.bridge.BridgeSession` the page created.
        bridge_id: Id,
        /// The neutral `softphone.media.MediaSession` both legs belong to.
        session_id: Id,
        /// The `softphone.control.Call` the page dialled.
        call_id: Id,
        /// Where to place the SIP call — a URI or a number, whatever the far end accepts.
        destination: String,
        /// The browser's SDP offer, entire.
        offer: String,
    },
    /// End both legs.
    Hangup {
        /// Which call.
        call_id: Id,
    },
    /// Send these digits on the SIP leg, as RFC 4733 telephone events and never as audio.
    Digits {
        /// Which call.
        call_id: Id,
        /// The keys, in order.
        digits: String,
    },
    /// Gate this side's outbound audio. Local to the browser leg; the far end is told nothing.
    Mute {
        /// Which call.
        call_id: Id,
        /// The new value, rather than a toggle — the shape `softphone.control.SetMuted` uses.
        muted: bool,
    },
    /// Stop sending and rendering. Local to the browser leg, for the same reason.
    Hold {
        /// Which call.
        call_id: Id,
        /// The new value.
        held: bool,
    },
}

/// Server → browser: one ESS command per message, and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToBrowser {
    /// The SDP answer. `softphone.bridge.ConfirmBridge` is the only command that moves a bridge
    /// to `Live`, and the answer is what it carries.
    ConfirmBridge {
        /// Which bridge.
        bridge_id: Id,
        /// Which media session.
        session_id: Id,
        /// The answer this server authored.
        answer: String,
    },
    /// The far leg is ringing.
    RingCall {
        /// Which call.
        call_id: Id,
    },
    /// The far leg answered.
    ConfirmAnswer {
        /// Which call.
        call_id: Id,
    },
    /// The far leg is gone — it never came up, it was refused, or it ended.
    FailCall {
        /// Which call.
        call_id: Id,
        /// Which of the six classes.
        cause: EndCause,
    },
    /// The bridge itself is gone.
    FailBridge {
        /// Which bridge.
        bridge_id: Id,
        /// Which media session.
        session_id: Id,
        /// Whose doing.
        cause: BridgeCause,
        /// What the media layer is told.
        reason: TerminationReason,
    },
}

impl ToBrowser {
    /// The specification's own name for this command.
    #[must_use]
    pub fn command(&self) -> &'static str {
        match self {
            Self::ConfirmBridge { .. } => "softphone.bridge.ConfirmBridge",
            Self::RingCall { .. } => "softphone.control.RingCall",
            Self::ConfirmAnswer { .. } => "softphone.control.ConfirmAnswer",
            Self::FailCall { .. } => "softphone.control.FailCall",
            Self::FailBridge { .. } => "softphone.bridge.FailBridge",
        }
    }

    /// The request the page's generated bridge dispatches, ready to serialize.
    ///
    /// The field names are the specification's own, and so are the enum spellings: the generated
    /// decoder reads `"Local"`, `"Remote"` and the rest as the variants' own names, so `serde`'s
    /// default for a unit variant is exactly right and a `rename_all` here would break it.
    #[must_use]
    pub fn request(&self) -> serde_json::Value {
        let input = match self {
            Self::ConfirmBridge {
                bridge_id,
                session_id,
                answer,
            } => json!({
                "bridge_id": bridge_id,
                "session_id": session_id,
                "answer": answer,
            }),
            Self::RingCall { call_id } | Self::ConfirmAnswer { call_id } => json!({
                "call_id": call_id,
            }),
            Self::FailCall { call_id, cause } => json!({
                "call_id": call_id,
                "cause": cause,
            }),
            Self::FailBridge {
                bridge_id,
                session_id,
                cause,
                reason,
            } => json!({
                "bridge_id": bridge_id,
                "session_id": session_id,
                "cause": cause,
                "reason": reason,
            }),
        };
        json!({
            "request": "command",
            "command": self.command(),
            "input": input,
        })
    }
}
