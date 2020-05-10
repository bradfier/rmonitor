use rmonitor::codec::RMonitorDecoder;
use std::error::Error;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio::time::timeout;
use tokio_util::codec::FramedRead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:4000").await?;

    let mut reader = FramedRead::new(stream, RMonitorDecoder::new_with_max_length(2048));

    while let Ok(Some(Ok(event))) = timeout(Duration::from_millis(10), reader.next()).await {
        println!("{:?}", event);
    }

    Ok(())
}
