use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use can::identifier::Id;
use socketcan::CANFrame;
use tokio::{pin, select, time::sleep};
use tracing::info;

use crate::{
    common::{
        config::CANParameters,
        error::{FieldIdentifier, FieldValue, InvalidResponse, InvalidResponseKind},
    },
    protocol::{
        can::{isotp::ISOTPSocket, raw::RawSocket},
        obd::services::current_data::decoder::AvailablePidDecoder,
    },
};

use super::{QueryError, CURRENT_DATA_SERVICE_ID};

struct AvailablePidRequest {
    query_pid: u8,
}

impl AvailablePidRequest {
    pub fn from_query_pid(query_pid: u8) -> Self {
        Self { query_pid }
    }

    pub fn payload(&self) -> [u8; 2] {
        [CURRENT_DATA_SERVICE_ID, self.query_pid]
    }

    pub fn parse_response(&self, data: &[u8]) -> Result<AvailablePidResponse, InvalidResponse> {
        // Has to be six bytes long:
        if data.len() != 6 {
            return Err(InvalidResponseKind::PayloadSize {
                actual: data.len(),
                expected: 6,
            }
            .into());
        }

        // First byte should be the service ID plus 0x40, and the second byte should be the query
        // PID, representing a response to our original query:
        let expected_service_id = CURRENT_DATA_SERVICE_ID + 0x40;
        if data[0] != expected_service_id {
            return Err(InvalidResponseKind::ServiceId {
                actual: data[0],
                expected: expected_service_id,
            }
            .into());
        }

        if data[1] != self.query_pid {
            return Err(InvalidResponseKind::FieldValue {
                field_id: FieldIdentifier::Position(1),
                actual: FieldValue::Byte(data[1]),
                expected: FieldValue::Byte(self.query_pid),
            }
            .into());
        }

        Ok(AvailablePidResponse {
            offset: self.query_pid,
            data: [data[2], data[3], data[4], data[5]],
        })
    }
}

struct AvailablePidResponse {
    data: [u8; 4],
    offset: u8,
}

impl AvailablePidResponse {
    pub fn offset(&self) -> u8 {
        self.offset
    }

    pub fn data(&self) -> [u8; 4] {
        self.data
    }
}

pub struct CurrentDataService {
    can_parameters: CANParameters,
}

impl CurrentDataService {
    pub fn new(can_parameters: CANParameters) -> Self {
        Self { can_parameters }
    }

    pub async fn query_available_pids(&mut self) -> Result<HashMap<Id, Vec<u8>>, QueryError> {
        let addressing = self.can_parameters.addressing;

        // We first issue a broadcast request to see what ECUs are willing to respond to us, and we
        // do that for a few hundred milliseconds to give them all a chance to transmit.  Once we
        // have a list of responding identifiers, we run through the normal serialized query process
        // for each of them individually.
        let mut raw_socket = RawSocket::builder()
            .can_parameters(self.can_parameters.clone())
            .source_id_filter(addressing.obd_response_address_filter())
            .build()?;

        // We need to craft our payload manually since we aren't using an ISO-TP socket which adds
        // the length byte for us automatically.
        //
        // TODO: Make this better via `can`, ideally.
        let broadcast_address = addressing.obd_broadcast_address();
        let broadcast_request = AvailablePidRequest::from_query_pid(0);
        let request_payload = broadcast_request.payload();
        let request_payload = &[0x02, request_payload[0], request_payload[1]];
        let request_frame =
            CANFrame::new(broadcast_address.id().as_raw(), request_payload, false, false)
                .expect("should never fail to construct broadcast request frame");

        info!(
            "Searching for devices via broadcast address {}...",
            broadcast_address
        );
        raw_socket.write(request_frame).await?;

        let listen_timeout = sleep(Duration::from_secs(1));
        pin!(listen_timeout);

        let mut response_ids = HashSet::new();
        loop {
            select! {
                // Stop listening for responses at this point.
                _ = &mut listen_timeout => break,

                // We got a response, so just keep track of the identifier.  We don't care about the
                // data at this point since we'll grab that after.
                result = raw_socket.read() => {
                    let frame = result?;
                    let id = frame.id();
                    match addressing.get_diagnostic_response_id(id) {
                        Some(id) => {
                            response_ids.insert(id);
                        },
                        None => panic!("shouldn't have response with ID that can't be converted"),
                    }
                },
            }
        }

        info!(
            "Discovered {} potential device(s) to query.  Enumerating...",
            response_ids.len()
        );

        let mut available_pids = HashMap::new();
        for response_id in response_ids {
            let request_id = response_id.into_request_address();

            info!("Querying device at {}...", request_id);

            let mut socket = ISOTPSocket::builder()
                .can_parameters(self.can_parameters.clone())
                .source_id(response_id)
                .destination_id(request_id)
                .build()?;

            let mut decoder = AvailablePidDecoder::new();
            while let Some(query_pid) = decoder.next_query_pid() {
                // Build the request and send it.
                let request = AvailablePidRequest::from_query_pid(query_pid);
                let payload = request.payload();
                socket.write(&payload[..]).await?;

                // Wait for a response and attempt to validate it against the request we just sent.
                let raw_response = socket.read().await?;
                let response = request.parse_response(&raw_response)?;

                // Integrate this response and potentially query the next query PID:
                decoder.integrate_response(response.offset(), response.data());
            }

            available_pids.insert(request_id.id(), decoder.into_available_pids());
        }

        Ok(available_pids)
    }
}
