//! Implementations of [Server List ping](https://wiki.vg/Server_List_Ping),
//! [Query](https://wiki.vg/Query), and [RCON](https://wiki.vg/RCON) using the
//! Minecraft networking protocol.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

macro_rules! create_timeout {
    ($name:ident, $ret:ty) => {
        ::paste::paste! {
            #[doc = concat!("Similar to [`", stringify!($name), "`]")]
            /// but with an added argument for timeout.
            ///
            /// Note that timeouts are not precise, and may vary on the order
            /// of milliseconds, because of the way the async event loop works.
            ///
            /// # Arguments
            /// * `host` - A string slice that holds the hostname of the server to connect to.
            /// * `port` - The port to connect to on that server.
            ///
            /// # Errors
            /// Returns `Err` on any condition that
            #[doc = concat!("[`", stringify!($name), "`]")]
            /// does, and also when the response is not fully recieved within `dur`.
            pub async fn [<$name _with_timeout>](
                host: &str,
                port: u16,
                dur: ::std::time::Duration,
            ) -> ::std::io::Result<$ret> {
                use crate::errors::timeout_err;
                use ::tokio::time::timeout;

                timeout(dur, $name(host, port))
                    .await
                    .unwrap_or(timeout_err::<$ret>())
            }
        }
    };
}

pub mod errors;
pub mod query;
pub mod rcon;
mod socket;
pub mod status;
mod varint;

pub use status::status;
