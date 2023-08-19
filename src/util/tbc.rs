use crate::err::{Error, Result};
use std::future::Future;
use tokio_util::sync::CancellationToken;

/// Cancellable I/O operation.
pub async fn cancellable<F, T, E>(
    fut: F,
    stop: &CancellationToken,
) -> Result<T>
where
    F: Future<Output = std::result::Result<T, E>>,
    E: Into<crate::err::Error>,
{
    tokio::select! {
        _ = stop.cancelled() => Err(Error::Cancelled),
        res = fut => {
            match res {
                Ok(v) => Ok(v),
                Err(e) => Err(e.into())
            }
        }
    }
}


/// Time-bounded cancellable I/O operation.
/// This version expects a future that returns a Result
pub async fn timebound_cancellable<F, T, E>(
    fut: F,
    timeout: std::time::Duration,
    stop: &CancellationToken,
) -> Result<T>
where
    F: Future<Output = std::result::Result<T, E>>,
    E: Into<crate::err::Error>,
{
    tokio::select! {
        _ = stop.cancelled() => Err(Error::Cancelled),
        _ = tokio::time::sleep(timeout) => Err(Error::TimedOut),
        res = fut => {
            match res {
                Ok(v) => Ok(v),
                Err(e) => Err(e.into())
            }
        }
    }
}


/// Time-bounded cancellable recv operation
pub async fn recv<F, T>(fut: F, timeout: std::time::Duration, stop: &CancellationToken) -> Result<T>
where
    F: Future<Output = Option<T>>,
{
    tokio::select! {
        _ = stop.cancelled() => Err(Error::Cancelled),
        _ = tokio::time::sleep(timeout) => Err(Error::TimedOut),
        res = fut => {
            match res {
                Some(v) => Ok(v),
                None => Err(Error::Disconnected)
            }
        }
    }
}

/// Time-bounded cancellable recv operation
pub async fn cancellable_recv<F, T>(fut: F, stop: &CancellationToken) -> Result<T>
where
    F: Future<Output = Option<T>>,
{
    tokio::select! {
        _ = stop.cancelled() => Err(Error::Cancelled),
        res = fut => {
            match res {
                Some(v) => Ok(v),
                None => Err(Error::Disconnected)
            }
        }
    }
}

/// Cancellable sleep
pub async fn sleep(timeout: std::time::Duration, stop: &CancellationToken) -> Result<()> {
    tokio::select! {
        _ = stop.cancelled() => Err(Error::Cancelled),
        _ = tokio::time::sleep(timeout) => Err(Error::TimedOut),
    }
}
