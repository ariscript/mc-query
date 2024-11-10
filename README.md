# mc-query

![Crates.io](https://img.shields.io/crates/v/mc-query?style=for-the-badge)
![Crates.io](https://img.shields.io/crates/d/mc-query?style=for-the-badge)
![Crates.io](https://img.shields.io/crates/l/mc-query?style=for-the-badge)

![docs.rs](https://img.shields.io/docsrs/mc-query?style=for-the-badge)

Implementations of [Server List ping](https://wiki.vg/Server_List_Ping), [Query](https://wiki.vg/Query), and [RCON](https://wiki.vg/RCON) using the Minecraft networking protocol.

Maybe in the future there will be a CLI to access these features as well.

## Installation

To use this library, just run `cargo add mc-query`.

## Usage

You can read the docs [here](https://docs.rs/mc-query).

## Examples

### Using `status` to get basic server information

```rs
use mc_query::status;
use tokio::io::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let data = status("mc.hypixel.net", 25565, Duration::from_secs(5)).await?;
    println!("{data:#?}");

    Ok(())
}
```

### Using `RconClient` to run commands via RCON

```rs
use mc_query::rcon::RconClient;
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = RconClient::new("localhost", 25565);
    client.authenticate("supersecretrconpassword").await?;

    let response = client.run_command("time set 0").await?;
    println!("{response}");

    Ok(())
}
```

### Using `stat_basic` to query the server

```rs
use mc_query::query;
use tokio::io::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let res = stat_basic("localhost", 25565, Duration::from_secs(5)).await?;
    println!(
       "Server has {} out of {} players online",
       res.num_players,
       res.max_players
    );
    
    Ok(())
}
```

### Using `stat_full` to query the server

```rs
use mc_query::query;
use tokio;:io::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let res = stat_full("localhost", 25565, Duration::from_secs(5)).await?;
    println!("Online players: {:#?}, res.players);
    
    Ok(())
}
```

## Reference

-   [wiki.vg](https://wiki.vg) - documentation of the various protocols implemented in this crate

## Testing

Some tests in this library require a minecraft server to be running on `localhost`.
If you are contributing a feature or bugfix that involves one of these tests,
run the convenient testing script `./test` (or `py -3 test` on Windows).
You can also just run a minecraft server without the cargo tests (useful for debugging with IDEs) with `./test --server-only true`.

This requires a decently modern version of Python 3, and Java 17 or higher to run the server.

## License

Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Mojang

This project is in no way involved with or endorsed by Mojang Synergies AB or Microsoft Corporation.
Any use of their services (including running some tests in this library) requires you to agree to their [terms](https://minecraft.net/eula).
