# mc-query

![Crates.io](https://img.shields.io/crates/v/mc-query?style=for-the-badge)
![Crates.io](https://img.shields.io/crates/d/mc-query?style=for-the-badge)
![Crates.io](https://img.shields.io/crates/l/mc-query?style=for-the-badge)

![docs.rs](https://img.shields.io/docsrs/mc-query?style=for-the-badge)

Implementations of [Server List ping](https://wiki.vg/Server_List_Ping), [Query](https://wiki.vg/Query), and [RCON](https://wiki.vg/RCON) using the Minecraft networking protocol.

Maybe in the future there will be a CLI to access these features as well.

## Installation

To use this library, just run `cargo add mc-query` or add `mc-query = "0.1.0"` to your `Cargo.toml`.

## Usage

You can read the docs [here](https://docs.rs/mc-query).

## Examples

### Using `status` to get basic server information

```rs
use mc_query::status;
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let data = status("mc.hypixel.net", 25565).await?;
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

(Query still under development...)

## Reference

-   [wiki.vg](https://wiki.vg) - documentation of the various protocols implemented in this crate

## Testing

Some tests in this library require a minecraft server to be running on `localhost`.
If you are contributing a feature or bugfix that involves one of these tests,
run the convienient testing script `./test` (or `py -3 test` on Windows).

This requires a decently modern version of Python 3, and a recent version of

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
