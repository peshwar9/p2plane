#[cfg(test)]
mod tests {
    use crate::p2plane::{
        tests::TestMessage,
        behavior::{Behavior, Event},
        traits::Message,
    };
    use libp2p::{
        kad::{
            store::MemoryStore,
            Behaviour as Kademlia,
            Config as KademliaConfig,
        },
        identify::{
            Behaviour as Identify,
            Event as IdentifyEvent,
            Info,
            Config as IdentifyConfig,
        },
        request_response::{
            self,
            ProtocolSupport,
        },
        PeerId,
        identity::Keypair,
        StreamProtocol,
    };

    #[tokio::test]
    async fn test_behavior_creation() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        // Create Kademlia
        let store = MemoryStore::new(peer_id);
        let kad_config = KademliaConfig::default();
        let kad = Kademlia::with_config(peer_id, store, kad_config);
        
        // Create Identify
        let identify_config = IdentifyConfig::new(
            "test".to_string(),
            keypair.public(),
        );
        let identify = Identify::new(identify_config);
        
        // Create RequestResponse
        let msg = TestMessage("".to_string());
        let protocol = StreamProtocol::new(msg.protocol_id());
        let protocols = vec![(protocol, ProtocolSupport::Full)];
        let request_response = request_response::cbor::Behaviour::<TestMessage, TestMessage>::new(
            protocols,
            request_response::Config::default()
        );

        let _behavior = Behavior::<TestMessage>::new(
            kad,
            identify,
            request_response,
        );
    }

    #[tokio::test]
    async fn test_event_handling() {
        let peer_id = PeerId::random();
        let keypair = Keypair::generate_ed25519();
        let identify_event = IdentifyEvent::Received {
            peer_id,
            info: Info {
                public_key: keypair.public(),
                protocol_version: "".to_string(),
                agent_version: "".to_string(),
                listen_addrs: vec![],
                protocols: vec![],
                observed_addr: "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
            },
        };

        let event: Event<TestMessage> = identify_event.into();
        assert!(matches!(event, Event::Identify(_)));
    }
} 