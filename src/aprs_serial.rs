use std::borrow::Cow;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    select,
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

pub async fn run<'a>(
    tty: impl Into<Cow<'a, str>>,
    baud_rate: u32,
    tx: mpsc::Sender<String>,
    stop: CancellationToken,
) {
    let tty = tty.into();
    while !stop.is_cancelled() {
        let port_res = tokio_serial::new(tty.as_ref(), baud_rate)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .flow_control(tokio_serial::FlowControl::None)
            .open_native_async();

        match port_res {
            Ok(stream) => {
                let tty = tty.as_ref();
                read_packets(tty, stream, tx.clone(), &stop).await;
            }
            Err(e) => {
                let tty = tty.as_ref();
                log::error!("{}: error opening serial port: {}", tty, e);
            }
        }
        sleep(std::time::Duration::from_secs(3), &stop).await;
    }
}

async fn read_packets(
    tty: &str,
    stream: SerialStream,
    mut packet_tx: mpsc::Sender<String>,
    stop: &CancellationToken,
) {
    let mut reader = BufReader::new(stream);

    loop {
        let mut line = String::new();
        let read_res = select! {
            _ = stop.cancelled() => return,
            r = reader.read_line(&mut line) => r
        };
        match read_res {
            Ok(bytes_read) => {
                if bytes_read == 0 || line.chars().all(|c| c.is_whitespace()) {
                    continue; // skip whitespace
                }
                log::debug!("{}: recv {:?}", tty, line);
                notify(line, &mut packet_tx, &stop).await;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    // Ignore timeouts.
                    // But should we, maybe reconnect is better?...
                    continue;
                }
                log::error!("{}: error receiving data: {}", tty, e);
                break;
            }
        }
        log::debug!("{}: exit read loop", tty);
    }
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
