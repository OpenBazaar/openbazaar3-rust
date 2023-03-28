use std::error::Error;
use futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::{Multiaddr};
use libp2p::kad::KademliaEvent;
use libp2p::swarm::{SwarmBuilder, NetworkBehaviour};
use tokio::sync::{mpsc, oneshot};
use libp2p::{swarm::SwarmEvent};
use libp2p::multiaddr::Protocol;

use libp2p::kad::{Kademlia, record::store::MemoryStore};

// TODO: connect to bootstrap nodes
// const BOOTNODES: [&str; 1] = [
//     "/ip4/[insert_ip]/tcp/4001/ipfs/[insert_peer_id]"
// ];

pub async fn new(keypair: Keypair) -> Result<(Client, EventLoop), Box<dyn Error>> {
    
    let peer_id = keypair.public().to_peer_id();

    // Create transport for determining how to send data on the network
    let transport = libp2p::tokio_development_transport(keypair).unwrap();

    // Behaviour outlines what bytes to send and to whom
    let behaviour = ComposedBehaviour {
        kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
        // TODO: add in OpenBazazar behaviour
    };
    
    // Create libp2p swarm
    let swarm = SwarmBuilder::with_tokio_executor(
        transport, 
        behaviour,
        peer_id)
            .build();
        
    // Create command channel with buffer of 1 to process messages in order
    let (command_sender, command_receiver) = mpsc::channel(1);

    Ok((
        Client {
            sender: command_sender,
        },
        EventLoop::new(swarm, command_receiver)
    ))

}

pub fn generate_key() -> libp2p::identity::Keypair {
    libp2p::identity::Keypair::generate_ed25519()
}

#[derive(Clone, Debug)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub async fn start_listening(
        &mut self, addr: Multiaddr,
    ) -> anyhow::Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender.send(Command::StartListening {addr, sender}).await.expect("Failed to send command");
        receiver.await.expect("Failed to send command")
    }
}

#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), anyhow::Error>>,
    },
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent", event_process = false)]
struct ComposedBehaviour {
    kademlia: Kademlia<MemoryStore>,
    // TODO: implement OpenBazaar network behaviour
}

#[derive(Debug)]
enum ComposedEvent {
    Kademlia(KademliaEvent),
    // TODO: implement OpenBazaar event
}

impl From<KademliaEvent> for ComposedEvent {
    fn from(event: KademliaEvent) -> Self {
        ComposedEvent::Kademlia(event)
    }
}

// TODO: placeholder implementation
// impl From<OpenBazaarEvent> for ComposedEvent {
//     fn from(event: OpenBazaarEvent) -> Self {
//         ComposedEvent::OpenBazaar(event)
//     }
// }

pub struct EventLoop {
    swarm: libp2p::Swarm<ComposedBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
}

impl EventLoop {
    fn new(swarm: libp2p::Swarm<ComposedBehaviour>, command_receiver: mpsc::Receiver<Command>) -> Self {
        Self {
            swarm,
            command_receiver,
        }
    }

    pub async fn run(&mut self) {
		loop {
			tokio::select! {
				event = self.swarm.next() => {
					self.handle_event(event.unwrap()).await
				},
				command = self.command_receiver.recv() => match command {
					Some(c) => self.handle_command(c).await,
					None => return,
				}
			}
		}
	}

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListening { addr, sender } => {
				let _ = match self.swarm.listen_on(addr) {
					Ok(_) => sender.send(Ok(())),
					Err(e) => sender.send(Err(anyhow::Error::from(e))),
				};
			}
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<ComposedEvent, std::io::Error>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
				let local_peer_id = *self.swarm.local_peer_id();
				println!("Local node is listening on {:?}",
					address.with(Protocol::P2p(local_peer_id.into()))
				)
			}
            SwarmEvent::ConnectionClosed { .. } => {},
			SwarmEvent::Dialing( .. ) => {},
			e => panic!("{:?}", e),
        }
    }

    
}