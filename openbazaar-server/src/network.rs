use futures::StreamExt;
use libp2p::core::PeerId;
use libp2p::identity::Keypair;
use libp2p::kad::record::Key;
use libp2p::kad::{
    AddProviderOk, GetClosestPeersOk, GetProvidersOk, GetRecordError, GetRecordOk, KademliaEvent,
    QueryId, QueryResult,
};
use libp2p::multiaddr::Protocol;
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::{NetworkBehaviour, SwarmBuilder};
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tokio::sync::{mpsc, oneshot};

use libp2p::kad::{record::store::MemoryStore, Kademlia};
use tracing::instrument;

use crate::openbazaar::NodeAddressType;

// TODO: connect to bootstrap nodes
// const BOOTNODES: [&str; 1] = [
//     "/ip4/[insert_ip]/tcp/4001/ipfs/[insert_peer_id]"
// ];

type ShareAddress = Vec<u8>;

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
    let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

    // Create command channel with buffer of 1 to process messages in order
    let (command_sender, command_receiver) = mpsc::channel(1);

    Ok((
        Client {
            sender: command_sender,
        },
        EventLoop::new(swarm, command_receiver),
    ))
}

#[derive(Clone, Debug)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    #[instrument]
    pub async fn start_listening(&mut self, addr: Multiaddr) -> anyhow::Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Failed to send command");
        receiver.await.expect("Failed to send command")
    }

    #[instrument]
    pub async fn dial(&mut self, peer_id: PeerId, addr: Multiaddr) -> anyhow::Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                addr,
                sender,
            })
            .await
            .expect("Failed to send command");
        receiver.await.expect("Failed to send command")
    }

    #[instrument]
    pub async fn start_providing(&self, share_addr: ShareAddress) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding { share_addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    #[instrument]
    pub async fn stop_providing(&self, share_addr: ShareAddress) {
        self.sender
            .send(Command::StopProviding { share_addr })
            .await
            .expect("Command receiver not to be dropped.");
    }

    #[instrument]
    pub async fn get_providers(&self, share_addr: ShareAddress) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetProviders { share_addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    // getclosestpeers
    #[instrument]
    pub async fn get_closest_peers(&self, addr: ShareAddress) -> anyhow::Result<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetClosestPeers { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    #[instrument]
    pub async fn get_clear_address(&self, peer_id: PeerId) -> anyhow::Result<NodeData> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetClearAddress { peer_id, sender })
            .await
            .expect("Command receiver not to be dropped.");
        let result = receiver.await.expect("Sender not to be dropped.");
        result
    }

    #[instrument]
    pub async fn put_clear_address(
        &self,
        address_type: crate::openbazaar::NodeAddressType,
        address: String,
    ) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::PutClearAddress {
                address_type,
                address,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
}

#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), anyhow::Error>>,
    },
    Dial {
        peer_id: PeerId,
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), anyhow::Error>>,
    },
    StartProviding {
        share_addr: ShareAddress,
        sender: oneshot::Sender<()>,
    },
    StopProviding {
        share_addr: ShareAddress,
    },
    GetProviders {
        share_addr: ShareAddress,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    GetClearAddress {
        peer_id: PeerId,
        sender: oneshot::Sender<anyhow::Result<NodeData>>,
    },
    PutClearAddress {
        address_type: crate::openbazaar::NodeAddressType,
        address: String,
        sender: oneshot::Sender<()>,
    },
    GetClosestPeers {
        addr: ShareAddress,
        sender: oneshot::Sender<anyhow::Result<PeerId>>,
    },
    GetListenAddress {
        sender: oneshot::Sender<anyhow::Result<Vec<Multiaddr>>>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeData {
    pub peer_id: Vec<u8>,
    pub address: String,
    pub address_type: NodeAddressType,
}

pub struct EventLoop {
    swarm: libp2p::Swarm<ComposedBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), anyhow::Error>>>,
    pending_start_providing: HashMap<QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<QueryId, oneshot::Sender<HashSet<PeerId>>>,
    pending_get_closest_peer: HashMap<QueryId, oneshot::Sender<anyhow::Result<PeerId>>>,
    pending_get_clear_address: HashMap<QueryId, oneshot::Sender<anyhow::Result<NodeData>>>,
    providing: HashSet<Key>,
}

impl EventLoop {
    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(anyhow::Error::from(e))),
                };
            }
            Command::Dial {
                peer_id,
                addr,
                sender,
            } => {
                if let std::collections::hash_map::Entry::Vacant(_) =
                    self.pending_dial.entry(peer_id)
                {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                    match self.swarm.dial(addr) {
                        Ok(_) => {
                            self.pending_dial.insert(peer_id, sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(anyhow::Error::from(e)));
                        }
                    }
                }
            }
            Command::StartProviding { share_addr, sender } => {
                let key: Key = share_addr.to_vec().into();
                println!("Start providing: {:?}", key);
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(key)
                    .expect("No store error.");
                self.pending_start_providing.insert(query_id, sender);
            }
            Command::StopProviding { share_addr } => {
                let key: Key = share_addr.to_vec().into();
                self.swarm.behaviour_mut().kademlia.stop_providing(&key);
            }
            Command::GetProviders { share_addr, sender } => {
                let key: Key = share_addr.to_vec().into();
                let query_id = self.swarm.behaviour_mut().kademlia.get_providers(key);
                self.pending_get_providers.insert(query_id, sender);
            }
            Command::GetClosestPeers { addr, sender } => {
                let query_id = self.swarm.behaviour_mut().kademlia.get_closest_peers(addr);
                self.pending_get_closest_peer.insert(query_id, sender);
            }
            Command::GetClearAddress { peer_id, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_record(Key::from(peer_id.to_bytes()));
                self.pending_get_clear_address.insert(query_id, sender);
            }
            Command::GetListenAddress { sender } => {
                let peer_id = self.swarm.local_peer_id().to_owned().into();
                let addr: Vec<Multiaddr> = self
                    .swarm
                    .listeners()
                    .map(|addr| addr.to_owned().with(Protocol::P2p(peer_id)))
                    .collect();
                sender
                    .send(Ok(addr))
                    .expect("Failed to send listen address.")
            }
            _ => todo!(),
        }
    }

    /// The result of [`Kademlia::bootstrap`].
    // Bootstrap(BootstrapResult),

    // /// The result of a (automatic) republishing of a provider record.
    // RepublishProvider(AddProviderResult),

    // /// The result of [`Kademlia::get_record`].
    // GetRecord(GetRecordResult),

    // /// The result of [`Kademlia::put_record`].
    // PutRecord(PutRecordResult),

    // /// The result of a (automatic) republishing of a (value-)record.
    // RepublishRecord(PutRecordResult),

    async fn handle_event(&mut self, event: SwarmEvent<ComposedEvent, std::io::Error>) {
        match event {
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result: QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))),
                    ..
                },
            )) => {
                let data = peer_record.record.value.clone();
                let b: anyhow::Result<NodeData, bincode::Error> = bincode::deserialize(&data);
                let bundle = match b {
                    Ok(r) => Ok(r),
                    Err(e) => Err(anyhow::Error::from(e)),
                };

                let _ = self
                    .pending_get_clear_address
                    .remove(&id)
                    .expect("Completed query to previously pending")
                    .send(bundle);
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result:
                        QueryResult::GetRecord(Err(GetRecordError::NotFound { key, closest_peers })),
                    ..
                },
            )) => {
                let _ = self
                    .pending_get_clear_address
                    .remove(&id)
                    .expect("Completed query to previously pending")
                    .send(Ok(NodeData {
                        peer_id: "".into(),
                        address: "".into(),
                        address_type: NodeAddressType::Clear,
                    }));
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result:
                        QueryResult::GetProviders(Ok(GetProvidersOk::FinishedWithNoAdditionalRecord {
                            closest_peers,
                            ..
                        })),
                    ..
                },
            )) => {
                let closest_peers_set = closest_peers.into_iter().collect::<HashSet<_>>();
                let _ = self
                    .pending_get_providers
                    .remove(&id)
                    .expect("Completed query to previously pending")
                    .send(closest_peers_set);
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result:
                        QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders { providers, key })),
                    ..
                },
            )) => {
                let mut providers = providers.into_iter().collect::<HashSet<_>>();
                if self.providing.contains(&key) {
                    providers.insert(*self.swarm.local_peer_id());
                }
                let _ = self
                    .pending_get_providers
                    .remove(&id)
                    .expect("No pending get providers request.")
                    .send(providers);
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result: QueryResult::GetClosestPeers(Ok(GetClosestPeersOk { peers, key })),
                    ..
                },
            )) => {
                // Get k-bucket where this key is located
                let key = libp2p::kad::kbucket::Key::from(key);

                let local_peer_id = *self.swarm.local_peer_id();
                let local_peer_key = libp2p::kad::kbucket::Key::from(local_peer_id);

                // Find the distance between the key and the local peer
                let host_distance = local_peer_key.distance(&key);

                let mut peer_id: libp2p::identity::PeerId;
                if peers.is_empty() {
                    peer_id = local_peer_id;
                } else {
                    peer_id = peers.get(0).unwrap().to_owned();
                }

                let remote_peer_key = libp2p::kad::kbucket::Key::from(peer_id);

                // Find the distance between the key and the remote peer
                let remote_distance = remote_peer_key.distance(&key);

                // Check if local peer is the closest
                if remote_distance > host_distance {
                    peer_id = local_peer_id;
                }

                let _ = self
                    .pending_get_closest_peer
                    .remove(&id)
                    .expect("Completed query to previously pending")
                    .send(Ok(peer_id));
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id: query_id,
                    result: QueryResult::StartProviding(Ok(AddProviderOk { key })),
                    ..
                },
            )) => {
                self.providing.insert(key);
                if let Some(sender) = self.pending_start_providing.remove(&query_id) {
                    let _ = sender.send(());
                }
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(..)) => {}
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                println!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id.into()))
                )
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                println!(
                    "Adding peer {} with address {}",
                    peer_id,
                    endpoint.get_remote_address().clone()
                );
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, endpoint.get_remote_address().clone());
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::Dialing(..) => {}
            SwarmEvent::IncomingConnection {
                local_addr,
                send_back_addr,
            } => {
                tracing::debug!("local address: {:?}", local_addr);
                tracing::debug!("send back address: {:?}", send_back_addr);
                let mut remote_addr = send_back_addr.clone();
                let remote_port = remote_addr.pop().unwrap();
                let new_remote_port = match remote_port {
                    Protocol::Tcp(p) => Protocol::Tcp(p - 1),
                    _ => {
                        tracing::error!("This is a fatal error this shouldn't have happened");
                        remote_port
                    }
                };
                remote_addr.push(new_remote_port);
                self.swarm
                    .dial(remote_addr)
                    .expect("Error dialing send back addr");
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                tracing::error!("Had outgoing connection error {:?}", &error);
            }
            e => panic!("{:?}", e),
        }
    }

    fn new(
        swarm: libp2p::Swarm<ComposedBehaviour>,
        command_receiver: mpsc::Receiver<Command>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            pending_get_closest_peer: Default::default(),
            pending_get_clear_address: Default::default(),
            providing: Default::default(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                event = self.swarm.next() => {
                    println!("Event => {:?}", event);
                    self.handle_event(event.unwrap()).await
                },
                command = self.command_receiver.recv() => match command {
                    Some(c) => self.handle_command(c).await,
                    None => return,
                }
            }
        }
    }
}
