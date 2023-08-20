use crate::{err::Result, util::tbc, util::time::Timestamp};
use std::path::{Path, PathBuf};
use std::time::Duration;
use time::Time;
use tokio::{fs::File, sync::mpsc, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}, time::timeout};
use tokio_util::sync::CancellationToken;
use serde_json::Value as JsonValue;

pub async fn run_writer(
    path: impl AsRef<Path>,
    mut rx: mpsc::Receiver<JsonValue>,
) {
    let path = path.as_ref();
    loop {
        if let Err(e) = write_file(path.to_path_buf(), &mut rx).await {
            log::error!("{}: {}", path.to_string_lossy(), e);
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn write_file(
    path: PathBuf,
    rx: &mut mpsc::Receiver<(Timestamp, String)>,
) -> Result<()> {
    log::debug!("{}: opening for append", path.to_string_lossy());
    let fd = File::create(path).await?;
    let writer = serde_jsonlines::AsyncJsonLinesWriter::new(fd);
    log::debug!("{}: starting to write", &path.to_string_lossy());
    loop {
        let (ts, data) = tbc::cancellable_recv(rx.recv(), &stop).await?;
        tbc::timebound_cancellable(
            out.write_all(format!("{} {}", ts, data).as_bytes()),
            timeout,
            &stop,
        ).await?;
    }
}