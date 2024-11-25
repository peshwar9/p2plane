pub mod behavior;
pub mod network;
pub mod peer_manager;
pub mod traits;

#[cfg(test)]
pub(crate) mod tests;

pub use behavior::{Behavior, Event as BehaviorEvent};
pub use network::{PeerManager, PeerStorage};
pub use traits::PeerManagement;

// Common types used across the library
use libp2p::PeerId;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<libp2p::Multiaddr>,
}