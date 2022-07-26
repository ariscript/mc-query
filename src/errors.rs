//! All the errors defined by this crate.

use std::io::{self, ErrorKind};
use thiserror::Error;

/// An error from the Minecraft networking protocol.
#[derive(Error, Debug)]
pub enum MinecraftProtocolError {
    /// VarInt data was invalid according to the spec.
    #[error("invalid varint data")]
    InvalidVarInt,

    /// Recieved invalid state information from the server.
    #[error("invalid state")]
    InvalidState,

    /// Recieved incorrectly formatted status response from the server.
    #[error("invalid status response")]
    InvalidStatusResponse,
}

impl From<MinecraftProtocolError> for io::Error {
    fn from(err: MinecraftProtocolError) -> Self {
        io::Error::new(ErrorKind::InvalidData, err)
    }
}
