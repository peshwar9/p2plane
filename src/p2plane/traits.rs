use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for managing peer connections and addresses. Applications can implement this trait
/// to customize how peers are stored and managed.
/// 
/// # Example
/// 
/// ```rust
/// use std::collections::{HashSet, HashMap};
/// use libp2p::{PeerId, Multiaddr};
/// use narwhal::p2plane::traits::PeerManagement;
/// 
/// #[derive(Debug)]
/// struct MyPeerManager {
///     peers: HashSet<PeerId>,
///     addresses: HashMap<PeerId, Multiaddr>,
/// }
/// 
/// impl MyPeerManager {
///     pub fn new() -> Self {
///         Self {
///             peers: HashSet::new(),
///             addresses: HashMap::new(),
///         }
///     }
/// }
/// 
/// impl PeerManagement for MyPeerManager {
///     fn get_peers(&self) -> Vec<PeerId> {
///         self.peers.iter().cloned().collect()
///     }
///     
///     fn add_peer_with_addr(&mut self, peer_id: PeerId, addr: Multiaddr) {
///         self.peers.insert(peer_id.clone());
///         self.addresses.insert(peer_id, addr);
///     }
/// }
/// ```
pub trait PeerManagement: Debug {
    fn get_peers(&self) -> Vec<PeerId>;
    fn add_peer_with_addr(&mut self, peer_id: PeerId, addr: Multiaddr);
}

/// Trait for application-specific messages that can be sent over the network.
/// 
/// # Example
/// 
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use narwhal::p2plane::traits::Message;
/// 
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyTransactionMessage {
///     data: Vec<u8>,
///     timestamp: u64,
/// }
/// 
/// impl Message for MyTransactionMessage {
///     fn protocol_id(&self) -> &'static str {
///         "/my-app/1.0.0"
///     }
/// }
/// ```
pub trait Message: 
    Serialize + 
    for<'de> Deserialize<'de> +
    Clone + 
    Send + 
    Sync + 
    Debug + 
    'static 
{
    fn protocol_id(&self) -> &'static str;
}