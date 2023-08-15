use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    select,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

pub static DEFAULT_SERVER: &str = "rotate.aprs2.net:14580";

pub async fn run(server: impl AsRef<str>, tx: mpsc::Sender<String>, stop: CancellationToken) {
    let server = server.as_ref();
    while !stop.is_cancelled()  {
        log::debug!("{}: connecting...", server);
        let stream_res = select! {
            _ = stop.cancelled() => return,
            r = tokio::net::TcpStream::connect(server) => r
        };

        match stream_res {
            Ok(stream) => {
                log::info!("{}: connected", server);
                run_session(server, stream, tx.clone(), stop.clone()).await;
                log::debug!("{}: session ended", server);
            }
            Err(e) => {
                log::error!("{}: connection error: {}", server, e);
            }
        }
        sleep(std::time::Duration::from_secs(3), &stop).await;
    }
}

async fn run_session(
    server: &str,
    mut stream: TcpStream,
    tx: mpsc::Sender<String>,
    stop: CancellationToken,
) {
    //let hello = b"user N0CALL pass -1 vers liveplaya 0.00 filter r/40.79608429483125/-119.19589964220306/200\r\n";
    let hello = b"user N0CALL pass -1 vers liveplaya 0.00 filter r/37.371111/-122.0375/100 r/40.79608429483125/-119.19589964220306/100\r\n";

    log::debug!("{}: starting session...", server);
    let write_res = select! {
        _ = stop.cancelled() => return,
        r = stream.write_all(hello) => r,
    };

    match write_res {
        Ok(_) => {
            log::debug!("{}: session started", server);
            read_packets(server, stream, tx, &stop).await;
        }
        Err(e) => {
            log::error!("{}: error sending data: {}", server, e);
        }
    }
}

async fn read_packets(
    server: &str,
    mut stream: TcpStream,
    mut packet_tx: mpsc::Sender<String>,
    stop: &CancellationToken,
) {
    let (stream_rx, _stream_tx) = stream.split();
    let mut reader = BufReader::new(stream_rx);

    loop {
        let mut line = String::new();
        let read_res = select! {
            _ = stop.cancelled() => break,
            r = reader.read_line(&mut line) => r
        };
        match read_res {
            Ok(bytes_read) => {
                if bytes_read == 0 || line.chars().all(|c| c.is_whitespace()) {
                    continue; // skip whitespace
                }
                log::debug!("{}: recv {:?}", server, line);
                if line.starts_with("# Login by user not allowed") {
                    log::error!("{}: can't login", server);
                    return;
                }
                if line.starts_with('#') {
                    continue;
                }
                notify(line, &mut packet_tx, &stop).await;
            }
            Err(e) => {
                log::error!("{}: error receiving data: {}", server, e);
                break;
            }
        }
    }
    log::debug!("{}: exit read loop", server);
}

async fn notify(packet: String, tx: &mut mpsc::Sender<String>, stop: &CancellationToken) {
    let send_res = select! {
        _ = stop.cancelled() => return,
        r = tx.send(packet) => r
    };
    if let Err(e) = send_res {
        // Another end of service disconnected, cancelling
        log::debug!("aprs_is reader disconnected, shutting down");
        stop.cancel();
    }
}

async fn sleep(dur: std::time::Duration, stop: &CancellationToken) {
    select! {
        _ = stop.cancelled() => return,
        _ = tokio::time::sleep(dur) => return,
    }
}
