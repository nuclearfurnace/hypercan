use async_trait::async_trait;

use crate::common::config::{AppConfig, CANParameters, Command};

use self::{query_available_pids::QueryAvailablePIDs, validate_socket::ValidateSocket};

mod query_available_pids;
mod validate_socket;

#[async_trait]
pub trait Operation {
    async fn run(self, can_parameters: CANParameters);
}

pub async fn run_operation(config: &AppConfig) {
    match config.command() {
        Command::ValidateSocket => {
            let validate_socket = ValidateSocket::default();
            validate_socket.run(config.can_parameters()).await
        }
        Command::QueryAvailablePIDs => {
            let query_available_pids = QueryAvailablePIDs::default();
            query_available_pids.run(config.can_parameters()).await
        }
    }
}
