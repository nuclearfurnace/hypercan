use core::fmt;

use thiserror::Error;

#[derive(Debug)]
pub enum FieldIdentifier {
    Position(usize),
    Logical(String),
}

impl FieldIdentifier {
    pub fn human_readable(&self) -> String {
        match self {
            Self::Position(pos) => format!("at position {}", pos),
            Self::Logical(name) => format!("in '{}'", name),
        }
    }
}

#[derive(Debug)]
pub enum FieldValue {
    Byte(u8),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Byte(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Error)]
pub enum InvalidResponseKind {
    #[error("expected payload to be {expected} bytes, got {actual} bytes instead")]
    PayloadSize { actual: usize, expected: usize },
    #[error("expected service ID to be {expected}, got {actual} instead")]
    ServiceId { actual: u8, expected: u8 },
    #[error(
		"expected value {} to be '{expected}', got '{actual}' instead",
		FieldIdentifier::human_readable(.field_id)
	)]
    FieldValue {
        field_id: FieldIdentifier,
        actual: FieldValue,
        expected: FieldValue,
    },
}

#[derive(Debug, Error)]
#[error("invalid response: {0}")]
pub struct InvalidResponse(#[from] InvalidResponseKind);
