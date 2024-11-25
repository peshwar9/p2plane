#[cfg(test)]
mod tests {
    use crate::p2plane::{
        network::{Node, NodeConfig},
        tests::TestMessage,
    };
    use std::error::Error;
    use libp2p::Multiaddr;

    #[tokio::test]
    async fn test_node_creation() -> Result<(), Box<dyn Error>> {
        let config = NodeConfig {
            listen_addr: "/ip4/127.0.0.1/tcp/0".parse()?,
            bootstrap_addr: None,
        };

        let _node = Node::<TestMessage>::new(config).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_node_config() -> Result<(), Box<dyn Error>> {
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/8000".parse()?;
        let config = NodeConfig {
            listen_addr: addr.to_string(),
            bootstrap_addr: None,
        };

        assert_eq!(config.listen_addr, addr.to_string());
        Ok(())
    }
} 