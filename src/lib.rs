//! A simple Tokio-compatible [`Decoder`] implementation of the [RMonitor Protocol]
//! supported by a variety of sports timing software.
//!
//! [`Decoder`]: tokio_util::codec::Decoder
//! [RMonitor Protocol]: https://github.com/bradfier/rmonitor/blob/master/docs/RMonitor%20Timing%20Protocol.pdf
//!
//! # Example
//!
//! ```no_run
//! use rmonitor::RMonitorDecoder;
//! # use std::error::Error;
//! use tokio::net::TcpStream;
//! # use tokio::stream::StreamExt;
//! use tokio_util::codec::FramedRead;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Connect to an RMonitor server
//!     let stream = TcpStream::connect("127.0.0.1:4000").await?;
//!
//!     // Create a FramedReader with our decoder
//!     let mut reader = FramedRead::new(stream, RMonitorDecoder::new_with_max_length(2048));
//!
//!     // Print out all the messages we receive
//!     while let Some(Ok(event)) = reader.next().await {
//!         println!("{:?}", event);
//!     }
//!     # Ok(())
//! }
//! ```

pub mod codec;
pub use codec::RMonitorDecoder;

pub mod protocol;
