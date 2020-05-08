use std::error::Error;
use rmonitor::codec::{RMonitorDecoder, RMonitorCodecError};
use rmonitor::protocol::Record;
use tokio_util::codec::{Decoder, FramedRead};
use tokio::stream::StreamExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:4000").await?;

    let mut reader = FramedRead::new(stream, RMonitorDecoder::new(2048));

    while let Ok(Some(Ok(event))) = timeout(Duration::from_millis(10), reader.next()).await {
        println!("{:?}", event);
    }

    Ok(())
}

