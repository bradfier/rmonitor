use rmonitor::codec::RMonitorDecoder;
use std::io::Read;
use std::net::TcpStream;
use tokio_util::bytes::{BufMut, BytesMut};
use tokio_util::codec::Decoder;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:4000").expect("Failed to open connection");

    // .read() won't use the BytesMut directly in the appropriate manner,
    // so we do one extra bit of buffering here
    let mut read_buf = [0u8; 256];

    // Create the decode buffer, and a decoder with a maximum line length
    // of 2048.
    let mut buffer = BytesMut::with_capacity(4096);
    let mut decoder = RMonitorDecoder::new_with_max_length(2048);

    loop {
        let r = stream
            .read(&mut read_buf)
            .expect("Failed to read from stream");

        // Put only the read bytes into the decode buffer (not any trailing 0s)
        if r > 0 {
            buffer.put(&read_buf[..r]);
        }

        let maybe_record = decoder.decode(&mut buffer);

        match maybe_record {
            Ok(None) => {
                continue;
            }
            Ok(Some(r)) => println!("{:?}", r),
            Err(e) => println!("{:?}", e),
        }
    }
}
