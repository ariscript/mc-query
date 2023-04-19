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
    pub favicon: Option<String>,

    /// Does the server preview chat?
    #[serde(rename = "previewsChat")]
    pub previews_chat: Option<bool>,

    /// Does the server use signed chat messages?
    /// Only returned for servers post 1.19.1
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: Option<bool>,
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

/// Represents a chat object (the MOTD is sent as a chat object).
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatObject {
    /// An individual chat object
    Object(ChatComponentObject),

    /// Vector of multiple chat objects
    Array(Vec<ChatObject>),

    /// Unknown data - raw JSON
    JsonPrimitive(serde_json::Value),
}

/// A piece of a `ChatObject`
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatComponentObject {
    /// Text of the chat message
    pub text: Option<String>,

    /// Translation key if the message needs to pull from the language file.
    /// See [wiki.vg](https://wiki.vg/Chat#Translation_component)
    pub translate: Option<String>,

    /// Displays the keybind for the specified key, or the string itself if unknown.
    pub keybind: Option<String>,

    /// Should the text be rendered **bold**?
    pub bold: Option<bool>,

    /// Should the text be rendered *italic*?
    pub italic: Option<bool>,

    /// Should the text be rendered __underlined__?
    pub underlined: Option<bool>,

    /// Should the text be rendered as ~~strikethrough~~
    pub strikethrough: Option<bool>,

    /// Should the text be rendered as obfuscated?
    /// Switching randomly between characters of the same width
    pub obfuscated: Option<bool>,

    /// The font to use to render, comes in three options:
    /// * `minecraft:uniform` - Unicode font
    /// * `minecraft:alt` - enchanting table font
    /// * `minecraft:default` - font based on resource pack (1.16+)
    /// Any other value can be ignored
    pub font: Option<String>,

    /// The color to display the chat item in.
    /// Can be a [chat color](https://wiki.vg/Chat#Colors),
    /// [format code](https://wiki.vg/Chat#Styles),
    /// or any valid web color
    pub color: Option<String>,

    /// Text to insert into the chat box when shift-clicking this component
    pub insertion: Option<String>,

    /// Defines an event that occurs when this chat item is clicked
    #[serde(rename = "clickEvent")]
    pub click_event: Option<ChatClickEvent>,

    /// Defines an event that occurs when this chat item is hovered on
    #[serde(rename = "hoverEvent")]
    pub hover_event: Option<ChatHoverEvent>,

    /// Sibling components to this chat item.
    /// If present, will not be empty
    pub extra: Option<Vec<ChatObject>>,
}

/// ClickEvent data for a chat component
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatClickEvent {
    // These are not renamed on purpose. (server returns them in snake_case)
    /// Opens the URL in the user's default browser. Protocol must be `http` or `https`
    pub open_url: Option<String>,

    /// Runs the command.
    /// Simply causes the user to say the string in chat -
    /// so only has command effect if it starts with /
    ///
    /// Irrelevant for motd purposes.
    pub run_command: Option<String>,

    /// Replaces the content of the user's chat box with the given text.
    ///
    /// Irrelevant for motd purposes.
    pub suggest_command: Option<String>,

    /// Copies the given text into the client's clipboard.
    pub copy_to_clipboard: Option<String>,
}

/// HoverEvent data for a chat component
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatHoverEvent {
    // These are not renamed on purpose. (server returns them in snake_case)
    /// Text to show when the item is hovered over
    pub show_text: Option<Box<ChatObject>>,

    /// Same as show_text, but for servers < 1.16
    pub value: Option<Box<ChatObject>>,

    /// Displays the item of the given NBT
    pub show_item: Option<String>,

    /// Displays information about the entity with the given NBT
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
