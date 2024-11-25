use narwhal::p2plane::{
    network::{Node, NodeConfig},
    traits::Message,
};
use serde::{Serialize, Deserialize};
use std::error::Error;
use libp2p::Multiaddr;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMessage(String);

impl Message for TestMessage {
    fn protocol_id(&self) -> &'static str {
        "/test/1.0.0"
    }
}

#[tokio::test]
async fn test_node_integration() -> Result<(), Box<dyn Error>> {
    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
    let config = NodeConfig {
        listen_addr: addr.to_string(),
        bootstrap_addr: None,
    };

    let _node = Node::<TestMessage>::new(config).await?;
    Ok(())
}

#[tokio::test]
async fn test_node_with_bootstrap() -> Result<(), Box<dyn Error>> {
    // Start bootstrap node
    let bootstrap_addr: Multiaddr = "/ip4/127.0.0.1/tcp/8000".parse()?;
    let bootstrap_config = NodeConfig {
        listen_addr: bootstrap_addr.to_string(),
        bootstrap_addr: None,
    };
    let _bootstrap_node = Node::<TestMessage>::new(bootstrap_config).await?;

    // Allow bootstrap node to start
    sleep(Duration::from_secs(1)).await;

    // Start peer node
    let peer_addr: Multiaddr = "/ip4/127.0.0.1/tcp/8001".parse()?;
    let peer_config = NodeConfig {
        listen_addr: peer_addr.to_string(),
        bootstrap_addr: Some(bootstrap_addr),
    };
    let _peer_node = Node::<TestMessage>::new(peer_config).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_nodes() -> Result<(), Box<dyn Error>> {
    let mut nodes = Vec::new();
    let base_port = 9000;

    // Create bootstrap node
    let bootstrap_addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", base_port).parse()?;
    let bootstrap_config = NodeConfig {
        listen_addr: bootstrap_addr.to_string(),
        bootstrap_addr: None,
    };
    let bootstrap_node = Node::<TestMessage>::new(bootstrap_config).await?;
    nodes.push(bootstrap_node);

    // Allow bootstrap node to start
    sleep(Duration::from_secs(1)).await;

    // Create additional nodes
    for i in 1..3 {
        let peer_addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", base_port + i).parse()?;
        let peer_config = NodeConfig {
            listen_addr: peer_addr.to_string(),
            bootstrap_addr: Some(bootstrap_addr.clone()),
        };
        let peer_node = Node::<TestMessage>::new(peer_config).await?;
        nodes.push(peer_node);
        
        // Allow each node to connect
        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}