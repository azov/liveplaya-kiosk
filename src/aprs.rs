use crate::core::*;

use ::aprs::Packet as PacketTrait;
pub use ::aprs::Symbol as AprsSymbol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Position(PositionReport),
}
impl Packet {
    pub fn parse(data: String) -> Result<Self> {
        let res = ::fap::Packet::new(data).map_err(|e| Error::Other(e.to_string()))?;

        let srccall = res.source();
        let pos = res.position();
        let speed = res.speed();
        let course = res.course();
        let _symbol = res.symbol();
        let _dstcsll = res.destination();
        let _altitude = res.altitude();
        let comment = res.comment();

        let pos = match pos {
            None => return Err(Error::msg("not a position update")),
            Some(pos) => Ok(Position {
                location: Point::new(pos.longitude as f64, pos.latitude as f64)?,
                location_error_m: pos.precision.map(|v| ft2m(v.as_f64())),
                heading_deg: course.map(|v| v.as_f64()),
                speed_mps: speed.map(|v| ::aprs::MetersPerSecond::from(v).as_f64()),
            }),
        }?;

        Ok(Self::Position(PositionReport { 
            srccall: srccall.into(),
            pos,
            // symbol,
            comment: comment.map(|v| v.into()),
        }))
    }

    pub fn srccall(&self) -> &str {
        match self {
            Packet::Position(PositionReport{ srccall, .. }) => srccall.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionReport {
    pub srccall: String,
    pub pos: Position,
    // symbol: (u8,u8),
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry{
    ts: Timestamp, 
    data: String,
    parsed: std::result::Result<Packet, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    /// A map between callsigns and most recent position updates
    lastpos: HashMap<String, (Timestamp, PositionReport)>,

    /// Most recent log entries
    log: VecDeque<LogEntry>,

    /// Max number of entries to store
    maxlen: usize,
}

impl Log {
    pub fn new() -> Self {
        let maxlen = 32;
        Self {
            lastpos: HashMap::new(),
            log: VecDeque::with_capacity(maxlen),
            maxlen,
        }
    }

    pub fn add(&mut self, ts: Timestamp, data: String) -> Result<()> {
        if let Some(last_entry) = self.log.back() {
            let last_ts = last_entry.ts;
            if last_ts > ts {
                return Err(Error::msg(format!(
                    "packets are supposed to be in chronological order, {} < {}",
                    last_ts, ts
                )));
            }
        }

        while self.log.len() >= self.maxlen - 1 {
            self.log.pop_front();
        }

        let parsed = Packet::parse(data.clone());
        if let Ok(Packet::Position(report)) = &parsed {
            self.lastpos.insert(report.srccall.clone(), (ts, report.clone()));
        }
        self.log.push_back(LogEntry{ts, data, parsed:parsed.map_err(|e| e.to_string())});

        Ok(())
    }

    pub fn merge(&mut self, other: Self) {
        for (other_call, (other_ts, other_pos)) in other.lastpos.into_iter() {
            match self.lastpos.get(&other_call) {
                Some((ts, _pos)) if other_ts <= *ts => {
                    () // do nothing, we already have more recent position
                }
                _ => {
                    self.lastpos
                        .insert(other_call, (other_ts, other_pos));
                }
            }
        }

        let mut it = self.log.iter().peekable();
        let mut otherit = other.log.iter().peekable();
        let mut newlog = VecDeque::with_capacity(self.maxlen);
        loop {
            match (it.peek(), otherit.peek()) {
                (Some(&entry), Some(&otherentry)) if entry.ts < otherentry.ts => {
                    newlog.push_back(entry);
                    it.next();
                }
                (Some(&_entry), Some(&otherentry)) => {
                    newlog.push_back(otherentry);
                    otherit.next();
                }
                (None, Some(&_otherentry)) => {
                    newlog.extend(otherit);
                    break;
                }
                (Some(&_entry), None) => {
                    newlog.extend(it);
                    break;
                }
                (None, None) => {
                    break;
                }
            }
        }
    }

    pub fn filter(&self, bounds: Option<BBox>) -> Self {
        match bounds {
            None => self.clone(),
            Some(bounds) => {
                let lastpos: HashMap<String, (Timestamp, PositionReport)> = self
                    .lastpos
                    .iter()
                    .filter(|(_k, (_ts, report))| bounds.contains(report.pos.location))
                    .map(|v| (v.0.clone(), v.1.clone()))
                    .collect();

                let log = self
                    .log
                    .iter()
                    .filter(|&v| match v {
                        LogEntry{ parsed: Ok(packet), .. } => {
                            lastpos.contains_key(packet.srccall())
                        }
                        _ => true,
                    })
                    .map(|v| v.clone())
                    .collect();

                Self {
                    log,
                    lastpos,
                    maxlen: self.maxlen,
                }
            }
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = &(Timestamp, PositionReport)> {
        self.lastpos.values()
    }

    pub fn station_count(&self) -> usize {
        self.lastpos.len()
    }
}
