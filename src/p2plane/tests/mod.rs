use crate::p2plane::traits::Message;
use serde::{Serialize, Deserialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TestMessage(pub String);

impl Message for TestMessage {
    fn protocol_id(&self) -> &'static str {
        "/test/1.0.0"
    }
}

#[cfg(test)]
mod behavior_tests;

#[cfg(test)]
mod network_tests;

#[cfg(test)]
mod peer_manager_tests;