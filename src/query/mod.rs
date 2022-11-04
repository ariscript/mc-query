//! Implementation of the [Query](https://wiki.vg/Query) protocol.

use rand::random;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{errors::QueryProtocolError, socket::ReadWriteNullTermString};

mod packet;

#[derive(Debug)]
pub struct BasicStatResponse {
    pub game_type: String,
    pub motd: String,
    pub map: String,
    pub current_players: u32,
    pub max_players: u32,
}

pub async fn stat_basic(host: &str, port: u16) -> io::Result<BasicStatResponse> {
    let mut socket = TcpStream::connect(format!("{host}:{port}")).await?;

    let session_id = random::<i32>() & 0x0f0f0f0f;

    // handshake packet
    socket.write_u16(0xfe_fd).await?;
    socket.write_u8(9).await?;
    socket.write_i32(session_id).await?;

    // handshake response
    let packet_type = socket.read_u8().await?;

    if ![0, 9].contains(&packet_type) {
        return Err(QueryProtocolError::InvalidPacketType.into());
    } else if packet_type != 9 {
        return Err(QueryProtocolError::UnexpectedPacketType.into());
    }

    let recieved_id = socket.read_i32().await?;

    if recieved_id != session_id {
        return Err(QueryProtocolError::SessionIdMismatch.into());
    }

    let challenge_token = socket
        .read_null_terminated_string()
        .await?
        .parse::<i32>()
        .map_err(|_| QueryProtocolError::InvalidChallengeToken)
        .map_err(io::Error::from)?;

    // basic stat request
    socket.write_u16(0xfe_fd).await?;
    socket.write_u8(0).await?;
    socket.write_i32(session_id).await?;
    socket.write_i32(challenge_token).await?;

    // basic stat response
    let packet_type = socket.read_u8().await?;

    if ![0, 9].contains(&packet_type) {
        return Err(QueryProtocolError::InvalidPacketType.into());
    } else if packet_type != 9 {
        return Err(QueryProtocolError::UnexpectedPacketType.into());
    }

    let recieved_id = socket.read_i32().await?;

    if recieved_id != session_id {
        return Err(QueryProtocolError::SessionIdMismatch.into());
    }

    let motd = socket.read_null_terminated_string().await?;
    let game_type = socket.read_null_terminated_string().await?;
    let map = socket.read_null_terminated_string().await?;
    let current_players = read_u32_from_string(&mut socket).await?;
    let max_players = read_u32_from_string(&mut socket).await?;

    let _host_port = socket.read_u16_le().await?;
    let _host_ip = socket.read_null_terminated_string().await?;

    Ok(BasicStatResponse {
        motd,
        game_type,
        map,
        current_players,
        max_players,
    })
}

async fn read_u32_from_string(socket: &mut TcpStream) -> io::Result<u32> {
    socket
        .read_null_terminated_string()
        .await?
        .parse::<u32>()
        .map_err(|_| QueryProtocolError::CannotParseInt.into())
}

#[cfg(test)]
mod tests {
    use tokio::io;

    use super::stat_basic;

    #[tokio::test]
    async fn test_stat_basic() -> io::Result<()> {
        let response = stat_basic("localhost", 25565).await?;
        println!("{response:?}");

        Ok(())
    }
}
