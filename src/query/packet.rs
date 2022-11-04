use crate::errors::QueryProtocolError;

pub(super) enum QueryPacketDirection {
    Serverbound,
    Clientbound,
}

pub(super) enum QueryPacketType {
    Handshake,
    Stat,
}

impl From<QueryPacketType> for u8 {
    fn from(packet_type: QueryPacketType) -> Self {
        match packet_type {
            QueryPacketType::Handshake => 9,
            QueryPacketType::Stat => 0,
        }
    }
}

impl TryFrom<u8> for QueryPacketType {
    type Error = QueryProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            9 => Ok(Self::Handshake),
            0 => Ok(Self::Stat),
            _ => Err(QueryProtocolError::InvalidPacketType),
        }
    }
}

pub(super) struct QueryPacket {}
