use async_trait::async_trait;
use tracing::{error, info};

use super::Operation;
use crate::{common::addressing::Addressing, protocol::obd::services::CurrentDataService};

pub struct QueryAvailablePIDs {
    addressing: Addressing,
}

impl QueryAvailablePIDs {
    pub fn new(extended: bool) -> Self {
        let addressing = if extended {
            Addressing::Extended
        } else {
            Addressing::Standard
        };
        Self { addressing }
    }
}

#[async_trait]
impl Operation for QueryAvailablePIDs {
    async fn run(self, socket_name: String) {
        let mut current_data_service = CurrentDataService::new(socket_name, self.addressing);
        match current_data_service.query_available_pids().await {
            Ok(pids) => {
                if pids.is_empty() {
                    info!("No available PIDs found.")
                } else {
                    info!("ECU reported {} available PIDs: {:02X?}", pids.len(), pids)
                }
            }
            Err(e) => error!("Error occurred while querying for available PIDs: {}", e),
        }
    }
}
