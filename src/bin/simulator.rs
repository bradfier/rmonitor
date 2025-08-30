use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

/// Simulates an RMonitor server using a text file from the samples directory as a data source
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:50000").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::task::spawn(async move {
            // Ignore dropped connections
            let _ = handle_socket(socket, addr).await;
        });
    }
}

async fn handle_socket(mut socket: TcpStream, addr: SocketAddr) -> std::io::Result<()> {
    println!("Client connected from: {:?}", addr);
    let content = include_bytes!("../../sample/Orbits_Mock_Session.txt");
    let lines = content.split(|&b| b == b'\n');

    for line in lines {
        socket.write_all(line).await?;
        socket.write_all(&[b'\n']).await?;

        // If this was a heartbeat message, delay sending the next for 1s
        if &line[..2] == b"$F" {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}
