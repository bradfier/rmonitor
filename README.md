# RMonitor for Rust

[![GHA Build Status](https://github.com/bradfier/rmonitor/workflows/CI/badge.svg)](https://github.com/bradfier/rmonitor/actions?query=workflow%3ACI)
![MIT/Apache Licensed](https://img.shields.io/crates/l/rmonitor)
[![crates.io](https://img.shields.io/crates/v/rmonitor)](https://crates.io/crates/rmonitor)
[![Docs](https://docs.rs/rmonitor/badge.svg)](https://docs.rs/rmonitor)

A simple, Tokio-compatible protocol decoder for RMonitor, a line based timing
protocol supported by different vendors of sport timing software.

The decoder supports both:
* The original [RMonitor Timing Protocol](./docs/RMonitor%20Timing%20Protocol.pdf)
* The [IMSA Enhanced](./docs/IMSA%20Enhanced%20RMon%20Timing%20Protocol%20v1.03.pdf) protocol, which adds two extended record types.

## Example

You'll need `rmonitor`, `tokio` and `tokio-util` in your dependencies:

```toml
rmonitor = "0.1"
tokio-util = { version = "0.3", features = ["codec"] }
tokio = { version = "0.2", features = ["full"] }
```

Then create your `main.rs`:

```rust
use rmonitor::codec::RMonitorDecoder;
use std::error::Error;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::FramedRead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to your target RMonitor server
    let stream = TcpStream::connect("127.0.0.1:4000").await?;

    // Constructs a decode with a maximum line length of 2048
    let mut reader = FramedRead::new(stream, RMonitorDecoder::new(2048));

    while let Ok(Some(Ok(event))) = reader.next().await {
        println!("{:?}", event);
    }

    Ok(())
}
```

A [synchronous example](./examples/sync.rs) is also available to show use of the decoder
without pulling in a Tokio runtime.

## License
Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
