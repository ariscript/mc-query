//! Implementation of the [Query](https://wiki.vg/Query) protocol.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::net::UdpSocket;
use rand::random;
use tokio::io;

use crate::errors::QueryProtocolError;

static QUERY_MAGIC: u16 = 0xfe_fd;
static SESSION_ID_MASK: u32 = 0x0f_0f_0f_0f;

/// A response from the server's basic query.
/// Taken from [wiki.vg](https://wiki.vg/Query#Response_2)
#[derive(Debug)]
pub struct BasicStatResponse {
    /// The "motd" - message shown in the server list by the client.
    pub motd: String,

    /// The server's game type.
    /// Vanilla servers hardcode this to "SMP".
    pub game_type: String,

    /// The server's world/map name.
    pub map: String,

    /// The current number of online players.
    pub num_players: usize,

    /// Maximum players online this server allows.
    pub max_players: usize,

    /// The port the serer is running on.
    pub host_port: u16,

    /// THe server's IP address.
    pub host_ip: String,
}

/// Perform a basic stat query of the server per the [Query Protocol](https://wiki.vg/Query#Basic_Stat).
/// Note that the server must have `query-enabled=true` set in its properties to get a response.
/// The `query.port` property might also be different from `server.port`.
///
/// # Arguments
/// * `host` - the hostname/IP of thr server to query
/// * `port` - the port that the server's Query is running on
///
/// # Examples
/// ```
/// use mc_query::query;
/// use tokio::io::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let res = query::stat_basic("localhost", 25565).await?;
///     println!("The server has {} players online out of {}", res.num_players, res.num_players);
///
///     Ok(())
/// }
/// ```
pub async fn stat_basic(host: &str, port: u16) -> io::Result<BasicStatResponse> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(format!("{host}:{port}")).await?;

    let (token, session) = handshake(&socket).await?;

    let mut bytes = BytesMut::new();
    bytes.put_u16(QUERY_MAGIC);
    bytes.put_u8(0); // packet type 0 - handshake
    bytes.put_i32(session);
    bytes.put_i32(token);
    socket.send(&bytes).await?;

    let mut res = recv_packet(&socket).await?;
    validate_packet(&mut res, 0, session)?;

    let motd = get_string(&mut res)?;
    let game_type = get_string(&mut res)?;
    let map = get_string(&mut res)?;
    let num_players = get_string(&mut res)?
        .parse()
        .map_err::<io::Error, _>(|_| QueryProtocolError::CannotParseInt.into())?;
    let max_players = get_string(&mut res)?
        .parse()
        .map_err::<io::Error, _>(|_| QueryProtocolError::CannotParseInt.into())?;

    let host_port = res.get_u16_le(); // shorts are little endian per protocol

    let host_ip = get_string(&mut res)?;

    Ok(BasicStatResponse {
        motd,
        game_type,
        map,
        num_players,
        max_players,
        host_port,
        host_ip,
    })
}

/// Perform a handshake request per https://wiki.vg/Query#Handshake
///
/// # Returns
/// A tuple `(challenge_token, session_id)` to be used in subsequent server interactions
async fn handshake(socket: &UdpSocket) -> io::Result<(i32, i32)> {
    // generate new token per interaction to avoid reset problems
    let session_id = (random::<u32>() & SESSION_ID_MASK) as i32;

    let mut req = BytesMut::with_capacity(7);
    req.put_u16(QUERY_MAGIC);
    req.put_u8(9); // packet type 9 - handshake
    req.put_i32(session_id);
    // no payload for handshake requests

    socket.send(&req).await?;

    let mut res = recv_packet(socket).await?;

    validate_packet(&mut res, 9, session_id)?;

    let token_str = get_string(&mut res)?;

    token_str.parse()
        .map(|t| (t, session_id))
        .map_err(|_| QueryProtocolError::CannotParseInt.into())
}

async fn recv_packet(socket: &UdpSocket) -> io::Result<Bytes> {
    let mut buf = [0u8; 65536];
    socket.recv(&mut buf).await?;

    Ok(Bytes::copy_from_slice(&buf))
}

fn validate_packet(packet: &mut Bytes, expected_type: u8, expected_session: i32) -> io::Result<()> {
    let recv_type = packet.get_u8();
    if recv_type != expected_type {
        return Err(QueryProtocolError::InvalidPacketType.into());
    }

    let recv_session = packet.get_i32();
    if recv_session != expected_session {
        return Err(QueryProtocolError::SessionIdMismatch.into());
    }

    Ok(())
}

fn get_string(bytes: &mut Bytes) -> io::Result<String> {
    let mut buf = vec![];
    loop {
        let byte = bytes.get_u8();
        if byte == 0 {
            break;
        }
        buf.push(byte);
    }

    String::from_utf8(buf)
        .map_err(|_| QueryProtocolError::InvalidUtf8.into())
}

#[cfg(test)]
mod tests {
    use tokio::io;

    use super::stat_basic;

    #[tokio::test]
    async fn test_stat_basic() -> io::Result<()> {
        let response = stat_basic("localhost", 25565).await?;
        println!("{response:#?}");

        Ok(())
    }
}
