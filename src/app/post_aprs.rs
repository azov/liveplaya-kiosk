use super::*;
use crate::util::time::Timestamp;

impl Server {
    pub async fn post_aprs(&mut self, ts: Timestamp, data: String) -> Result<()> {
        self.aprs_cache.push(ts, data)
    }
}
