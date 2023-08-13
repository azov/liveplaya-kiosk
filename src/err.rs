use geojson::JsonValue;
use thiserror;
use serde::{Serialize, Deserialize};

#[derive(thiserror::Error, Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    #[error("invalid clock value {0}:{1}")]
    BadClock(u16, u16),

    #[error("invalid latitude value {0}, must be in [-90,90] range")]
    BadLatitude(f64),

    #[error("invalid longitude value {0}, must be in [-180,180] range")]
    BadLongitude(f64),

    // #[error("failed to serialize: {0}")]
    // Serialize(#[from] serde_json::Error),
    #[error("can't read file; path={0}, error={1}")]
    // LoadFile(std::path::PathBuf, #[source] Box<dyn std::error::Error>),
    LoadFile(std::path::PathBuf, String),

    // #[error("can't parse time: {0}")]
    // ParseTime(#[from] chrono::ParseError),
    #[error("time out of range: {0}")]
    TimeOutOfRange(String),

    #[error("missing required argument {0}")]
    MissingRequiredArgument(String),

    #[error("APRS_TTY {tty}: {err}")]
    OpenTTY { tty: String, err: String },

    #[error("APRS_TTY {tty}: {err}")]
    ReadTTY { tty: String, err: String },

    #[error("{context}: disconnected")]
    Disconnected{context: String},

    #[error("{context}: busy")]
    Busy{context: String},

    #[error("{0}")]
    Other(String),

    #[error("{0}: {1}")]
    OtherWithContext(&'static str, String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn msg(v: impl ToString) -> Self {
        Error::Other(v.to_string())
    }

    pub fn disconnected(v: impl ToString) -> Self {
        Error::Disconnected{context: v.to_string()}
    }

    pub fn busy(v: impl ToString) -> Self {
        Error::Busy{context: v.to_string()}
    }

    pub fn to_json(&self) -> JsonValue {
        serde_json::json!({
            "status": "error",
            "message": self.to_string(),
        })
    }

}

#[cfg(feature = "wasm")]
impl Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(self.to_string().as_str())
    }
}


// Verify error types implement appropriate traits
const _: () = {
    fn assert_send_clone<T: Send + 'static + Clone>() {}
    fn assert_all() {
        assert_send_clone::<Error>();
        assert_send_clone::<Result<i32>>();
    }
};
