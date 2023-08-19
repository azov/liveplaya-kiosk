use crate::{
    aprs,
    bm::BurningMan,
    err::Result,
    io,
    util::{time::Timestamp, twoway},
};
use geojson;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;

struct Worker {
    bm: BurningMan,
    log: aprs::Log,
}

impl Worker {
    pub fn new() -> Self {
        let bm: BurningMan = crate::bmorg::data_for_year(2023)
            .unwrap()
            .try_into()
            .unwrap();
        let log = aprs::Log::new();
        Self { bm, log }
    }

    pub fn view(&self, _q: io::Query) -> Result<io::View> {
        let log = &self.log;
        let mut last_positions = log.last_positions().collect::<Vec<&(Timestamp, aprs::PositionReport)>>();
        let now = Timestamp::now();

        let map = if let Some(city) = self.bm.city() {
            let mut features: Vec<geojson::Feature> = vec![];
            for (_ts, pr) in log.last_positions() {
                let loc = pr.pos.location;
                features.push(geojson::Feature {
                    geometry: Some(geojson::Geometry {
                        bbox: None,
                        value: loc.into(),
                        foreign_members: None,
                    }),
                    bbox: None,
                    id: None,
                    foreign_members: None,
                    properties: Some(
                        serde_json::json!({
                            "liveplaya": "poi",
                            "poi": "beacon",
                            "name": pr.src_callsign,
                            "location": city.rgeocode(loc),
                        })
                        .as_object()
                        .unwrap()
                        .clone(),
                    ),
                });
            }
            for f in city.other_features() {
                features.push(f.clone());
            }

            let data = geojson::FeatureCollection {
                bbox: None,
                foreign_members: None,
                features,
            };

            last_positions.sort_by_key(|(ts, pr)| {
                let man_dist = pr.pos.location.haversine_distance_m(city.center());
                let near_brc = man_dist < 10000.;
                let seen_recently = ts.duration_between(now).as_secs() < 3600*3;
                (near_brc, seen_recently, &pr.src_callsign) 
            });
    
            Some(io::Map {
                bearing_deg: 45.,
                center: city.center().lnglat(),
                zoom: 13.,
                data,
            })
        } else {
            None
        };

        let mut logmsgs = Vec::new();
        for (id, ts, raw, parsed) in self.log.recent_entries() {
            match parsed {
                Ok(aprs::Packet::Position(_pr)) => {
                    logmsgs.push(io::LogMessage::Info {
                        id: *id,
                        time: *ts,
                        text: format!("{}", raw),
                    });
                }
                Err(e) => {
                    logmsgs.push(io::LogMessage::Error {
                        id: *id,
                        time: *ts,
                        text: format!("{}: {}", raw, e),
                    });
                }
            }
        }


        let mut refs = Vec::new();
        for (ts, pr) in last_positions {
            let location = if let Some(city) = self.bm.city() {
                city.rgeocode(pr.pos.location)
            } else {
                pr.pos.location.to_string()
            };

            refs.push(io::FeatureRef::Beacon {
                name: pr.src_callsign.clone(),
                slug: format!("aprs/{}", pr.src_callsign.to_ascii_lowercase()),
                lastseen: *ts,
                location,
            })
        }

        Ok(io::View {
            name: format!("Black Rock City {}", self.bm.year()),
            description: self.bm.theme().map(|theme| {
                format!(
                    "The site of Burning Man event '{}'\nThere's {} APRS stations.",
                    theme,
                    self.log.station_count()
                )
            }),
            time: Timestamp::now(),
            map,
            log: logmsgs,
            refs,
        })
    }

    pub fn post_aprs(&mut self, ts: Timestamp, data: String) -> Result<()> {
        self.log.push(ts, data)
    }
}

pub async fn run(
    mut query_rx: io::QueryReceiver,
    mut aprs_rx: mpsc::Receiver<String>,
    log_tx: mpsc::Sender<(Timestamp, String)>,
    stop: CancellationToken,
) {
    let mut worker = Worker::new();

    loop {
        select! {
            _ = stop.cancelled() => break,
            Some(q) = query_rx.recv() => {
                twoway::respond(q, |q| {
                    worker.view(q)
                });
            },
            res = aprs_rx.recv() => {
                match res {
                    Some(packet_data) => {
                        let now = Timestamp::now();
                        let _ = worker.post_aprs(now, packet_data.clone());
                        let _ = log_tx.send((now, packet_data)).await;
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
