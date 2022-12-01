use crate::{configuration::Settings};
// use crate::{email_client::EmailClient};
// use std::time::Duration;

// use futures::prelude::*;
// use libp2p::ping::{Ping, PingConfig};
// use libp2p::swarm::{Swarm, SwarmEvent};
// use libp2p::{identity, Multiaddr, PeerId};




use futures::StreamExt;
use libp2p::core::identity;
use libp2p::core::PeerId;
use libp2p::multiaddr::Protocol;
use libp2p::ping::{Ping, PingConfig, PingEvent, PingSuccess};
use libp2p::swarm::SwarmEvent;
use libp2p::Swarm;
use libp2p::{development_transport, rendezvous, Multiaddr};
use std::time::Duration;

const NAMESPACE: &str = "rendezvous";


















pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    println!( "{}",configuration.database.database_name.to_lowercase() );
    // let connection_pool = get_connection_pool(&configuration.database);
    // let email_client = configuration.email_client.client();
    // worker_loop(email_client).await
    worker_loop().await
}

async fn worker_loop() -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task().await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty
    ),
    err
)]
pub async fn try_execute_task() -> Result<ExecutionOutcome, anyhow::Error> {

    // inizio loop
    println!("ciao mondo");
    tracing::debug!("ciao mondo");

































    // ******************************************************************

    let identity = identity::Keypair::generate_ed25519();
    let rendezvous_point_address = "/ip4/127.0.0.1/tcp/62649".parse::<Multiaddr>().unwrap();
    let rendezvous_point = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
        .parse()
        .unwrap();

    let mut swarm = Swarm::new(
        development_transport(identity.clone()).await.unwrap(),
        MyBehaviour {
            rendezvous: rendezvous::client::Behaviour::new(identity.clone()),
            ping: Ping::new(
                PingConfig::new()
                    .with_interval(Duration::from_secs(1))
                    .with_keep_alive(true),
            ),
        },
        PeerId::from(identity.public()),
    );

    log::info!("Local peer id: {}", swarm.local_peer_id());

    let _ = swarm.dial(rendezvous_point_address.clone()).unwrap();

    let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
    let mut cookie = None;

    loop {
        tokio::select! {
                event = swarm.select_next_some() => match event {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == rendezvous_point => {
                        log::info!(
                            "Connected to rendezvous point, discovering nodes in '{}' namespace ...",
                            NAMESPACE
                        );

                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                            None,
                            None,
                            rendezvous_point,
                        );
                    }
                    SwarmEvent::Behaviour(MyEvent::Rendezvous(rendezvous::client::Event::Discovered {
                        registrations,
                        cookie: new_cookie,
                        ..
                    })) => {
                        cookie.replace(new_cookie);

                        for registration in registrations {
                            for address in registration.record.addresses() {
                                let peer = registration.record.peer_id();
                                log::info!("Discovered peer {} at {}", peer, address);

                                let p2p_suffix = Protocol::P2p(*peer.as_ref());
                                let address_with_p2p =
                                    if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                                        address.clone().with(p2p_suffix)
                                    } else {
                                        address.clone()
                                    };

                                swarm.dial(address_with_p2p).unwrap()
                            }
                        }
                    }
                    SwarmEvent::Behaviour(MyEvent::Ping(PingEvent {
                        peer,
                        result: Ok(PingSuccess::Ping { rtt }),
                    })) if peer != rendezvous_point => {
                        log::info!("Ping to {} is {}ms", peer, rtt.as_millis())
                    }
                    other => {
                        log::debug!("Unhandled {:?}", other);
                    }
            },
            _ = discover_tick.tick(), if cookie.is_some() =>
                swarm.behaviour_mut().rendezvous.discover(
                    Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                    cookie.clone(),
                    None,
                    rendezvous_point
                    )
        }
    }

    // ******************************************************************






    // fine loop

    
}




















#[derive(Debug)]
enum MyEvent {
    Rendezvous(rendezvous::client::Event),
    Ping(PingEvent),
}

impl From<rendezvous::client::Event> for MyEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        MyEvent::Rendezvous(event)
    }
}

impl From<PingEvent> for MyEvent {
    fn from(event: PingEvent) -> Self {
        MyEvent::Ping(event)
    }
}

#[derive(libp2p::NetworkBehaviour)]
#[behaviour(event_process = false)]
#[behaviour(out_event = "MyEvent")]
struct MyBehaviour {
    rendezvous: rendezvous::client::Behaviour,
    ping: Ping,
}
