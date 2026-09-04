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
use std::time::Duration;

use sipx_call::Call;
use sipx_media::{Bridge, MediaSession};

/// One bridge, holding both halves.
///
/// Dropping it stops the forwarding in both directions at once. Half a bridge is a call where one
/// party can hear and the other cannot, which is worse than a call that has ended — so sipx's
/// `Bridge` takes the whole thing down when either direction stops, and this type does not try to
/// hold one direction open.
///
/// **Muting and holding are two flags, and each gates a different leg.** They were one flag until
/// they were found to alias: unholding cleared a mute the page never lifted, and clearing mute
/// resumed a leg that was still held. `softphone.control.Call` declares `muted` and `held` as two
/// independent Boolean fields with two commands and two events, and `sipx-call`'s own `Call::mute`
/// heads a section "Mute is not hold". One `AtomicBool` cannot carry two independent answers.
///
/// **And which leg a flag gates is the thing to get right, because a gate is directional.**
/// `MediaSession::set_muted` replaces this session's *outbound* frames with silence and says of
/// itself that "reception is not affected in any way". So:
///
/// | to silence | gate the outbound of |
/// | --- | --- |
/// | the page's microphone, as the far end hears it | the **SIP** leg |
/// | the far end's voice, as the page hears it | the **browser** leg |
///
/// The page's microphone arrives on the browser leg's *reception*, crosses the bridge, and leaves
/// through the SIP leg — so muting the microphone is a gate on the SIP leg, and gating the browser
/// leg instead would leave the far end hearing a muted page while the page went deaf. That is
/// exactly what this type did until it was measured: 4 000 samples at loudest 7 932 crossing to the
/// far end while muted, and loudest 0 arriving at the page. [`Self::regate`] is the one place the
/// two flags meet the two legs.
#[derive(Debug)]
pub struct Bridged {
    bridge: Bridge,
    /// The leg whose outbound audio the page hears.
    browser: Arc<MediaSession>,
    /// The leg whose outbound audio the far end hears. The same session `call.media_handle()`
    /// answers, so `Call::is_muted` observes this gate.
    sip: Arc<MediaSession>,
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
        let sip = call.media_handle();
        let bridge = Bridge::connect(Arc::clone(&browser), Arc::clone(&sip));
        Self {
            bridge,
            browser,
            sip,
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

    /// Mute the page's microphone, and answer what **mute** was before.
    ///
    /// "The browser's mute", not "a mute of the browser leg": what it silences is the audio the
    /// page is sending, which reaches the far end out of the *SIP* leg. The far end is told
    /// nothing — no re-INVITE, no lifecycle move — which is what `softphone.control` says muting
    /// is, and what the page hears is untouched, because that arrives out of the browser leg and
    /// mute does not gate it. Hold is the one that gates both.
    ///
    /// The previous value comes back rather than a success flag, because the caller that wants to
    /// report a real transition would otherwise have to read the flag and then write it, and those
    /// two steps race a second muter. It is the previous value of `muted` alone: clearing mute on a
    /// held call answers `true` and leaves both legs silent, because hold is still holding them.
    pub fn set_browser_muted(&self, muted: bool) -> bool {
        let was = self.muted.swap(muted, Ordering::SeqCst);
        self.regate();
        was
    }

    /// Hold, which gates **both** legs and neither leg's signalling.
    ///
    /// `softphone.control` says holding is local: "the far end keeps the leg, this side stops
    /// sending and rendering". Stops sending is the SIP leg's gate and stops rendering is the
    /// browser leg's, so holding sets both — the far end goes on hearing a stream, of silence, and
    /// so does the page. No re-INVITE is sent. A held call is still `Active`, which is why this is
    /// not a lifecycle move.
    ///
    /// It no longer takes the `Call`. It used to call `Call::mute` itself, which was the only
    /// reason hold worked at all while mute was gating the wrong leg; both gates are now
    /// [`Self::regate`]'s and there is one place they are decided.
    ///
    /// Answers what **hold** was before, for the same reason [`Self::set_browser_muted`] answers
    /// what mute was.
    pub fn set_held(&self, held: bool) -> bool {
        let was = self.held.swap(held, Ordering::SeqCst);
        self.regate();
        was
    }

    /// The one place the two flags meet the two legs.
    ///
    /// Mute silences the microphone, which leaves through the SIP leg. Hold silences that *and*
    /// what the page hears, which leaves through the browser leg. So the SIP leg's gate is the
    /// disjunction and the browser leg's is hold alone — and lifting one flag does not answer the
    /// other's question, which is what the two flags exist for.
    fn regate(&self) {
        let muted = self.muted.load(Ordering::SeqCst);
        let held = self.held.load(Ordering::SeqCst);
        self.sip.set_muted(muted || held);
        self.browser.set_muted(held);
    }

    /// Resolve once the forwarding has stopped.
    ///
    /// Polled, because `sipx_media::Bridge` offers `is_connected` and nothing awaitable: its two
    /// directions are `JoinHandle`s it owns, and there is no completion this crate can wait on
    /// without reaching inside it. One wakeup per call per interval is the cost of that.
    pub async fn until_lost(&self, poll: Duration) {
        while self.is_connected() {
            tokio::time::sleep(poll).await;
        }
    }
}
