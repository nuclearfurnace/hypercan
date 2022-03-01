use tracing::trace;

const PID_QUERY_STRIDE: u8 = 32;

pub struct AvailablePidDecoder {
    pids: Vec<u8>,
    next_query_pid: Option<u8>,
}

impl AvailablePidDecoder {
    pub fn new() -> Self {
        Self {
            pids: Vec::new(),
            next_query_pid: Some(0),
        }
    }

    pub fn integrate_response(&mut self, offset: u8, data: [u8; 4]) {
        let should_continue = data[3] & 0x1 == 1;
        for i in 0..31 {
            let byte_idx = i as usize / 8;
            let bit_idx = 7 - (i as usize % 8);

            if data[byte_idx] & (1 << bit_idx) > 0 {
                trace!("pid query: {:02x}, data[{}]: {:08b}, bit: {}, present: true (actual pid: 0x{:02x})",
                    offset, byte_idx, data[byte_idx], bit_idx, offset + i + 1);
                self.pids.push(offset + i + 1);
            }
        }

        self.next_query_pid = if should_continue {
            Some(offset + PID_QUERY_STRIDE)
        } else {
            None
        };
    }

    pub fn next_query_pid(&self) -> Option<u8> {
        self.next_query_pid
    }

    pub fn into_available_pids(self) -> Vec<u8> {
        self.pids
    }
}
