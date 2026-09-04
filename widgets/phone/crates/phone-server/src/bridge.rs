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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sipx_call::Call;
use sipx_media::{Bridge, MediaSession};

/// One bridge, holding both halves.
///
/// Dropping it stops the forwarding in both directions at once. Half a bridge is a call where one
/// party can hear and the other cannot, which is worse than a call that has ended — so sipx's
/// `Bridge` takes the whole thing down when either direction stops, and this type does not try to
/// hold one direction open.
///
/// **Muting and holding are two flags, and the browser leg's gate is their disjunction.** They
/// were one flag until they were found to alias: unholding cleared a mute the page never lifted,
/// and clearing mute resumed a leg that was still held. `softphone.control.Call` declares `muted`
/// and `held` as two independent Boolean fields with two commands and two events, and `sipx-call`'s
/// own `Call::mute` heads a section "Mute is not hold". One `AtomicBool` cannot carry two
/// independent answers, so there are two — and the leg is silent while *either* is set.
#[derive(Debug)]
pub struct Bridged {
    bridge: Bridge,
    browser: Arc<MediaSession>,
    /// `softphone.control.Call.muted`, as the page last set it.
    muted: AtomicBool,
    /// `softphone.control.Call.held`, as the page last set it.
    held: AtomicBool,
}

impl Bridged {
    /// Forward audio between one browser session and one placed SIP call.
    #[must_use]
    pub fn connect(browser: Arc<MediaSession>, call: &Call) -> Self {
        // `media_handle` and not `media`: the bridge outlives this call frame, and what it holds
        // is each leg's media handle rather than a borrow of either leg.
        let bridge = Bridge::connect(Arc::clone(&browser), call.media_handle());
        Self {
            bridge,
            browser,
            muted: AtomicBool::new(false),
            held: AtomicBool::new(false),
        }
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

    /// `softphone.control.Call.muted`, as the page last set it.
    #[must_use]
    pub fn is_browser_muted(&self) -> bool {
        self.muted.load(Ordering::SeqCst)
    }

    /// `softphone.control.Call.held`, as the page last set it.
    #[must_use]
    pub fn is_held(&self) -> bool {
        self.held.load(Ordering::SeqCst)
    }

    /// Gate the browser leg's outbound audio, and answer what **mute** was before.
    ///
    /// Local to the browser leg: the far end is told nothing, which is what `softphone.control`
    /// says muting is, and the SIP leg keeps sending. What the browser hears is unaffected —
    /// muting gates this side's outbound audio and nothing else.
    ///
    /// The previous value comes back rather than a success flag, because the caller that wants to
    /// report a real transition would otherwise have to read the flag and then write it, and those
    /// two steps race a second muter. It is the previous value of `muted` alone: clearing mute on a
    /// held call answers `true` and leaves the leg silent, because hold is still holding it.
    pub fn set_browser_muted(&self, muted: bool) -> bool {
        let was = self.muted.swap(muted, Ordering::SeqCst);
        self.regate();
        was
    }

    /// Hold, which is both legs of this bridge and neither leg's signalling.
    ///
    /// `softphone.control` says holding is local: "the far end keeps the leg, this side stops
    /// sending and rendering". So both legs are gated and no re-INVITE is sent — the far end goes
    /// on hearing a stream, of silence, and the page goes on receiving one. A held call is still
    /// `Active`, which is why this is not a lifecycle move.
    ///
    /// Answers what **hold** was before, for the same reason [`Self::set_browser_muted`] answers
    /// what mute was.
    pub fn set_held(&self, held: bool, call: &Call) -> bool {
        let was = self.held.swap(held, Ordering::SeqCst);
        self.regate();
        // The SIP leg's half of a hold is hold's alone. Mute never touches it — that is what makes
        // "mute is local to the browser leg" true — so this is the one place it moves.
        if held {
            call.mute();
        } else {
            call.unmute();
        }
        was
    }

    /// Apply both flags to the browser leg's one gate.
    ///
    /// The disjunction is the whole point: a leg that is muted *or* held sends silence, and
    /// lifting one of the two does not answer the other's question.
    fn regate(&self) {
        self.browser
            .set_muted(self.muted.load(Ordering::SeqCst) || self.held.load(Ordering::SeqCst));
    }
}
