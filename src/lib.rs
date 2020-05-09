//! A simple Tokio-compatible [`Decoder`] implementation of the [RMonitor Protocol]
//! supported by a variety of sports timing software.
//!
//! [`Decoder`]: tokio_util::codec::Decoder
//! [RMonitor Protocol]: https://github.com/bradfier/rmonitor/blob/master/docs/RMonitor%20Timing%20Protocol.pdf

pub mod codec;
pub mod protocol;
