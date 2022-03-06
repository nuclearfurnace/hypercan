use async_trait::async_trait;
use can::identifier::StandardId;
use tracing::{error, info};

use crate::{common::config::CANParameters, protocol::can::isotp::ISOTPSocket};

use super::Operation;

#[derive(Default)]
pub struct ValidateSocket;

#[async_trait]
impl Operation for ValidateSocket {
    async fn run(self, can_parameters: CANParameters) {
        let socket_name = can_parameters.socket_name.clone();
        let socket_builder = ISOTPSocket::builder()
            .can_parameters(can_parameters)
            .source_id(StandardId::ZERO)
            .destination_id(StandardId::MAX);
        match socket_builder.build() {
            Err(e) => error!("Failed to open device: '{}': {}", socket_name, e),
            Ok(_) => info!("Successfully opened '{}'.", socket_name),
        }
    }
}
