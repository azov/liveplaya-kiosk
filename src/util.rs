pub use ::std::collections::{HashMap, VecDeque};
pub use ::std::fmt::{Debug, Display};

pub use ::serde::{Serialize, Deserialize, de::DeserializeOwned};
pub use ::serde_json::{Value as JsonValue, json};
pub use ::log;

use crate::{err::Error, err::Result};

fn to_json_value(v: impl Serialize + Debug + Clone, ctx: impl AsRef<str>) -> Result<JsonValue> {
    serde_json::value::to_value(v.clone())
        .map_err(|e| Error::msg(format!("failed to serialize {} {:?} to JSON: {}", ctx.as_ref(), v, e)))
}

fn from_json_value<'de, T: Debug + Serialize + DeserializeOwned + Clone>(v: JsonValue, ctx: impl AsRef<str>) -> Result<T> {
    serde_json::value::from_value(v.clone())
        .map_err(|e| Error::msg(format!("failed to deserialize {} {:?} from JSON: {}", ctx.as_ref(), v, e)))
}
