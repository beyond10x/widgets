//! The bridge: `sipx_media::Bridge` between the browser leg and the SIP leg.
//!
//! Both legs are sipx media sessions, so this is the media coupling and nothing else. **No PCM is
//! injected into a live call anywhere in this crate** — every sample and payload moves between
//! the two sessions over their own channels, which is what keeps the read-side seam of
//! `docs/specs/call-audio-seam.md` off this path. If a change here ever needs to push samples into
//! a call by hand, the design has moved and the story is wrong rather than the code being clever.
//!
//! `sipx-call`'s `CallBridge` is the path from *two `Call`s* to that same forwarding, and it is
//! not what this arrangement has: the browser leg is a `MediaSession` with no dialog, no signalling
//! and no `Call` around it. So the media type is reached directly, which is the seam
//! `crates/sipx-call/src/bridge.rs` itself points at.

use std::sync::Arc;

use sipx_call::Call;
use sipx_media::{Bridge, MediaSession};

/// One bridge, holding both halves.
///
/// Dropping it stops the forwarding in both directions at once. Half a bridge is a call where one
/// party can hear and the other cannot, which is worse than a call that has ended — so sipx's
/// `Bridge` takes the whole thing down when either direction stops, and this type does not try to
/// hold one direction open.
#[derive(Debug)]
pub struct Bridged {
    bridge: Bridge,
    browser: Arc<MediaSession>,
}

impl Bridged {
    /// Forward audio between one browser session and one placed SIP call.
    #[must_use]
    pub fn connect(browser: Arc<MediaSession>, call: &Call) -> Self {
        // `media_handle` and not `media`: the bridge outlives this call frame, and what it holds
        // is each leg's media handle rather than a borrow of either leg.
        let bridge = Bridge::connect(Arc::clone(&browser), call.media_handle());
        Self { bridge, browser }
    }

    /// Whether both directions are still running.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.bridge.is_connected()
    }

    /// Whether audio is being decoded and re-encoded rather than passed through.
    ///
    /// Reported rather than inferred, and worth reporting: a browser leg on Opus bridged to a
    /// G.711 SIP leg transcodes, and that costs quality as well as CPU.
    #[must_use]
    pub fn is_transcoding(&self) -> bool {
        self.bridge.is_transcoding()
    }

    /// Gate the browser leg's outbound audio, and answer what the gate was before.
    ///
    /// Local to the browser leg: the far end is told nothing, which is what `softphone.control`
    /// says muting is, and the SIP leg keeps sending. What the browser hears is unaffected —
    /// muting gates this side's outbound audio and nothing else.
    ///
    /// The previous value comes back rather than a success flag, because the caller that wants to
    /// report a real transition would otherwise have to read the flag and then write it, and those
    /// two steps race a second muter.
    pub fn set_browser_muted(&self, muted: bool) -> bool {
        self.browser.set_muted(muted)
    }

    /// Hold, which is both directions of this bridge and neither leg's signalling.
    ///
    /// `softphone.control` says holding is local: "the far end keeps the leg, this side stops
    /// sending and rendering". So both legs are gated and no re-INVITE is sent — the far end goes
    /// on hearing a stream, of silence, and the page goes on receiving one. A held call is still
    /// `Active`, which is why this is not a lifecycle move.
    pub fn set_held(&self, held: bool, call: &Call) {
        self.browser.set_muted(held);
        if held {
            call.mute();
        } else {
            call.unmute();
        }
    }
}
