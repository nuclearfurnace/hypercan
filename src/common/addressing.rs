// TODO: this could maybe live in `can`, as like `can::identifier::obd` or something, since they are
// standardized so it's not like we'd be including totally random stuff.  might also want dedicated
// types that wrap the IDs so we can have a more type-driven approach to target/source addresses
// when using physical addressing, since right now it's ugly/gross to do the back and forth.

use can::identifier::{obd::{DiagnosticBroadcastAddress, DiagnosticResponseFilter, DiagnosticResponseAddress}, StandardId, ExtendedId};
use clap::ArgEnum;
use socketcan::CANFilter;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum Addressing {
    Standard,
    Extended,
}

impl Addressing {
    pub fn obd_broadcast_address(&self) -> DiagnosticBroadcastAddress {
        match self {
            Self::Standard => DiagnosticBroadcastAddress::standard(),
            Self::Extended => DiagnosticBroadcastAddress::extended(),
        }
    }

    pub fn obd_response_address_filter(&self) -> CANFilter {
        match self {
            Self::Standard => DiagnosticResponseFilter::standard().into(),
            Self::Extended => DiagnosticResponseFilter::extended().into(),
        }
    }

    pub fn get_diagnostic_response_id(&self, id: u32) -> Option<DiagnosticResponseAddress> {
        match self {
            Self::Standard => id.try_into().ok()
                .and_then(StandardId::new)
                .and_then(|id| DiagnosticResponseAddress::from_id(id.into())),
            Self::Extended => ExtendedId::new(id)
                .and_then(|id| DiagnosticResponseAddress::from_id(id.into())),
        }
    }
}
