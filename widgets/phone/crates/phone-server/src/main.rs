//! `phone-server` — one WebSocket per phone, one SIP leg behind each.
//!
//! The transport here is deliberately thin: accept a connection, read JSON, hand it to
//! [`phone_server::session`], write back whatever that says. Every decision is in the library, and
//! everything in this file would be the same for any other framing.
//!
//! **One phone per connection, and no state outside it.** A connection holds its own bridge and
//! nothing else does; when it closes, the bridge is dropped and both legs go with it. That is the
//! ADR's arrangement made structural rather than remembered — there is no map here for a second
//! phone's call to be looked up in.

use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use phone_server::session::{self, apply, relay, Config, Live, OpenFailed, Phone};
use phone_server::wire::{BridgeCause, FromBrowser, TerminationReason, ToBrowser};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

/// Terminate one browser WebRTC audio leg, place one SIP call, bridge them.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Where the control channel listens.
    #[arg(long, env = "PHONE_SERVER_LISTEN", default_value = "127.0.0.1:8780")]
    listen: SocketAddr,

    /// Where SIP signalling binds.
    #[arg(long, env = "PHONE_SERVER_SIP_BIND", default_value = "0.0.0.0:0")]
    sip_bind: SocketAddr,

    /// The first SIP hop every call goes to — the PBX or proxy.
    #[arg(long, env = "PHONE_SERVER_SIP_PROXY")]
    sip_proxy: SocketAddr,

    /// This server's own address of record.
    #[arg(long, env = "PHONE_SERVER_FROM")]
    from: String,

    /// The address this server advertises for RTP on both legs.
    ///
    /// Separate from what the sockets bind: behind a NAT the two differ, and the socket's view is
    /// the wrong one to put in an SDP.
    #[arg(long, env = "PHONE_SERVER_MEDIA_ADDRESS")]
    media_address: IpAddr,

    /// The interface media sockets bind on. Defaults to the advertised address.
    #[arg(long, env = "PHONE_SERVER_MEDIA_BIND")]
    media_bind: Option<IpAddr>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let config = Config {
        sip_bind: args.sip_bind,
        sip_proxy: args.sip_proxy,
        from: args.from.clone(),
        media_address: args.media_address,
        media_bind: args.media_bind.unwrap_or(args.media_address),
    };

    // One signalling endpoint for the process, shared by every phone: a SIP endpoint is a port and
    // a transaction layer, not a per-call thing, and `sipx_call::dial` takes it by reference.
    let (signalling, _unsolicited) =
        sipx_transport::bind(sipx_transport::Config::new(config.sip_bind)).await?;

    let listener = TcpListener::bind(args.listen).await?;
    tracing::info!(listen = %args.listen, proxy = %config.sip_proxy, "phone-server is up");

    loop {
        let (stream, from) = listener.accept().await?;
        let phone = Phone::new(config.clone(), signalling.clone());
        tokio::spawn(async move {
            if let Err(error) = serve(stream, phone).await {
                tracing::warn!(%from, %error, "the phone's channel ended");
            }
        });
    }
}

