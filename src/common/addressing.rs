// TODO: this could maybe live in `can`, as like `can::identifier::obd` or something, since they are
// standardized so it's not like we'd be including totally random stuff.  might also want dedicated
// types that wrap the IDs so we can have a more type-driven approach to target/source addresses
// when using physical addressing, since right now it's ugly/gross to do the back and forth.

use can::identifier::{ExtendedId, Id, StandardId};
use clap::ArgEnum;
use socketcan::CANFilter;

const OBD_BROADCAST_ADDRESS_STANDARD: u16 = 0x7DF;
const OBD_BROADCAST_ADDRESS_EXTENDED: u32 = 0x18DB33F1;
const OBD_RESPONSE_ADDRESS_START_STANDARD: u16 = 0x7E8;
const OBD_RESPONSE_ADDRESS_END_STANDARD: u16 = 0x7EF;
const OBD_RESPONSE_ADDRESS_START_EXTENDED: u32 = 0x18DAF101;
const OBD_RESPONSE_ADDRESS_END_EXTENDED: u32 = 0x18DAF108;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum Addressing {
    Standard,
    Extended,
}

impl Addressing {
    pub fn is_standard(&self) -> bool {
        matches!(self, Self::Standard)
    }

    pub fn obd_broadcast_address(&self) -> Id {
        match self {
            Self::Standard => StandardId::new(OBD_BROADCAST_ADDRESS_STANDARD)
                .unwrap()
                .into(),
            Self::Extended => ExtendedId::new(OBD_BROADCAST_ADDRESS_EXTENDED)
                .unwrap()
                .into(),
        }
    }

    pub fn obd_response_address_filter(&self) -> CANFilter {
        match self {
            Self::Standard => CANFilter::new(
                OBD_RESPONSE_ADDRESS_START_STANDARD as u32,
                OBD_RESPONSE_ADDRESS_START_STANDARD as u32,
            )
            .expect("cannot fail to construct"),
            Self::Extended => CANFilter::new(
                OBD_RESPONSE_ADDRESS_START_EXTENDED,
                OBD_RESPONSE_ADDRESS_START_EXTENDED,
            )
            .expect("cannot fail to construct"),
        }
    }

    pub fn get_physical_obd_request_id_from_response(&self, id: u32) -> Option<Id> {
        // Try to parse the ID as either a standard or extended ID, and calculate the correct
        // request ID for the given response ID if it is in fact a valid response ID.
        if self.is_standard() {
            StandardId::new(id as u16)
                .and_then(|id| {
                    let raw_id = id.as_raw();
                    if raw_id <= OBD_RESPONSE_ADDRESS_END_STANDARD
                        && raw_id >= OBD_RESPONSE_ADDRESS_START_STANDARD
                    {
                        let request_id = raw_id - 8;
                        StandardId::new(request_id)
                    } else {
                        None
                    }
                })
                .map(Into::into)
        } else {
            ExtendedId::new(id)
                .and_then(|id| {
                    let raw_id = id.as_raw();
                    if raw_id <= OBD_RESPONSE_ADDRESS_END_EXTENDED
                        && raw_id >= OBD_RESPONSE_ADDRESS_START_EXTENDED
                    {
                        let request_id = (raw_id & 0xFFFF0000)
                            | (raw_id & 0x0000FF00) >> 8
                            | (raw_id & 0x000000FF) << 8;
                        ExtendedId::new(request_id)
                    } else {
                        None
                    }
                })
                .map(Into::into)
        }
    }

    pub fn get_physical_obd_response_id_from_request_id(&self, id: Id) -> Option<Id> {
        match id {
            Id::Standard(sid) => StandardId::new(sid.as_raw() + 8).map(Into::into),
            Id::Extended(eid) => {
                let raw_id = eid.as_raw();
                let response_id =
                    (raw_id & 0xFFFF0000) | (raw_id & 0x0000FF00) >> 8 | (raw_id & 0x000000FF) << 8;
                ExtendedId::new(response_id).map(Into::into)
            }
        }
    }
}
