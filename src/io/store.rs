use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Record {
    #[serde(alias = "aprs-packet")]
    #[serde(rename = "aprs")]
    AprsPacket { data: String },
}
