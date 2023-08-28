use crate::{
    err::{Result, Error},
    io::aprs::{Packet, PositionReport},
    motion::Position,
    util::{
        geo::Point,
        time::Timestamp,
        units::ft2m,
    },
};
use ::aprs::Packet as PacketTrait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};


#[derive(Debug, Clone)]
pub struct Log {
    /// A map between callsigns and most recent position updates
    lastpos: HashMap<String, (Timestamp, PositionReport)>,

    /// Most recent packets entries
    recent: VecDeque<(u64, Timestamp, String, Result<Packet>)>,

    /// Max number of entries to store
    maxlen: usize,

    /// Next available id
    nextid: u64,
}

impl Log {
    pub fn new() -> Self {
        let maxlen = 32;
        Self {
            lastpos: HashMap::new(),
            recent: VecDeque::with_capacity(maxlen),
            maxlen,
            nextid: 1,
        }
    }

    pub fn push(&mut self, ts: Timestamp, data: String) -> Result<()> {
        if let Some((_msgid, last_ts, _last_data, _last_res)) = self.recent.back() {
            if last_ts > &ts {
                return Err(Error::AprsOrder {
                    new_ts: ts.to_string(),
                    last_ts: (*last_ts).to_string(),
                });
            }
        }

        while self.recent.len() >= self.maxlen - 1 {
            self.recent.pop_front();
        }

        let parsed = Packet::parse(data.clone());
        if let Ok(Packet::Position(report)) = &parsed {
            self.lastpos
                .insert(report.src_callsign.clone(), (ts, report.clone()));
        }
        let id = self.nextid;
        self.nextid += 1;
        self.recent.push_back((id, ts, data, parsed));
        Ok(())
    }

    #[cfg(never)]
    pub fn merge(&mut self, other: Self) {
        for (other_call, (other_ts, other_pos)) in other.lastpos.into_iter() {
            match self.lastpos.get(&other_call) {
                Some((ts, _pos)) if other_ts <= *ts => {
                    () // do nothing, we already have more recent position
                }
                _ => {
                    self.lastpos.insert(other_call, (other_ts, other_pos));
                }
            }
        }

        let mut it = self.recent.iter().peekable();
        let mut otherit = other.recent.iter().peekable();
        let mut newlog = VecDeque::with_capacity(self.maxlen);
        loop {
            match (it.peek(), otherit.peek()) {
                (Some(&entry), Some(&otherentry)) if entry.0 < otherentry.0 => {
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

    #[cfg(never)]
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
                    .recent
                    .iter()
                    .filter(|&v| match &v.2 {
                        Ok(packet) => lastpos.contains_key(packet.srccall()),
                        _ => true,
                    })
                    .map(|v| v.clone())
                    .collect();

                Self {
                    recent: log,
                    lastpos,
                    maxlen: self.maxlen,
                    nextid: self
                }
            }
        }
    }

    pub fn station_count(&self) -> usize {
        self.lastpos.len()
    }

    pub fn last_positions(&self) -> impl Iterator<Item = &(Timestamp, PositionReport)> {
        self.lastpos.values()
    }

    pub fn recent_entries(&self) -> impl Iterator<Item = &(u64, Timestamp, String, Result<Packet>)> {
        self.recent.iter()
    }
}
