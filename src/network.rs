use std::error::Error;
use futures::StreamExt;
use libp2p::Multiaddr;
use libp2p::kad::KademliaEvent;
use libp2p::swarm::{SwarmBuilder, NetworkBehaviour};
use tokio::sync::{mpsc, oneshot};
use libp2p::{swarm::SwarmEvent};
use libp2p::kad::{Kademlia, record::store::MemoryStore};

pub async fn new() -> Result<(Client, EventLoop), Box<dyn Error>> {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();

    // Create transport
    let transport = libp2p::development_transport(keypair).await.unwrap();

    // Create libp2p swarm
    let swarm = SwarmBuilder::with_tokio_executor(
        transport, 
        ComposedBehaviour {
            kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
        },
        peer_id)
            .build();

    // Create command channel
    let (command_sender, command_receiver) = mpsc::channel(1);

    Ok((
        Client {
            sender: command_sender,
        },
        EventLoop::new(swarm, command_receiver)
    ))

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
}

#[derive(Debug)]
enum ComposedEvent {
    Kademlia(KademliaEvent),
}

impl From<KademliaEvent> for ComposedEvent {
    fn from(event: KademliaEvent) -> Self {
        ComposedEvent::Kademlia(event)
    }
}

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
            SwarmEvent::ConnectionClosed { .. } => {},
			SwarmEvent::Dialing( .. ) => {},
			e => panic!("{:?}", e),
        }
    }

    
}