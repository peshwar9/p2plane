use libp2p::{
    Multiaddr, PeerId,
    swarm::NetworkBehaviour,
    request_response::{
        cbor::Behaviour as RequestResponse,
        Event as RequestResponseEvent,
        ResponseChannel,
    },
    kad::{
        store::MemoryStore,
        Behaviour as Kademlia,
        Event as KadEvent,
    },
    identify::{
        Behaviour as Identify,
        Event as IdentifyEvent,
    },
};
use crate::p2plane::traits::Message;

// Define Event enum before the Behavior struct
#[derive(Debug)]
pub enum Event<M> {
    Kad(KadEvent),
    Identify(IdentifyEvent),
    RequestResponse(RequestResponseEvent<M, M>),
}

// Implement From traits for each event type
impl<M> From<KadEvent> for Event<M> {
    fn from(event: KadEvent) -> Self {
        Event::Kad(event)
    }
}

impl<M> From<IdentifyEvent> for Event<M> {
    fn from(event: IdentifyEvent) -> Self {
        Event::Identify(event)
    }
}

impl<M> From<RequestResponseEvent<M, M>> for Event<M> {
    fn from(event: RequestResponseEvent<M, M>) -> Self {
        Event::RequestResponse(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event<M>")]
pub struct Behavior<M: Message> {
    pub kad: Kademlia<MemoryStore>,
    pub identify: Identify,
    pub request_response: RequestResponse<M, M>,
}

impl<M: Message> Behavior<M> {
    pub fn new(
        kad: Kademlia<MemoryStore>,
        identify: Identify,
        request_response: RequestResponse<M, M>,
    ) -> Self {
        Self {
            kad,
            identify,
            request_response,
        }
    }
}
