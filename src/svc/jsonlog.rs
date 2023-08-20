use crate::{
    err::{Error, Result},
    util::time::{Timespan, Timestamp},
};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
};

pub struct JsonLog<T> {
    path: PathBuf,
    writer: Option<BufWriter<File>>,
    reader: Option<BufReader<File>>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: std::fmt::Debug + Serialize + DeserializeOwned> JsonLog<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let phantom = std::marker::PhantomData;
        let reader = None;
        let writer = None;
        Self {
            path,
            phantom,
            reader,
            writer,
        }
    }

    pub async fn write(&mut self, ts: Timestamp, record: &T) -> Result<()> {
        let writer = if let Some(writer) = &mut self.writer {
            writer
        } else {
            let fd = File::options()
                .create(true)
                .append(true)
                .open(&self.path)
                .await
                .map_err(|e| {
                    Error::Other(format!(
                        "{}: can't append, {}",
                        self.path.to_string_lossy(),
                        e
                    ))
                })?;
            self.writer = Some(BufWriter::new(fd));
            self.writer.as_mut().unwrap()
        };
        let data = format!("{} {}\n", ts, serde_json::to_string(&record).unwrap()); // TODO: Fix this unwrap!
        if let Err(e) = writer.write(data.as_bytes()).await {
            self.close();
            Err(e)?
        } else {
            writer.flush().await?;
        }
        Ok(())
    }

    pub async fn query(&mut self, span: Timespan) -> Result<Vec<(Timestamp, T)>> {
        let reader = if let Some(reader) = &mut self.reader {
            reader
        } else {
            tokio::fs::create_dir_all(
                self.path
                    .parent()
                    .ok_or(Error::msg("event log name must have a parent directory"))?,
            )
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "{}: can't create, {}",
                    self.path.to_string_lossy(),
                    e
                ))
            })?;
            let fd = File::open(&self.path).await.map_err(|e| {
                Error::Other(format!(
                    "{}: can't read, {}",
                    self.path.to_string_lossy(),
                    e
                ))
            })?;

            self.reader = Some(BufReader::new(fd));
            self.reader.as_mut().unwrap()
        };
        let mut linenum = 0;
        let mut res = Vec::new();
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            linenum += 1;
            match Self::parse_line(line) {
                Ok((ts, rec)) => {
                    if span.includes(ts) {
                        res.push((ts, rec))
                    }
                }
                Err(e) => log::error!("{}, line {}: {}", self.path.to_string_lossy(), linenum, e),
            };
        }
        Ok(res)
    }

    fn parse_line(line: String) -> Result<(Timestamp, T)> {
        let (ts, data) = Self::split_line(line)?;
        let rec = serde_json::from_str::<T>(&data)?;
        Ok((ts, rec))
    }

    fn split_line(line: String) -> Result<(Timestamp, String)> {
        let mut split = line.splitn(2, " ");
        match (split.next().map(Timestamp::from_iso_string), split.next()) {
            (Some(Ok(ts)), Some(data)) => Ok((ts, data.to_string())),
            _ => Err(Error::msg(
                "expected timestamp and data separated by a space",
            )),
        }
    }

    pub fn close(&mut self) {
        self.reader = None;
        self.writer = None;
    }
}

#[cfg(never)]
pub async fn write_all(path: impl AsRef<Path>, mut rx: mpsc::Receiver<Event>) -> Result<()> {
    let path = path.as_ref();

    loop {
        if let Err(e) = try_write(path, &mut rx).await {
            log::error!("{}: {}", path.to_string_lossy(), e);
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

#[cfg(never)]
async fn try_write(path: &Path, rx: &mut mpsc::Receiver<Event>) -> Result<()> {
    let fd = File::options()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    let mut writer = tokio::io::BufWriter::new(fd);
    let mut last_flush_ts = Timestamp::now();
    // tokio::pin!(writer);
    while let Some(evt) = rx.recv().await {
        let str = serde_json::to_string(&evt)
            .map_err(|e| Error::Other(format!("failed to serialize {}", e)))?;
        writer.write(str.as_bytes()).await?;
        let now = Timestamp::now();
        if now.millis_between(last_flush_ts) > 1000 {
            writer.flush().await?;
            last_flush_ts = now;
        }
    }
    Ok(())
}
