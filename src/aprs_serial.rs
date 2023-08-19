use crate::{
    err::{Error, Result},
    util::tbc,
};
use std::borrow::Cow;
use std::time::Duration;
use tokio::{io::AsyncBufReadExt, io::BufReader, sync::mpsc};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::sync::CancellationToken;

pub async fn run<'a>(
    tty: impl Into<Cow<'a, str>>,
    baud_rate: u32,
    tx: mpsc::Sender<String>,
    stop: CancellationToken,
) {
    let tty = tty.into();
    while !stop.is_cancelled() {
        if let Err(e) = run_session(&tty, baud_rate, tx.clone(), &stop).await {
            log::error!("{}: {}", tty, e);
        }
        _ = tbc::sleep(Duration::from_secs(3), &stop).await;
    }
}

async fn run_session(
    tty: &str,
    baud_rate: u32,
    tx: mpsc::Sender<String>,
    stop: &CancellationToken,
) -> Result<()> {
    let stream = tokio_serial::new(tty, baud_rate)
        .data_bits(tokio_serial::DataBits::Eight)
        .parity(tokio_serial::Parity::None)
        .stop_bits(tokio_serial::StopBits::One)
        .flow_control(tokio_serial::FlowControl::None)
        .open_native_async()
        .map_err(|e| Error::Other(format!("{}: error opening serial port: {}", tty, e)))?;

    let mut reader = BufReader::new(stream);

    loop {
        let mut line = String::new();
        let bytes_read = tbc::timebound_cancellable(
            reader.read_line(&mut line),
            //  restart the session if we get nothing from the port for a few mins
            Duration::from_secs(180),
            &stop,
        )
        .await?;

        if bytes_read == 0 || line.chars().all(|c| c.is_whitespace()) {
            continue; // skip whitespace
        }

        log::info!("{}: recv {:?}", tty, line);

        // Send the packet back. Use fairly long timeout in case the server is
        // busy
        tbc::timebound_cancellable(tx.send(line), Duration::from_secs(200), stop).await?;
    }
}
