//! Get the status of a server using the [Server List Ping](https://wiki.vg/Server_List_Ping) protocol.
//! See documentation for [`status`] for more information.

pub mod data;
mod packet;

use crate::{
    errors::MinecraftProtocolError,
    socket::{ReadWriteMinecraftString, ReadWriteVarInt},
    varint::VarInt,
};
use tokio::{
    io::{self, AsyncWriteExt, Interest},
    net::TcpStream,
};

use self::{
    data::StatusResponse,
    packet::{Packet, PacketId},
};

/// Ping the server for information following the [Server List Ping](https://wiki.vg/Server_List_Ping) protocol.
///
/// # Arguments
/// * `host` - A string slice that holds the hostname of the server to connect to.
/// * `port` - The port to connect to on that server.
///
/// # Errors
/// Returns `Err` if there was a network issue or the server sent invalid data.
///
/// # Examples
/// ```
/// use mc_query::status;
/// use tokio::io::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let data = status("mc.hypixel.net", 25565).await?;
///     println!("{data:#?}");
///
///     Ok(())
/// }
/// ```
pub async fn status(host: &str, port: u16) -> io::Result<StatusResponse> {
    let mut socket = TcpStream::connect(format!("{host}:{port}")).await?;

    socket
        .ready(Interest::READABLE | Interest::WRITABLE)
        .await?;

    // handshake packet
    // https://wiki.vg/Server_List_Ping#Handshake
    let handshake = Packet::builder(PacketId::Handshake)
        .add_varint(&VarInt::from(-1))
        .add_string(host)
        .add_u16(port)
        .add_varint(&VarInt::from(PacketId::Status))
        .build();

    socket.write_all(&handshake.bytes()).await?;

    // status request packet
    // https://wiki.vg/Server_List_Ping#Status_Request
    let status_request = Packet::builder(PacketId::Handshake).build();
    socket.write_all(&status_request.bytes()).await?;

    // listen to status response
    // https://wiki.vg/Server_List_Ping#Status_Response
    let _len = socket.read_varint().await?;
    let id = socket.read_varint().await?;

    if id != 0 {
        return Err(MinecraftProtocolError::InvalidStatusResponse.into());
    }

    let data = socket.read_mc_string().await?;
    socket.shutdown().await?;

    serde_json::from_str::<StatusResponse>(&data)
        .map_err(|_| MinecraftProtocolError::InvalidStatusResponse.into())
}

#[cfg(test)]
mod tests {
    use super::status;
    use tokio::io::Result;

    #[tokio::test]
    async fn test_hypixel_status() -> Result<()> {
        let data = status("mc.hypixel.net", 25565).await?;
        println!("{data:#?}");

        Ok(())
    }

    #[tokio::test]
    async fn test_local_status() -> Result<()> {
        let data = status("localhost", 25565).await?;
        println!("{data:#?}");

        Ok(())
    }
}
