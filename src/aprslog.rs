use crate::{
    err::{Error, Result},
    io::Event,
    util::time::Timestamp,
};
use std::path::Path;
use tokio::{fs::File, io::AsyncBufReadExt, sync::mpsc};

pub async fn read(path: impl AsRef<Path>, tx: mpsc::Sender<Event>) -> Result<()> {
    let path = path.as_ref();
    let fd = File::open(path).await?;
    let mut lines = tokio::io::BufReader::new(fd).lines();
    let mut linenum = 0;
    while let Some(line) = lines.next_line().await? {
        linenum += 1;
        match parse_line(line).map(|l| send(l, &tx)) {
            Ok(_) => (),
            Err(e) => {
                log::error!("{}, line {}: {}", path.to_string_lossy(), linenum, e);
            }
        }
    }
    Ok(())
}

fn parse_line(v: String) -> Result<Event> {
    let mut split = v.splitn(2, " ");
    match (split.next().map(Timestamp::from_iso_string), split.next()) {
        (Some(Ok(ts)), Some(data)) => Ok(Event::AprsPacketLoaded { ts, data: data.to_string() }),
        _ => Err(Error::msg(
            "expected timestamp and data separated by a space",
        )),
    }
}

async fn send(evt: Event, tx: &mpsc::Sender<Event>) -> Result<()> {
    if let Err(_e) = tx.send(evt).await {
        Err(Error::msg("aprs log receiver disconnected"))
    } else {
        Ok(())
    }
}
