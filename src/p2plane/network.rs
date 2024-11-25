use crate::p2plane::{
    traits::{Message, PeerManagement},
    behavior::{Behavior, Event as BehaviorEvent},
};
use std::fs;
use libp2p::{
    identity, Multiaddr, PeerId, SwarmBuilder,
    swarm::{Swarm,  SwarmEvent},
    kad::{
        store::MemoryStore,
        Behaviour as Kademlia,
        Config as KadConfig,
        Event as KadEvent,
    },
    identify::{
        Behaviour as Identify,
        Config as IdentifyConfig,
    },
    request_response::{
        cbor::Behaviour as RequestResponse,
        Config as RequestResponseConfig,
        Event as RequestResponseEvent,
        Message as RequestResponseMessage,
        ProtocolSupport,
    },
    tcp::Config as TcpConfig,
    yamux,
    noise,
    core::ConnectedPoint,
    StreamProtocol,
};
use libp2p::futures::StreamExt;
use log::{debug, error, info};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex as TokioMutex;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use libp2p::request_response::OutboundFailure;

// Custom serialization wrapper for PeerId
#[derive(Debug, Serialize, Deserialize)]
struct SerializablePeerId(String);

impl From<PeerId> for SerializablePeerId {
    fn from(peer_id: PeerId) -> Self {
        SerializablePeerId(peer_id.to_string())
    }
}

impl TryFrom<SerializablePeerId> for PeerId {
    type Error = String;
    fn try_from(spid: SerializablePeerId) -> Result<Self, Self::Error> {
        spid.0
            .parse()
            .map_err(|e| format!("Failed to parse PeerId: {}", e))
    }
}

// Custom serialization wrapper for Multiaddr
#[derive(Debug, Serialize, Deserialize)]
struct SerializableMultiaddr(String);

impl From<Multiaddr> for SerializableMultiaddr {
    fn from(addr: Multiaddr) -> Self {
        SerializableMultiaddr(addr.to_string())
    }
}

impl TryFrom<SerializableMultiaddr> for Multiaddr {
    type Error = String;
    fn try_from(sma: SerializableMultiaddr) -> Result<Self, Self::Error> {
        sma.0
            .parse()
            .map_err(|e| format!("Failed to parse Multiaddr: {}", e))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PeerStorage {
    peers: HashSet<String>,             // Store peer IDs as strings
    addresses: HashMap<String, String>, // Store addresses as strings
}

impl PeerStorage {
    pub fn new(peer_id: &PeerId) -> Self {
        let filename = format!("peers_{}.json", peer_id.to_base58());
        info!("Loading peer storage from {}", filename);

        match fs::read_to_string(&filename) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(storage) => {
                    info!("Successfully loaded peer storage from {}", filename);
                    storage
                }
                Err(e) => {
                    error!("Failed to parse {}: {}. Creating new storage.", filename, e);
                    PeerStorage::default()
                }
            },
            Err(e) => {
                info!("No existing storage found ({}), creating new one at {}", e, filename);
                PeerStorage::default()
            }
        }
    }

    pub fn save_to_disk(&self, peer_id: &PeerId) -> Result<(), std::io::Error> {
        let filename = format!("peers_{}.json", peer_id.to_base58());
        info!("Saving peer storage to {}", filename);

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&filename, content)?;
        info!("Successfully saved peer storage to {}", filename);
        Ok(())
    }
}

#[derive(Debug)]
pub struct PeerManager {
    pub peers: HashSet<PeerId>,
    pub peer_addresses: HashMap<PeerId, Multiaddr>,
    local_peer_id: PeerId,
}

impl PeerManager {
    pub fn new(local_peer_id: PeerId) -> Self {
        info!(
            "[PeerManager::new] Creating new instance for {:?}",
            local_peer_id
        );
        let storage = PeerStorage::new(&local_peer_id);

        let peers = storage
            .peers
            .iter()
            .filter_map(|p| match p.parse::<PeerId>() {
                Ok(peer_id) => {
                    info!("[PeerManager::new] Loaded peer: {:?}", peer_id);
                    Some(peer_id)
                }
                Err(e) => {
                    error!("[PeerManager::new] Failed to parse peer ID {}: {}", p, e);
                    None
                }
            })
            .collect();

        let manager = PeerManager {
            peers,
            peer_addresses: storage
                .addresses
                .iter()
                .filter_map(|(p, a)| {
                    let peer_id = p.parse::<PeerId>().ok()?;
                    let addr = a.parse::<Multiaddr>().ok()?;
                    info!(
                        "[PeerManager::new] Loaded peer address: {:?} -> {:?}",
                        peer_id, addr
                    );
                    Some((peer_id, addr))
                })
                .collect(),
            local_peer_id,
        };

        info!(
            "[PeerManager::new] Created with {} peers",
            manager.peers.len()
        );
        manager
    }

    pub fn get_peers(&self) -> Vec<PeerId> {
        let peers = self.peers.iter().cloned().collect::<Vec<_>>();
        info!(
            "[PeerManager::get_peers] Returning {} peers: {:?}",
            peers.len(),
            peers
        );
        peers
    }

    pub fn add_peer_with_addr(&mut self, peer_id: PeerId, addr: Multiaddr) {
        debug!(
            "[PeerManager::add_peer_with_addr] Adding peer {:?} with addr {:?}",
            peer_id, addr
        );
        if peer_id == self.local_peer_id {
            debug!("[PeerManager::add_peer_with_addr] Skipping self peer");
            return;
        }
        self.peers.insert(peer_id.clone());
        self.peer_addresses.insert(peer_id, addr);
        debug!(
            "[PeerManager::add_peer_with_addr] Current peers: {:?}",
            self.peers
        );
        self.save_to_disk();
    }

