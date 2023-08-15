pub use crate::util::*;
pub use crate::core::Timestamp;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct View {
    pub name: String,
    pub unix_timestamp_ms: u64,
    pub description: Option<String>,
    pub map: Option<JsonValue>, // maplibre style
}
