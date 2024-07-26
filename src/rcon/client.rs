//! Implementation of the [RCON](https://wiki.vg/RCON) protocol.

use super::{
    packet::{RconPacket, RconPacketType},
    MAX_LEN_CLIENTBOUND,
};
use crate::errors::RconProtocolError;
use bytes::{BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, Error},
    net::TcpStream,
};

/// Struct that stores the connection and other state of the RCON protocol with the server.
///
/// # Examples
///
/// ```no_run
/// use mc_query::rcon::RconClient;
/// use tokio::io::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let mut client = RconClient::new("localhost", 25575).await?;
///     client.authenticate("password").await?;
///
///     let output = client.run_command("time set day").await?;
///     println!("{output}");
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct RconClient {
    socket: TcpStream,
}

impl RconClient {
    /// Construct an [RconClient] that connects to the given host and port.
    /// Note: to authenticate use the `authenticate` method, this method does not take a password.
    ///
    /// # Arguments
    /// * `host` - A string slice that holds the hostname of the server to connect to.
    /// * `port` - The port to connect to.
    ///
    /// # Errors
    /// Returns `Err` if there was a network error.
    pub async fn new(host: &str, port: u16) -> io::Result<Self> {
        let connection = TcpStream::connect(format!("{host}:{port}")).await?;

        Ok(Self { socket: connection })
    }

    /// Disconnect from the server and close the RCON connection.
    ///
    /// # Errors
    /// Returns `Err` if there was an issue closing the connection.
    pub async fn disconnect(mut self) -> io::Result<()> {
        self.socket.shutdown().await
    }

    /// Authenticate with the server, with the given password.
    ///
    /// If authentication fails, this method will return [RconProtocolError::AuthFailed].
    ///
    /// # Arguments
    /// * `password` - A string slice that holds the RCON password.
    ///
    /// # Errors
    /// Returns the raw `tokio::io::Error` if there was a network error.
    /// Returns an apprpriate [`RconProtocolError`] if the authentication failed for other reasons.
    pub async fn authenticate(&mut self, password: &str) -> io::Result<()> {
        let packet =
            RconPacket::new(1, RconPacketType::Login, password.to_string()).map_err(Error::from)?;

        self.write_packet(packet).await?;

        let packet = self.read_packet().await?;

        if !matches!(packet.packet_type, RconPacketType::RunCommand) {
            return Err(RconProtocolError::InvalidPacketType.into());
        }

        if packet.request_id == -1 {
            return Err(RconProtocolError::AuthFailed.into());
        } else if packet.request_id != 1 {
            return Err(RconProtocolError::RequestIdMismatch.into());
        }

        Ok(())
    }

    /// Run the given command on the server and return the result.
    ///
    /// # Arguments
    /// * `command` - A string slice that holds the command to run. Must be ASCII and under 1446 bytes in length.
    ///
    /// # Errors
    /// Returns an error if there was a network issue or an [`RconProtocolError`] for other failures.
    pub async fn run_command(&mut self, command: &str) -> io::Result<String> {
        let packet = RconPacket::new(1, RconPacketType::RunCommand, command.to_string())
            .map_err(Error::from)?;

        self.write_packet(packet).await?;

        let mut full_payload = String::new();

        loop {
            let recieved = self.read_packet().await?;

            if recieved.request_id == -1 {
                return Err(RconProtocolError::AuthFailed.into());
            } else if recieved.request_id != 1 {
                return Err(RconProtocolError::RequestIdMismatch.into());
            }

            full_payload.push_str(&recieved.payload);

            // wiki says this method of determining if this is the end of the
            // response is not 100% reliable, but this is the best solution imo
            // if this ends up being a problem, this can be changed later
            if recieved.payload.len() < MAX_LEN_CLIENTBOUND {
                break;
            }
        }

        Ok(full_payload)
    }

    /// Read a packet from the socket.
    async fn read_packet(&mut self) -> io::Result<RconPacket> {
        let len = self.socket.read_i32_le().await?;

        let mut bytes = BytesMut::new();
        bytes.put_i32_le(len);

        for _ in 0..len {
            let current = self.socket.read_u8().await?;
            bytes.put_u8(current);
        }

        RconPacket::try_from(bytes.freeze()).map_err(Error::from)
    }

    /// Write a packet to the socket.
    ///
    /// # Arguments
    /// * `packet` - An owned [`RconPacket`] to write to the socket.
    async fn write_packet(&mut self, packet: RconPacket) -> io::Result<()> {
        let bytes = packet.bytes();

        self.socket.write_all(&bytes).await
    }
}

#[cfg(test)]
mod tests {
    use super::RconClient;
    use tokio::io;

    #[tokio::test]
    async fn test_rcon_command() -> io::Result<()> {
        let mut client = RconClient::new("localhost", 25575).await?;
        client.authenticate("mc-query-test").await?;
        let response = client.run_command("time set day").await?;

        println!("recieved response: {response}");

        Ok(())
    }

    #[tokio::test]
    async fn test_rcon_unauthenticated() -> io::Result<()> {
        let mut client = RconClient::new("localhost", 25575).await?;
        let result = client.run_command("time set day").await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_rcon_incorrect_password() -> io::Result<()> {
        let mut client = RconClient::new("localhost", 25575).await?;
        let result = client.authenticate("incorrect").await;

        assert!(result.is_err());

        Ok(())
    }
}
