//! Implementations of [Server List ping](https://wiki.vg/Server_List_Ping),
//! [Query](https://wiki.vg/Query), and [RCON](https://wiki.vg/RCON) using the
//! Minecraft networking protocol.

#![warn(missing_docs)]

pub mod errors;
pub mod rcon;
mod socket;
pub mod status;
mod varint;

pub use status::status; // so users don't need to `use mc_query::status::status;`
