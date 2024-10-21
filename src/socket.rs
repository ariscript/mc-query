use crate::varint::{VarInt, CONTINUE_BIT};
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, Result};

/// Trait to allow for reading and writing `VarInt`s from the socket.
///
/// The type is specified [in wiki.vg](https://wiki.vg/Protocol#VarInt_and_VarLong).
#[async_trait]
pub(crate) trait ReadWriteVarInt {
    /// Read a [VarInt] from the socket.
    /// Returns the parsed value as [i32] in a [Result].
    async fn read_varint(&mut self) -> Result<i32>;
}

/// Trait to allow for reading and writing strings from the socket.
///
/// The format for strings is specified [in this table in wiki.vg](https://wiki.vg/Protocol#Data_types).
/// It is a UTF-8 string prefixed with its size in bytes as a [`VarInt`].
#[async_trait]
pub(crate) trait ReadWriteMinecraftString {
    /// Read a [String] from the socket.
    /// Returns the parsed value recieved from the socket in a [Result].
    async fn read_mc_string(&mut self) -> Result<String>;
}

#[async_trait]
impl<T> ReadWriteVarInt for T
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn read_varint(&mut self) -> Result<i32> {
        let mut bytes = BytesMut::with_capacity(5);

        loop {
            let current = self.read_u8().await?;
            bytes.put_u8(current);

            if current & CONTINUE_BIT == 0 {
                break;
            }
        }

        VarInt::new(bytes.freeze())
            .try_into()
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}

#[async_trait]
impl<T> ReadWriteMinecraftString for T
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn read_mc_string(&mut self) -> Result<String> {
        let len = self.read_varint().await?;
        let mut buffer = vec![0; len as usize];
        self.read_exact(&mut buffer).await?;

        String::from_utf8(buffer).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}