use bytes::Bytes;
use std::ops::Deref;

use crate::errors::MinecraftProtocolError;

pub(crate) const SEGMENT_BITS: u8 = 0x7f; // 0111 1111
pub(crate) const CONTINUE_BIT: u8 = 0x80; // 1000 0000

pub(crate) struct VarInt {
    bytes: Bytes,
}

impl VarInt {
    pub(crate) fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }

    pub(crate) fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        let mut value = (value as u64) & 0xffff_ffff;
        let mut buffer = vec![];

        loop {
            let temp = (value & SEGMENT_BITS as u64) as u8;
            value >>= 7;

            if value != 0 {
                buffer.push(temp | CONTINUE_BIT);
            } else {
                buffer.push(temp);
            }

            if value == 0 {
                break;
            }
        }

        Self {
            bytes: Bytes::from(Box::from(buffer)),
        }
    }
}

impl TryInto<i32> for VarInt {
    type Error = MinecraftProtocolError;

    fn try_into(self) -> Result<i32, Self::Error> {
        let mut value: i32 = 0;
        let mut position = 0;

        for current_byte in self.bytes {
            value |= ((current_byte & SEGMENT_BITS) as i32) << position;

            if current_byte & CONTINUE_BIT == 0 {
                return Ok(value);
            }

            position += 7;
            if position >= 32 {
                return Err(MinecraftProtocolError::InvalidVarInt);
            }
        }

        unreachable!();
    }
}

impl From<VarInt> for Bytes {
    fn from(varint: VarInt) -> Self {
        varint.bytes
    }
}

impl Deref for VarInt {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use crate::errors::MinecraftProtocolError;
    use bytes::Bytes;
    use std::collections::HashMap;

    #[test]
    fn test_into_varint() {
        let cases = HashMap::from([
            (0, b"\x00".as_slice()),
            (1, b"\x01"),
            (2, b"\x02"),
            (127, b"\x7f"),
            (128, b"\x80\x01"),
            (255, b"\xff\x01"),
            (25565, b"\xdd\xc7\x01"),
            (2097151, b"\xff\xff\x7f"),
            (i32::MAX, b"\xff\xff\xff\xff\x07"),
            (-1, b"\xff\xff\xff\xff\x0f"),
            (i32::MIN, b"\x80\x80\x80\x80\x08"),
        ]);

        for (k, v) in cases {
            let varint: VarInt = k.into();
            assert_eq!(varint.bytes.len(), v.len());
            assert_eq!(varint.bytes, v);
        }
    }

    #[test]
    fn test_from_varint() {
        let cases = HashMap::from([
            (0, b"\x00".as_slice()),
            (1, b"\x01"),
            (2, b"\x02"),
            (127, b"\x7f"),
            (128, b"\x80\x01"),
            (255, b"\xff\x01"),
            (25565, b"\xdd\xc7\x01"),
            (2097151, b"\xff\xff\x7f"),
            (i32::MAX, b"\xff\xff\xff\xff\x07"),
            (-1, b"\xff\xff\xff\xff\x0f"),
            (i32::MIN, b"\x80\x80\x80\x80\x08"),
        ])
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                VarInt {
                    bytes: Bytes::from(v),
                },
            )
        })
        .collect::<HashMap<_, _>>();

        for (k, v) in cases {
            let x: Result<i32, _> = v.try_into();

            if let Err(MinecraftProtocolError::InvalidVarInt) = x {
                panic!("{k} as VarInt returned Err during conversion");
            }
            let x = x.unwrap();

            assert_eq!(x, k);
        }
    }
}
