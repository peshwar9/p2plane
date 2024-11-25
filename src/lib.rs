//! P2Plane: A P2P networking library

pub mod p2plane;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}

// Re-export commonly used types and traits
pub use p2plane::{
    behavior::Behavior,
    network::PeerManager,
    traits::{Message, PeerManagement},
};

// Common type definitions
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
