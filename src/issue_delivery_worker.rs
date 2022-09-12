use crate::{configuration::Settings};
// use crate::{email_client::EmailClient};
use std::time::Duration;

use futures::prelude::*;
use libp2p::ping::{Ping, PingConfig};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{identity, Multiaddr, PeerId};



















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
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // uso il trasporto development (tcp + cifratura noise)
    let transport = libp2p::development_transport(local_key).await?;
    // Crea comportamento di rete ping.
    //
    // A scopo illustrativo, il protocollo ping è configurato per mantenere viva la connessione,
    // quindi una sequenza continua di ping può essere osservata.
    let behavior = Ping::new(PingConfig::new().with_keep_alive(true));
































    // ******************************************************************
    
    // ******************************************************************






    // unisce transport, behavior e local_peer_id per farli lavorare insieme
    let mut swarm = Swarm::new(transport, behavior, local_peer_id);

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {}", addr)
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::Behaviour(event) => println!("{:?}", event),
            _ => {}
        }
    }

    // fine loop

    
}

