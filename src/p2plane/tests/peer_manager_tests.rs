#[cfg(test)]
use crate::p2plane::peer_manager::PeerManager;
use crate::p2plane::traits::PeerManagement;
use libp2p::{PeerId, Multiaddr};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_manager_creation() {
        let peer_id = PeerId::random();
        let manager = PeerManager::new(peer_id);
        assert!(manager.get_peers().is_empty());
    }

    #[test]
    fn test_add_peer() {
        let local_peer_id = PeerId::random();
        let mut manager = PeerManager::new(local_peer_id);
        
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/8000".parse().unwrap();
        
        manager.add_peer_with_addr(peer_id, addr.clone());
        assert_eq!(manager.get_peers().len(), 1);
        assert!(manager.get_peers().contains(&peer_id));
    }
} 