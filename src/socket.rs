use crate::varint::{VarInt, CONTINUE_BIT};
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};

/// Trait to allow for reading and writing `VarInt`s from the socket.
///
/// The type is specified [in wiki.vg](https://wiki.vg/Protocol#VarInt_and_VarLong).
#[async_trait]
pub(crate) trait ReadWriteVarInt {
    /// Read a [VarInt] from the socket.
    /// Returns the parsed value as [i32] in a [Result].
    async fn read_varint(&mut self) -> Result<i32>;

    /// Write a [VarInt] to the socket.
    /// Writes the given integer in the form of a [VarInt].
    /// Returns whether the operation was successful in a [Result].
    async fn write_varint(&mut self, value: i32) -> Result<()>;
}

/// Trait to allow for reading and writing `VarLong`s from the socket.
///
/// The type is specified [in wiki.vg](https://wiki.vg/Protocol#VarInt_and_VarLong).
#[async_trait]
pub(crate) trait ReadWriteVarLong {
    /// Read a `VarLong` from the socket.
    /// Returns the parsed value as [i64] in a [Result].
    async fn read_varlong(&mut self) -> Result<i64>;

    /// Write a `VarLong` to the socket.
    /// Writes the given integer in the form of a `VarLong`.
    /// Returns whether the operation was successful in a [Result].
    async fn write_varlong(&mut self, value: i64) -> Result<()>;
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

    /// Write a [str] to the socket.
    /// Returns whether the operation was successful in a [Result].
    async fn write_mc_string(&mut self, value: &str) -> Result<()>;
}

#[async_trait]
impl<T> ReadWriteVarInt for T
where
    T: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync,
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

    async fn write_varint(&mut self, value: i32) -> Result<()> {
        self.write_all(&VarInt::from(value)).await
    }
}

#[async_trait]
impl<T> ReadWriteMinecraftString for T
where
    T: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync,
{
    async fn read_mc_string(&mut self) -> Result<String> {
        let len = self.read_varint().await?;
        let mut buffer = vec![0; len as usize];
        self.read_exact(&mut buffer).await?;

        String::from_utf8(buffer).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }

    async fn write_mc_string(&mut self, value: &str) -> Result<()> {
        self.write_varint(value.len() as i32).await?;
        self.write_all(value.as_bytes()).await?;

        Ok(())
    }
}
