use crate::{
    err::{Error, Result},
};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;

pub type Query<In, Out> = (In, oneshot::Sender<Result<Out>>);
pub type Sender<In, Out> = mpsc::Sender<Query<In, Out>>;
pub type Receiver<In, Out> = mpsc::Receiver<Query<In, Out>>;

pub fn channel<In, Out>(bound: usize) -> (Sender<In, Out>, Receiver<In, Out>) {
    mpsc::channel::<(In, oneshot::Sender<Result<Out>>)>(bound)
}

pub async fn request<In, Out>(
    input: In,
    sender: &Sender<In, Out>,
    stop: &CancellationToken,
) -> Result<Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    let (out_tx, out_rx) = oneshot::channel::<Result<Out>>();
    match sender.try_send((input, out_tx)) {
        Ok(_) => recv(out_rx, &stop).await,
        Err(mpsc::error::TrySendError::Full(_)) => Err(Error::Busy /*("sending request")*/),
        Err(mpsc::error::TrySendError::Closed(_)) => {
            stop.cancel();
            Err(Error::Disconnected /*("sending request")*/)
        }
    }
}

async fn recv<T>(
    mut receiver: oneshot::Receiver<Result<T>>,
    stop: &CancellationToken,
) -> Result<T> {
    select! {
        _ = stop.cancelled() => Err(Error::Cancelled),
        r = &mut receiver => {
            match r {
                Ok(v) => v,
                Err(_) => Err(Error::Disconnected /*("receiving response")*/),
            }
        }
    }
}

pub fn respond<In, Out, F>(q: Query<In, Out>, f: F)
where
    F: FnOnce(In) -> Result<Out>,
{
    let (input, output) = q;
    let res = f(input);
    if let Err(_) = output.send(res) {
        log::debug!("did not send the response: client disconnected");
    }
}
