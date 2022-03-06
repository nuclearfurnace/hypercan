pub(self) mod decoder;

mod service;
use thiserror::Error;

use crate::{
    common::error::InvalidResponse,
    protocol::can::error::{SocketBuildError, SocketError},
};

pub use self::service::CurrentDataService;

const CURRENT_DATA_SERVICE_ID: u8 = 0x01;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("failed to initialize socket: {0}")]
    Initialization(#[from] SocketBuildError),
    #[error("socket error while querying service: {0}")]
    Io(#[from] SocketError),
    #[error(transparent)]
    InvalidResponse(#[from] InvalidResponse),
}
