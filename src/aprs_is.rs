use crate::{
    err::{Error, Result},
    util::tbc,
};
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

pub static DEFAULT_SERVER: &str = "rotate.aprs2.net:14580";

pub async fn run(server: impl AsRef<str>, tx: mpsc::Sender<String>, stop: CancellationToken) {
    let server = server.as_ref();

    while !stop.is_cancelled() {
        if let Err(e) = run_session(server, tx.clone(), &stop).await {
            log::error!("{}: {}", server, e);
        }
        _ = tbc::sleep(Duration::from_secs(3), &stop).await;
    }
}

async fn run_session(server: &str, tx: mpsc::Sender<String>, stop: &CancellationToken) -> Result<()> {
    let timeout = std::time::Duration::from_secs(5);

    log::debug!("{}: connecting...", server);
    let mut stream = tbc::timebound_cancellable(
        tokio::net::TcpStream::connect(server),
        timeout,
        &stop,
    )
    .await?;
    log::info!("{}: connected", server);

    //let hello = b"user N0CALL pass -1 vers liveplaya 0.00 filter r/40.79608429483125/-119.19589964220306/200\r\n";
    let hello = b"user N0CALL pass -1 vers liveplaya 0.00 filter r/37.371111/-122.0375/100 r/40.79608429483125/-119.19589964220306/100\r\n";

    log::debug!("{}: starting session...", server);
    _ = tbc::timebound_cancellable(
        stream.write_all(hello),
        timeout,
        &stop
    ).await?;

    let (stream_rx, _stream_tx) = stream.split();
    let mut reader = BufReader::new(stream_rx);

    // read packets
    loop {
        let mut line = String::new();
        let bytes_read = tbc::timebound_cancellable(
            reader.read_line(&mut line),
            //  restart the session if we get nothing from the server for a few mins
            Duration::from_secs(180),
            &stop
        ).await?;

        if bytes_read == 0 || line.chars().all(|c| c.is_whitespace()) {
            continue; // skip whitespace
        }
        log::info!("{}: recv {:?}", server, line);
        if line.starts_with("# Login by user not allowed") {
            return Err(Error::msg("can't login"));
        }
        if line.starts_with('#') {
            continue;
        }

        // Send the packet back. Use fairly long timeout in case the server is
        // busy
        tbc::timebound_cancellable(tx.send(line), Duration::from_secs(200), stop).await?;
    }
}

