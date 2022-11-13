//! Implementation of the [Server List Ping](https://wiki.vg/Server_List_Ping) protocol

mod packet;

use self::packet::{Packet, PacketId};
use crate::{
    errors::MinecraftProtocolError,
    socket::{ReadWriteMinecraftString, ReadWriteVarInt},
    varint::VarInt,
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncWriteExt, Interest},
    net::TcpStream,
};

/// Response from the server with status information.
/// Represents [this JSON object](https://wiki.vg/Server_List_Ping#Status_Response)
/// to be serialized and deserialized.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Information about the game and protocol version.
    /// See [Version] for more information.
    pub version: Version,

    // Information about players on the server.
    /// See [Players] for more information.
    pub players: Players,

    /// The "motd" - message shown in the server list by the client.
    #[serde(rename = "description")]
    pub motd: ChatObject,

    /// URI to the server's favicon.
    pub favicon: String,

    /// Does the server preview chat?
    #[serde(rename = "previewsChat")]
    pub previews_chat: Option<bool>,
}

/// Struct that stores information about players on the server.
///
/// Not intended to be used directly, but only as a part of [StatusResponse].
#[derive(Debug, Serialize, Deserialize)]
pub struct Players {
    /// The maximum number of players allowed on the server.
    pub max: u32,

    /// The number of players currently online.
    pub online: u32,

    /// A listing of some online Players.
    /// See [Sample] for more information.
    pub sample: Option<Vec<Sample>>,
}

/// A player listed on the server's list ping information.
///
/// Not intended to be used directly, but only as a part of [StatusResponse].
#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    /// The player's username.
    pub name: String,

    /// The player's UUID.
    pub id: String,
}

/// Struct that stores version information about the server.
///
/// Not intended to be used directly, but only as a part of [StatusResponse].
#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    /// The game version (e.g: 1.19.1)
    pub name: String,
    /// The version of the [Protocol](https://wiki.vg/Protocol) being used.
    ///
    /// See [the wiki.vg page](https://wiki.vg/Protocol_version_numbers) for a
    /// reference on what versions these correspond to.
    pub protocol: u16,
}

/// The ChatObject, this is contained within the motd for a server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    Object(ChatComponentObject),
    Array(Vec<ChatObject>),
    JsonPrimitive(serde_json::Value),
}

/// A piece of a `ChatObject`
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatComponentObject {
    pub text: Option<String>,
    pub translate: Option<String>,
    pub keybind: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub font: Option<String>,
    pub color: Option<String>,
    pub insertion: Option<String>,
    #[serde(rename = "clickEvent")]
    pub click_event: Option<ChatClickEvent>,
    #[serde(rename = "hoverEvent")]
    pub hover_event: Option<ChatHoverEvent>,
    pub extra: Option<Vec<ChatObject>>,
}

/// ClickEvent data for a chat component
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatClickEvent {
    // These are not renamed on purpose.
    pub open_url: Option<String>,
    pub run_command: Option<String>,
    pub suggest_command: Option<String>,
    pub copy_to_clipboard: Option<String>,
}

/// HoverEvent data for a chat component
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHoverEvent {
    // These are not renamed on purpose.
    pub show_text: Option<Box<ChatObject>>,
    pub show_item: Option<String>,
    pub show_entity: Option<String>,
}

/// Ping the server for information following the [Server List Ping](https://wiki.vg/Server_List_Ping) protocol.
///
/// # Arguments
/// * `host` - A string slice that holds the hostname of the server to connect to.
/// * `port` - The port to connect to on that server.
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
    let status_packet_id: u8 = PacketId::Status.into();

    let handshake = Packet::builder(PacketId::Handshake)
        .add_varint(&VarInt::from(-1))
        .add_string(host)
        .add_u16(port)
        .add_varint(&VarInt::from(status_packet_id as i32))
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
}
