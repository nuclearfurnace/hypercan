use async_trait::async_trait;

use crate::common::config::{AppConfig, Command};

use self::{query_available_pids::QueryAvailablePIDs, validate_socket::ValidateSocket};

mod query_available_pids;
mod validate_socket;

#[async_trait]
pub trait Operation {
    async fn run(self, socket_name: String);
}

pub async fn run_operation(config: &AppConfig) {
    match config.command() {
        Command::ValidateSocket => {
            let validate_socket = ValidateSocket::default();
            validate_socket.run(config.socket()).await
        }
        Command::QueryAvailablePIDs { extended } => {
            let query_available_pids = QueryAvailablePIDs::new(extended);
            query_available_pids.run(config.socket()).await
        }
    }
}
