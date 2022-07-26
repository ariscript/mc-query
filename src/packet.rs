use crate::{errors::MinecraftProtocolError, varint::VarInt};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub(crate) enum PacketId {
    Handshake = 0,
    Status = 1,
}

impl From<PacketId> for u8 {
    fn from(id: PacketId) -> Self {
        match id {
            PacketId::Handshake => 0,
            PacketId::Status => 1,
        }
    }
}

impl TryFrom<u8> for PacketId {
    type Error = MinecraftProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Handshake),
            1 => Ok(Self::Status),
            _ => Err(MinecraftProtocolError::InvalidState),
        }
    }
}

impl From<PacketId> for VarInt {
    fn from(id: PacketId) -> Self {
        let number: u8 = id.into();
        VarInt::from(number as i32)
    }
}

#[derive(Debug)]
pub(crate) struct Packet {
    id: u8,
    payload: Bytes,
}

impl Packet {
    pub fn builder(id: PacketId) -> PacketBuilder {
        PacketBuilder::new(id)
    }

    pub fn bytes(self) -> Bytes {
        self.into()
    }
}

impl From<Packet> for Bytes {
    fn from(packet: Packet) -> Self {
        let len: i32 = (VarInt::from(packet.id as i32).bytes().len() + packet.payload.len()) as i32;
        let mut bytes = BytesMut::new();

        bytes.extend_from_slice(&VarInt::from(len).bytes());
        bytes.put_u8(packet.id);
        bytes.extend_from_slice(&packet.payload);

        bytes.freeze()
    }
}

#[derive(Debug)]
pub(crate) struct PacketBuilder {
    id: PacketId,
    bytes: BytesMut,
}

impl PacketBuilder {
    pub fn new(id: PacketId) -> Self {
        Self {
            id,
            bytes: BytesMut::new(),
        }
    }

    pub fn add_varint(mut self, varint: &VarInt) -> Self {
        self.bytes.extend_from_slice(varint);
        self
    }

    pub fn add_string(self, string: &str) -> Self {
        let mut inst = self.add_varint(&VarInt::from(string.len() as i32));
        inst.bytes.put(string.as_bytes());
        inst
    }

    pub fn add_u16(mut self, short: u16) -> Self {
        self.bytes.put_u16(short);
        self
    }

    pub fn build(self) -> Packet {
        Packet {
            id: self.id.into(),
            payload: self.bytes.freeze(),
        }
    }
}
