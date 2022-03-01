use embedded_can::{ExtendedId, Id, StandardId};

const OBD_BROADCAST_ADDRESS_STANDARD: u16 = 0x7DF;
const OBD_BROADCAST_ADDRESS_EXTENDED: u32 = 0x18DB33F1;
const OBD_RESPONSE_ADDRESS_START_STANDARD: u16 = 0x7E8;
const OBD_RESPONSE_ADDRESS_END_STANDARD: u16 = 0x7EF;
const OBD_RESPONSE_ADDRESS_START_EXTENDED: u32 = 0x18DAF101;
const OBD_RESPONSE_ADDRESS_END_EXTENDED: u32 = 0x18DAF108;
const OBD_RESPONSE_ADDRESS_ECU_STANDARD: u16 = OBD_RESPONSE_ADDRESS_START_STANDARD;
const OBD_RESPONSE_ADDRESS_ECU_EXTENDED: u32 = OBD_RESPONSE_ADDRESS_START_EXTENDED;

#[derive(Clone, Copy, Debug)]
pub enum Addressing {
    Standard,
    Extended,
}

impl Addressing {
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

    pub fn obd_response_addresses(&self) -> Vec<Id> {
        match self {
            Self::Standard => (OBD_RESPONSE_ADDRESS_START_STANDARD
                ..OBD_RESPONSE_ADDRESS_END_STANDARD)
                .map(|id| StandardId::new(id).unwrap())
                .map(Into::into)
                .collect(),
            Self::Extended => (OBD_RESPONSE_ADDRESS_START_EXTENDED
                ..OBD_RESPONSE_ADDRESS_END_EXTENDED)
                .map(|id| ExtendedId::new(id).unwrap())
                .map(Into::into)
                .collect(),
        }
    }

    pub fn obd_response_address_ecu(&self) -> Id {
        match self {
            Self::Standard => StandardId::new(OBD_RESPONSE_ADDRESS_ECU_STANDARD)
                .unwrap()
                .into(),
            Self::Extended => ExtendedId::new(OBD_RESPONSE_ADDRESS_ECU_EXTENDED)
                .unwrap()
                .into(),
        }
    }
}
