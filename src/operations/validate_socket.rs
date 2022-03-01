use async_trait::async_trait;
use embedded_can::StandardId;
use tracing::{error, info};

use crate::protocol::isotp::ISOTPSocket;

use super::Operation;

#[derive(Default)]
pub struct ValidateSocket;

#[async_trait]
impl Operation for ValidateSocket {
    async fn run(self, socket_name: String) {
        let socket_builder = ISOTPSocket::builder()
            .socket_name(socket_name.clone())
            .source_id(StandardId::ZERO)
            .destination_id(StandardId::MAX);
        match socket_builder.build() {
            Err(e) => error!("Failed to open device: '{}': {}", socket_name, e),
            Ok(_) => info!("Successfully opened '{}'.", socket_name),
        }
    }
}
