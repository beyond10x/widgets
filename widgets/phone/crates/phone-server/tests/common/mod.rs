//! The far end, for every test that needs one: an ordinary loopback endpoint playing the role
//! `sipx answer` plays.
//!
//! Shared by `sip_leg.rs` and `bridge.rs`, each of which compiles its own copy — which is why the
//! allow below is here rather than a set of helpers split by which file uses them. A harness
//! written twice is a harness that drifts.
#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use phone_server::sip::Destination;
use sipx_call::Call;
use sipx_transport::{Handle, Incoming, Target, TransportKind};
use tokio::sync::mpsc;

pub const LOOPBACK: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// Long enough that a loopback exchange is never the reason a case fails, short enough that a
/// wedged one fails the run instead of hanging it.
pub const PATIENCE: Duration = Duration::from_secs(10);

/// One ordinary signalling endpoint on a port the operating system chose.
pub async fn endpoint() -> (Handle, mpsc::Receiver<Incoming>) {
    let bind = SocketAddr::new(LOOPBACK, 0);
    sipx_transport::bind(sipx_transport::Config::new(bind))
        .await
        .expect("a loopback signalling endpoint")
}

/// Ring, answer, and take the ACK — the far end of a call the caller is going to keep using.
pub async fn answer_one(endpoint: &Handle, incoming: &mut mpsc::Receiver<Incoming>) -> Call {
    let invitation = incoming.recv().await.expect("an INVITE arrives");
    let ringing = sipx_call::ring(endpoint, &invitation, 180, "Ringing", false)
        .await
        .expect("the far end rings");
    let mut call = sipx_call::answer_ringing(endpoint, &invitation, LOOPBACK, &ringing)
        .await
        .expect("the far end answers");
    let ack = incoming.recv().await.expect("the ACK arrives");
    let _ = call.handle(&ack).await;
    call
}

/// [`answer_one`], then hold the call until the far end ends it.
///
/// The ACK and then the BYE arrive as ordinary requests on this endpoint; a call that is never
/// handed them never leaves its dialog.
pub async fn answer_and_wait(endpoint: Handle, mut incoming: mpsc::Receiver<Incoming>) -> Call {
    let mut call = answer_one(&endpoint, &mut incoming).await;
    while !call.is_ended() {
        let Some(request) = incoming.recv().await else {
            break;
        };
        let _ = call.handle(&request).await;
    }
    call
}

/// Where the far side is, as this crate's SIP leg names it.
pub fn destination(far: SocketAddr) -> Destination {
    Destination {
        to: sipx_sip::Uri::parse(bytes::Bytes::from(format!("sip:asterisk@{far}")))
            .expect("a dialable URI"),
        via: Target::new(far, TransportKind::Udp),
        from: "sip:phone-server@127.0.0.1".to_owned(),
        media_address: LOOPBACK,
    }
}
