use crate::errors::RconProtocolError;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::mem::size_of;

#[derive(Debug)]
pub(super) enum RconPacketType {
    Response,
    Login,
    RunCommand,
}

impl From<RconPacketType> for i32 {
    fn from(packet_type: RconPacketType) -> Self {
        match packet_type {
            RconPacketType::Response => 0,
            RconPacketType::RunCommand => 2,
            RconPacketType::Login => 3,
        }
    }
}

impl TryFrom<i32> for RconPacketType {
    type Error = RconProtocolError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RconPacketType::Response),
            2 => Ok(RconPacketType::RunCommand),
            3 => Ok(RconPacketType::Login),
            _ => Err(RconProtocolError::InvalidPacketType),
        }
    }
}

#[derive(Debug)]
pub(super) struct RconPacket {
    pub request_id: i32,
    pub packet_type: RconPacketType,
    pub payload: String,
}

impl RconPacket {
    pub fn new(
        request_id: i32,
        packet_type: RconPacketType,
        payload: String,
    ) -> Result<Self, RconProtocolError> {
        if !payload.is_ascii() {
            return Err(RconProtocolError::NonAsciiPayload);
        }

        if payload.len() > 1446 {}

        Ok(Self {
            request_id,
            packet_type,
            payload,
        })
    }

    pub fn bytes(self) -> Bytes {
        Bytes::from(self)
    }
}

impl TryFrom<Bytes> for RconPacket {
    type Error = RconProtocolError;

    fn try_from(mut bytes: Bytes) -> Result<Self, Self::Error> {
        let len = bytes.get_i32_le(); // length of remaining packet (not including this integer)
        let request_id = bytes.get_i32_le();
        let packet_type = bytes.get_i32_le();

        let mut payload = "".to_string();

        loop {
            let current = bytes.get_u8();
            if current == 0 {
                // null terminated ASCII string, so stop reading here
                break;
            }

            payload.push(current as char);
        }

        // if the payload is already normal ASCII (without 0xa7), no need to
        // check each character to be ASCII or 0xa7
        if !payload.is_ascii() {
            for c in payload.chars() {
                // 0xa7 is an acceptable (though non-ASCII) character
                if !c.is_ascii() && (c as u8) != 0xa7 {
                    return Err(RconProtocolError::NonAsciiPayload);
                }
            }
        }

        let pad = bytes.get_u8(); // there must be a remaining 0 byte as padding
        if pad != 0 {
            return Err(RconProtocolError::InvalidRconResponse);
        }

        // validate if the lengths match
        if get_remaining_length(&payload) != len {
            return Err(RconProtocolError::InvalidRconResponse);
        }

        Self::new(request_id, packet_type.try_into()?, payload)
    }
}

impl From<RconPacket> for Bytes {
    fn from(packet: RconPacket) -> Self {
        let len = get_remaining_length(&packet.payload);
        let packet_type: i32 = packet.packet_type.into();

        let mut bytes = BytesMut::new();

        bytes.put_i32_le(len);
        bytes.put_i32_le(packet.request_id);
        bytes.put_i32_le(packet_type);
        bytes.put(packet.payload.as_bytes());
        bytes.put_u16(0x00_00);

        bytes.freeze()
    }
}

/// Get the *remaining length* of the packet given its payload.
///
/// Remaining length here refers to the length of the packet in bytes excluding
/// the first four bytes which communicate this value. So it refers to the
/// length of the packet *after* the length field.
///
/// As the remainder of the packet is composed of two [i32]s (request ID and type),
/// the payload, and **TWO** 0 bytes (because rust strings are not null-terminated),
/// it is the size of two [i32]s + the length of the payload + 2.
fn get_remaining_length(payload: &str) -> i32 {
    (payload.len() + size_of::<i32>() * 2 + 2) as i32
}
