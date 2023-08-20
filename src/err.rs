use geojson::JsonValue;
use serde::{Deserialize, Serialize};
use thiserror;

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

    #[error("can't parse APRS packet {what}: {why}")]
    AprsParse { what: String, why: String },

    #[error("APRS packets are supposed to be in chronological order; {new_ts} < {last_ts}")]
    AprsOrder {
        new_ts: String,
        last_ts: String,
    },

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("I/O error: {0}")]
    IOError(String),

    #[error("{msg}")]
    OutOfRange{msg: String},

    #[error("timed out")]
    TimedOut,

    #[error("disconnected")]
    Disconnected,

    #[error("busy")]
    Busy,

    #[error("cancelled")]
    Cancelled,

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

    pub fn to_json(&self) -> JsonValue {
        serde_json::json!({
            "status": "error",
            "message": self.to_string(),
        })
    }
}


pub trait LogResult {
    fn log_result(self);
}

impl<E: std::fmt::Display> LogResult for std::result::Result<(), E> {
    fn log_result(self) {
        if let Err(e) = self {
            log::error!("{}", e);
        }
    }
}


impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::TimedOut {
            Self::TimedOut
        } else {
            Self::IOError(e.to_string())
        }
    }
}

impl<T> std::convert::From<tokio::sync::mpsc::error::TrySendError<T>> for Error {
    fn from(value: tokio::sync::mpsc::error::TrySendError<T>) -> Self {
        match value {
            tokio::sync::mpsc::error::TrySendError::Full(_) => Self::Busy,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => Self::Disconnected,
        }
    }
}

impl<T> std::convert::From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        match value {
            tokio::sync::mpsc::error::SendError(_v) => Self::Disconnected,
        }
    }
}

impl std::convert::From<tokio::time::error::Elapsed> for Error {
    fn from(_value: tokio::time::error::Elapsed) -> Self {
        Error::msg("timed out")
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Other(format!("failed to serialize {:?}", value))
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