/// One phone, for as long as its connection lasts.
async fn serve(stream: TcpStream, phone: Phone) -> Result<(), Box<dyn std::error::Error>> {
    let websocket = tokio_tungstenite::accept_async(stream).await?;
    let (mut sink, mut incoming) = websocket.split();

    // Everything the page is told goes through one channel, so the ordering the specification
    // relies on — the answer before the ringing, the ringing before the answer — is the ordering
    // of one queue rather than of two writers racing for a socket.
    let (say, mut outgoing) = unbounded_channel::<ToBrowser>();
    let writer = tokio::spawn(async move {
        while let Some(message) = outgoing.recv().await {
            let text = message.request().to_string();
            if sink.send(Message::text(text)).await.is_err() {
                break;
            }
        }
    });

    let mut live: Option<Live> = None;
    let mut relaying: Option<tokio::task::JoinHandle<()>> = None;
    // The identifiers the page minted, kept so that a bridge dying mid-call can be named in the
    // `FailBridge` the page is told about. This server holds no phone state; these are the page's
    // own strings echoed back to it.
    let mut opened_bridge = String::new();
    let mut opened_session = String::new();

    loop {
        // A bounded wait rather than a plain `next()`, because two things can end a phone and only
        // one of them arrives as a frame: the page can say something, or the bridge can stop
        // forwarding. `sipx_media::Bridge` offers nothing awaitable, so a bridge that dies mid-call
        // is noticed by looking — and this is where the looking happens, on a connection that is
        // otherwise idle. `session::bridge_lost` holds the same interval.
        let arrived = tokio::time::timeout(session::BRIDGE_POLL, incoming.next()).await;
        if let Some(live_now) = live.as_ref() {
            if !live_now.bridge.is_connected() {
                tracing::info!("the bridge stopped forwarding; the page is told");
                let _ = say.send(ToBrowser::FailBridge {
                    bridge_id: opened_bridge.clone(),
                    session_id: opened_session.clone(),
                    cause: BridgeCause::Transport,
                    reason: TerminationReason::TransportLost,
                });
                break;
            }
        }
        let Ok(next) = arrived else {
            continue;
        };
        let Some(message) = next else {
            break;
        };

        let Message::Text(text) = message? else {
            // Binary, ping and close frames: nothing on this channel is a byte string, and a
            // frame that is not JSON is not a message this protocol has.
            continue;
        };
        // A frame this server will not parse is warned and dropped, and so is a second
        // `OpenBridge` below — so a page whose message was malformed waits for an answer that never
        // comes, with nothing on the wire saying why. **That wants a refusal on the wire**, and it
        // is deliberately not in this round: the specification has no command for "your control
        // frame was malformed" — every server-to-browser message here is an ESS command a kernel
        // actor may issue — so answering it means either adding one to `ess/`, which is not this
        // unit's surface, or inventing a message outside the specification, which is the one thing
        // the ADR forbids this channel. It is named here rather than left for the next reader to
        // rediscover.
        let message: FromBrowser = match serde_json::from_str(&text) {
            Ok(message) => message,
            Err(error) => {
                tracing::warn!(%error, "a frame this control channel has no message for");
                continue;
            }
        };

        match (&message, live.as_mut()) {
            (
                FromBrowser::OpenBridge {
                    bridge_id,
                    session_id,
                    call_id,
                    destination,
                    offer,
                },
                None,
            ) => {
                let told = say.clone();
                let opened = phone
                    .open(bridge_id, session_id, destination, offer, &move |command| {
                        let _ = told.send(command);
                    })
                    .await;
                match opened {
                    Ok((opened, reports)) => {
                        opened_bridge.clone_from(bridge_id);
                        opened_session.clone_from(session_id);
                        let told = say.clone();
                        let call_id = call_id.clone();
                        relaying = Some(tokio::spawn(async move {
                            relay(call_id, reports, |command| {
                                let _ = told.send(command);
                            })
                            .await;
                        }));
                        live = Some(opened);
                    }
                    Err(failure) => {
                        tracing::warn!(%failure, "the bridge did not come up");
                        report(&say, &failure, bridge_id, session_id, call_id);
                    }
                }
            }
            (_, Some(live)) => apply(&message, live).await,
            (_, None) => {
                // Silent for the same reason as a malformed frame, and recorded there.
                tracing::warn!("a message about a bridge this channel does not have; ignored");
            }
        }
    }

    if let Some(task) = relaying {
        task.abort();
    }
    drop(live);
    drop(say);
    let _ = writer.await;
    Ok(())
}

/// Tell the page about a bridge that did not come up.
///
/// Two commands and not one: the bridge is gone, and so is the call the page dialled. A page told
/// only about the bridge would be left with a `Call` in `Requested` that nothing will ever move.
///
/// Both classifications are [`OpenFailed`]'s, not this function's. It used to decide the call's
/// class here and got it wrong for four of the five variants — every one that was not a far-leg
/// failure came out as `Media`.
fn report(
    say: &UnboundedSender<ToBrowser>,
    failure: &OpenFailed,
    bridge_id: &str,
    session_id: &str,
    call_id: &str,
) {
    let _ = say.send(failure.fail_call(&call_id.to_owned()));
    let _ = say.send(failure.fail_bridge(&bridge_id.to_owned(), &session_id.to_owned()));
}
