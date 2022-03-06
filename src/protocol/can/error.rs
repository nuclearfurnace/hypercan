use std::{io, time::Duration};

use socketcan::CANSocketOpenError;
use socketcan_isotp::Error as IsoTpError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketBuildError {
    #[error("required field was not configured: {field_name}")]
    MissingRequiredField { field_name: &'static str },
    #[error("the specified socket was not found")]
    SocketNotFound,
    #[error("I/O error while building the socket: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
}

impl From<IsoTpError> for SocketBuildError {
    fn from(ie: IsoTpError) -> Self {
        match ie {
            IsoTpError::Lookup { .. } => SocketBuildError::SocketNotFound,
            IsoTpError::Io { source } => SocketBuildError::Io { source },
        }
    }
}

impl From<CANSocketOpenError> for SocketBuildError {
    fn from(ie: CANSocketOpenError) -> Self {
        match ie {
            CANSocketOpenError::LookupError(_) => SocketBuildError::SocketNotFound,
            CANSocketOpenError::IOError(source) => SocketBuildError::Io { source },
        }
    }
}

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
}
