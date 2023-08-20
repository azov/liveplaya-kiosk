use crate::err::{Error, Result};
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    sync::mpsc,
};

pub static DEFAULT_SERVER: &str = "rotate.aprs2.net:14580";

pub async fn read(server: impl AsRef<str>, tx: mpsc::Sender<String>) -> Result<()> {
    let server = server.as_ref();
    loop {
        if let Err(e) = try_read(&server, tx.clone()).await {
            log::error!("{}: {}", server, e);
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn try_read(server: &str, tx: mpsc::Sender<String>) -> Result<()> {
    let timeout = std::time::Duration::from_secs(5);

    log::debug!("{}: connecting...", server);
    let mut stream =
        tokio::time::timeout(timeout, tokio::net::TcpStream::connect(server)).await??;
    log::info!("{}: connected", server);

    let hello = b"user N0CALL pass -1 vers liveplaya 0.00 filter r/40.79608429483125/-119.19589964220306/200\r\n";
    // let hello = b"user N0CALL pass -1 vers liveplaya 0.00 filter r/37.371111/-122.0375/100 r/40.79608429483125/-119.19589964220306/100\r\n";

    log::debug!("{}: starting session...", server);
    tokio::time::timeout(timeout, stream.write_all(hello)).await??;

    let (_stream_rx, _stream_tx) = stream.split();
    let mut lines = tokio::io::BufReader::new(stream).lines();

    // read packets
    let timeout = std::time::Duration::from_secs(180);
 
    while let Some(line) = tokio::time::timeout(timeout, lines.next_line()).await?? {
        if line.chars().all(|c| c.is_whitespace()) {
            continue; // skip whitespace
        }
        log::info!("{}: recv {:?}", server, line);
        if line.starts_with("# Login by user not allowed") {
            return Err(Error::msg("can't login"));
        }
        if line.starts_with('#') {
            continue;
        }

        match tx.try_send(line) {
            Ok(()) => (),
            Err(_) => {
                log::error!("busy, dropping packet");
            }
        }
    }
    Err(Error::Disconnected)
}
