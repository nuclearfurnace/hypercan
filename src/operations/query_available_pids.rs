use async_trait::async_trait;
use tracing::{error, info};

use super::Operation;
use crate::{common::config::CANParameters, protocol::obd::services::CurrentDataService};

#[derive(Default)]
pub struct QueryAvailablePIDs;

#[async_trait]
impl Operation for QueryAvailablePIDs {
    async fn run(self, can_parameters: CANParameters) {
        let mut current_data_service = CurrentDataService::new(can_parameters);
        match current_data_service.query_available_pids().await {
            Ok(pid_map) => {
                if pid_map.is_empty() {
                    info!("No available PIDs found.")
                } else {
                    for (id, pids) in pid_map {
                        info!(
                            "Discovered {} PIDs for response ID {}: {:02X?}",
                            pids.len(),
                            id,
                            &pids[..]
                        );
                    }
                }
            }
            Err(e) => error!("Error occurred while querying for available PIDs: {}", e),
        }
    }
}
