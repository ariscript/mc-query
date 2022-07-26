# mc-query

Implementations of [Server List ping](https://wiki.vg/Server_List_Ping), [Query](https://wiki.vg/Query), and [RCON](https://wiki.vg/RCON) using the Minecraft networking protocol.

Maybe in the future there will be a CLI to access these features as well.

## Installation

To use this library, just run `cargo add mc-query` or add `mc-query = "0.1.0"` to your `Cargo.toml`.

## Usage

You can read the docs [here](https://docs.rs/mc-query).

## Examples

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

(Query and RCON are still under development...)

## Reference

-   [wiki.vg](https://wiki.vg) - documentation of the various protocols implemented in this crate

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
