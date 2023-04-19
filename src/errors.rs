//! All the errors defined by this crate.

use std::io::{self, ErrorKind};
use thiserror::Error;

/// An error from the Minecraft networking protocol.
#[derive(Error, Debug)]
pub enum MinecraftProtocolError {
    /// VarInt data was invalid according to the spec.
    #[error("invalid varint data")]
    InvalidVarInt,

    /// Received invalid state information from the server.
    #[error("invalid state")]
    InvalidState,

    /// Received incorrectly formatted status response from the server.
    #[error("invalid status response")]
    InvalidStatusResponse,
}

impl From<MinecraftProtocolError> for io::Error {
    fn from(err: MinecraftProtocolError) -> Self {
        io::Error::new(ErrorKind::InvalidData, err)
    }
}

/// An error from the RCON protocol.
#[derive(Error, Debug)]
pub enum RconProtocolError {
    /// Received non-ASCII payload data from the server.
    ///
    /// Note: some servers (for example Craftbukkit for Minecraft 1.4.7) reply
    /// with the section sign (0xa7) as a prefix for the payload. This error
    /// will not be returned in that case.
    #[error("non-ascii payload")]
    NonAsciiPayload,

    /// Authentication failed. You probably entered the wrong RCON password.
    #[error("authentication failed")]
    AuthFailed,

    /// Invalid or unexpected packet type received from the server.
    #[error("invalid packet type")]
    InvalidPacketType,

    /// Other kind of invalid response as defined by the spec.
    #[error("invalid rcon response")]
    InvalidRconResponse,

    /// Payload too long.
    ///
    /// | Direction   | Payload Length limit |
    /// | ----------- | -------------------- |
    /// | Serverbound | 1446                 |
    /// | Clientbound | 4096                 |
    #[error("payload too long")]
    PayloadTooLong,

    /// Mismatch with the given request ID.
    ///
    /// Note: the server replies with a request ID of -1 in the case of an
    /// authentication failure. In that case, `AuthFailed` will be returned.
    /// This variant is returned if any *other* request ID was received.
    #[error("request id mismatch")]
    RequestIdMismatch,
}

impl From<RconProtocolError> for io::Error {
    fn from(err: RconProtocolError) -> Self {
        io::Error::new(ErrorKind::InvalidData, err)
    }
}

/// An error from the Query protocol.
#[derive(Error, Debug)]
pub enum QueryProtocolError {
    /// Received invalid packet type.
    /// Valid types are 9 for handshake, 0 for stat
    #[error("invalid packet type")]
    InvalidPacketType,

    /// Unexpected packet type.
    #[error("unexpected packet type")]
    UnexpectedPacketType,

    /// Mismatch with the generated session ID.
    #[error("session id mismatch")]
    SessionIdMismatch,

    /// Received invalid challenge token from server.
    #[error("invalid challenge token")]
    InvalidChallengeToken,

    /// Invalid integer.
    /// Did not receive valid characters to parse as an integer in the string
    #[error("cannot parse int")]
    CannotParseInt,

    /// Invalid UTF8.
    /// Did not receive valid UTF from the server when a string was expected
    #[error("invalid UTF-8")]
    InvalidUtf8,

    /// Invalid key/value section.
    /// Expecting something like [this](https://wiki.vg/Query#K.2C_V_section)
    #[error("invalid key/value section")]
    InvalidKeyValueSection,
}

impl From<QueryProtocolError> for io::Error {
    fn from(err: QueryProtocolError) -> Self {
        io::Error::new(ErrorKind::InvalidData, err)
    }
}
