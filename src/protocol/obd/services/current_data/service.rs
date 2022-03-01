use std::time::Duration;

use crate::{
    common::{
        addressing::Addressing,
        error::{FieldIdentifier, FieldValue, InvalidResponse, InvalidResponseKind},
    },
    protocol::isotp::ISOTPSocket,
};

use super::{decoder::AvailablePidDecoder, QueryError, CURRENT_DATA_SERVICE_ID};

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
            data: [data[0], data[1], data[2], data[3]],
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
    socket_name: String,
    addressing: Addressing,
}

impl CurrentDataService {
    pub fn new(socket_name: String, addressing: Addressing) -> Self {
        Self {
            socket_name,
            addressing,
        }
    }

    pub async fn query_available_pids(&mut self) -> Result<Vec<u8>, QueryError> {
        let mut socket = ISOTPSocket::builder()
            .socket_name(self.socket_name.clone())
            .source_id(self.addressing.obd_response_address_ecu())
            .destination_id(self.addressing.obd_broadcast_address())
            .read_timeout(Duration::from_secs(2))
            .write_timeout(Duration::from_secs(2))
            .use_isotp_frame_padding()
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

        Ok(decoder.into_available_pids())
    }
}
