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

