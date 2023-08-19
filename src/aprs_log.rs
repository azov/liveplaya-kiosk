use crate::{err::Result, util::tbc, util::time::Timestamp};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::{fs::File, sync::mpsc, io::AsyncWriteExt};
use tokio_util::sync::CancellationToken;

pub async fn run_writer(
    path: impl AsRef<Path>,
    mut rx: mpsc::Receiver<(Timestamp, String)>,
    stop: CancellationToken,
) {
    let path = path.as_ref();
    while !stop.is_cancelled() {
        if let Err(e) = run_session(path.to_path_buf(), &mut rx, stop.clone()).await {
            log::error!("{}: {}", path.to_string_lossy(), e);
        }
        _ = tbc::sleep(Duration::from_secs(3), &stop).await;
    }
}

async fn run_session(
    path: PathBuf,
    rx: &mut mpsc::Receiver<(Timestamp, String)>,
    stop: CancellationToken,
) -> Result<()> {
    log::debug!("{}: opening for append", path.to_string_lossy());
    let timeout = Duration::from_secs(3);
    let mut fopt = File::options();
    fopt.create(true).append(true);
    let mut out = tbc::timebound_cancellable(fopt.open(&path), timeout, &stop).await?;

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
