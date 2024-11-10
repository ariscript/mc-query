//! Enables remote command execution for minecraft servers.
//! See the documentation for [`RconClient`] for more information.

mod client;
mod packet;

pub use client::RconClient;

const MAX_LEN_CLIENTBOUND: usize = 4096;
const MAX_LEN_SERVERBOUND: usize = 1446;