    fn save_to_disk(&self) {
        let storage = PeerStorage {
            peers: self
                .peers
                .iter()
                .map(|p| p.to_base58().to_string())
                .collect(),
            addresses: self
                .peer_addresses
                .iter()
                .map(|(p, a)| (p.to_base58().to_string(), a.to_string()))
                .collect(),
        };

        if let Err(e) = storage.save_to_disk(&self.local_peer_id) {
            error!("Failed to save peer storage: {}", e);
        } else {
            info!(
                "Successfully saved peers to disk for {:?}",
                self.local_peer_id
            );
        }
    }

    pub fn get_peer_address(&self, peer_id: &PeerId) -> Option<&Multiaddr> {
        self.peer_addresses.get(peer_id)
    }
}

impl PeerManagement for PeerManager {
    fn add_peer_with_addr(&mut self, peer_id: PeerId, addr: Multiaddr) {
        PeerManager::add_peer_with_addr(self, peer_id, addr)
    }

    fn get_peers(&self) -> Vec<PeerId> {
        PeerManager::get_peers(self)
    }
}

pub struct Node<M: Message> {
    swarm: Swarm<Behavior<M>>,
    peer_manager: Arc<TokioMutex<PeerManager>>,
    config: NodeConfig,
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub listen_addr: String,
    pub bootstrap_addr: Option<Multiaddr>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".to_string(),
            bootstrap_addr: None,
        }
    }
}

impl<M: Message> Node<M> {
    pub async fn new(config: NodeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {}", local_peer_id);

        let peer_manager = Arc::new(TokioMutex::new(PeerManager::new(local_peer_id)));
        info!("Created peer manager for {}", local_peer_id);
        
        // Log the storage file name
        let storage_file = format!("peers_{}.json", local_peer_id.to_base58());
        info!("Peer storage file will be: {}", storage_file);

        let swarm = Self::build_swarm(local_key, peer_manager.clone()).await?;

        Ok(Self {
            swarm,
            peer_manager,
            config,
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Start listening
        self.swarm.listen_on(self.config.listen_addr.parse()?)?;

        // Try to connect to bootstrap node if specified
        if let Some(addr) = &self.config.bootstrap_addr {
            self.connect_with_retry(addr.clone()).await?;
        }

        // Event loop
        while let Some(event) = self.swarm.next().await {
            self.handle_event(event).await?;
        }

        Ok(())
    }

    pub async fn broadcast_message(&mut self, message: M) -> Result<(), Box<dyn std::error::Error>> {
        let peers = {
            let pm = self.peer_manager.lock().await;
            pm.get_peers()
        };

        for peer in peers {
            match self.swarm.behaviour_mut().request_response.send_request(&peer, message.clone()) {
                id => {
                    debug!("Sent message to peer {}, request id: {:?}", peer, id);
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviorEvent<M>>) -> Result<(), Box<dyn StdError>> {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!("Connection established with peer: {:?}", peer_id);
                let mut pm = self.peer_manager.lock().await;
                if let ConnectedPoint::Dialer { address, .. } = endpoint {
                    pm.add_peer_with_addr(peer_id, address);
                }
            }
            SwarmEvent::Behaviour(BehaviorEvent::RequestResponse(event)) => {
                match event {
                    RequestResponseEvent::Message { peer, message } => {
                        info!("Received message from peer {:?}: {:?}", peer, message);
                        match message {
                            RequestResponseMessage::Request { request, channel, .. } => {
                                if let Err(e) = self.swarm.behaviour_mut().request_response.send_response(channel, request.clone()) {
                                    error!("Failed to send response to peer {}: {:?}", peer, e);
                                    return Err(format!("Failed to send response: {:?}", e).into());
                                }
                            }
                            RequestResponseMessage::Response { .. } => {
                                // Handle response
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn connect_with_retry(&mut self, addr: Multiaddr) -> Result<(), Box<dyn std::error::Error>> {
        let mut retry_count = 0;
        while retry_count < 3 {
            match self.swarm.dial(addr.clone()) {
                Ok(_) => {
                    info!("Connected to bootstrap node: {}", addr);
                    return Ok(());
                }
                Err(e) => {
                    error!("Failed to connect to {}: {}", addr, e);
                    retry_count += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
        Err("Failed to connect after 3 retries".into())
    }

    async fn build_swarm(
        local_key: identity::Keypair,
        _peer_manager: Arc<TokioMutex<PeerManager>>,
    ) -> Result<Swarm<Behavior<M>>, Box<dyn std::error::Error>> {
        let local_peer_id = PeerId::from(local_key.public());
        info!("LocalPeerID: {local_peer_id}");

        let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                TcpConfig::default(),
                noise::Config::new,
                || yamux::Config::default(),
            )?
            .with_behaviour(|key| {
                let local_peer_id = PeerId::from(key.public());
                
                // Setup Kademlia
                let kad_store = MemoryStore::new(local_peer_id);
                let kad = Kademlia::with_config(
                    local_peer_id,
                    kad_store,
                    KadConfig::default(),
                );

                // Setup Identify
                let identify = Identify::new(
                    IdentifyConfig::new("/p2plane/1.0.0".to_string(), key.public())
                        .with_push_listen_addr_updates(true)
                        .with_interval(Duration::from_secs(30)),
                );

                // Setup Request/Response
                let rr_protocol = StreamProtocol::new("/p2plane/message/1.0.0");
                let request_response = RequestResponse::new(
                    [(rr_protocol, ProtocolSupport::Full)],
                    RequestResponseConfig::default(),
                );

                // Create behavior
                Ok(Behavior::new(kad, identify, request_response))
            })?
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
            .build();

        Ok(swarm)
    }
}
