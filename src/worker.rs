use crate::{
    err::{Error, Result},
    io, map, twoway,
    time::Timestamp,
};
use serde_json::Value as JsonValue;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;

struct Worker {
    data: map::Data,
}

impl Worker {
    pub fn new() -> Self {
        let data = map::Data::new();
        Self { data }
    }

    pub fn get_map(&self, q: map::Query) -> Result<map::Snapshot> {
        Ok(self.data.snapshot(q.feature))
    }
    
    pub fn post_aprs(&mut self, packet: String) -> Result<()> {
        let now = Timestamp::now();
        self.data.update_aprs(now, packet)
    }
}


pub async fn run(
    mut query_rx: twoway::Receiver<JsonValue, JsonValue>,
    mut aprs_rx: mpsc::Receiver<String>,
    stop: CancellationToken,
) {

    let worker = Worker{ data: map::Data::new()};

    loop {
        select! {
            _ = stop.cancelled() => break,
            Some(q) = query_rx.recv() => {
                twoway::respond(q, |q| {
                    let query : map::Query = serde_json::from_value(q).map_err(|e| Error::BadRequest(e.to_string()))?;
                    let res = worker.get_map(query);
                    res.map(|v| v.to_json())
                });
            },
            res = aprs_rx.recv() => {
                match res {
                    Some(packet_data) => {
                        log::debug!("processing aprs packet: {}", packet_data);
                    },
                    None => {
                        stop.cancel();
                    }
                }
            }
        };
    }
    log::debug!("worker finished")
}


