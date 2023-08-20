use crate::err::{Error, Result};
use std::borrow::Cow;
use std::time::Duration;
use tokio::{io::AsyncBufReadExt, sync::mpsc};
use tokio_serial::SerialPortBuilderExt;

pub async fn read<'a>(
    tty: impl Into<Cow<'a, str>>,
    baud_rate: u32,
    tx: mpsc::Sender<String>,
) -> Result<()> {
    let tty = tty.into();

    loop {
        if let Err(e) = try_read(&tty, baud_rate, tx.clone()).await {
            log::error!("{}: {}", tty, e);
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn try_read(tty: &str, baud_rate: u32, tx: mpsc::Sender<String>) -> Result<()> {
    let timeout = Duration::from_secs(180);

    let stream = tokio_serial::new(tty, baud_rate)
        .data_bits(tokio_serial::DataBits::Eight)
        .parity(tokio_serial::Parity::None)
        .stop_bits(tokio_serial::StopBits::One)
        .flow_control(tokio_serial::FlowControl::None)
        .open_native_async()
        .map_err(|e| Error::Other(format!("{}: error opening serial port: {}", tty, e)))?;

    let mut lines = tokio::io::BufReader::new(stream).lines();

    while let Some(line) = tokio::time::timeout(timeout, lines.next_line()).await?? {
        if line.chars().all(|c| c.is_whitespace()) {
            continue; // skip whitespace
        }
        log::info!("{}: recv {:?}", tty, line);

        match tx.try_send(line) {
            Ok(()) => (),
            Err(_) => {
                log::error!("busy, dropping packet");
            }
        }
    }
    Err(Error::Disconnected)
}
