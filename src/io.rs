pub use crate::util::*;
pub use crate::core::Timestamp;
pub use crate::sync::{Request, TaskProcessor, TaskQueue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub feature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct View {
    pub name: String,
    pub unix_timestamp_ms: u64,
    pub description: Option<String>,
    pub map: Option<JsonValue>, // maplibre style
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AprsPacket {
    pub received: Timestamp,
    pub data: String,
}

#[derive(Debug, Clone)]
pub enum Task {
    Shutdown,
    GetView(Request<Query, View>),
    PostAprs(AprsPacket),
}

pub type Tasks = crate::sync::TaskQueue<Task>;
