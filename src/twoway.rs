use crate::err::{Error, Result};
use crate::util::*;
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
    In: Debug + Send + 'static,
    Out: Debug + Send + 'static,
{
    let (out_tx, out_rx) = oneshot::channel::<Result<Out>>();
    match sender.try_send((input, out_tx)) {
        Ok(_) => recv(out_rx, &stop).await,
        Err(mpsc::error::TrySendError::Full(_)) => Err(Error::busy("sending request")),
        Err(mpsc::error::TrySendError::Closed(_)) => {
            stop.cancel();
            Err(Error::disconnected("sending request"))
        }
    }
}

async fn recv<T>(
    mut receiver: oneshot::Receiver<Result<T>>,
    stop: &CancellationToken,
) -> Result<T> {
    select! {
        _ = stop.cancelled() => Err(Error::cancelled("receiving response")),
        r = &mut receiver => {
            match r {
                Ok(v) => v,
                Err(_) => Err(Error::disconnected("receiving response")),
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

// #[derive(Debug, Clone)]
// pub struct Request<In, Out> {
//     input: In,
//     output: tokio::sync::mpsc::Sender<Result<Out>>,
// }
// impl<In, Out> Request<In, Out>
// where
//     In: Debug + Send + 'static,
//     Out: Debug + Send + 'static,
// {
//     pub fn new(input: In) -> (Self, Receiver<Out>) {
//         let (tx, rx) = tokio::sync::mpsc::channel::<Result<Out>>(1);
//         (Self { input, output: tx }, Receiver::<Out> { output: rx })
//     }
//     pub fn fulfill(self, f: impl FnOnce(In) -> Result<Out>) -> Result<()> {
//         let Self { input, output } = self;
//         let res = f(input);
//         output
//             .try_send(res)
//             .map_err(|_| Error::disconnected("failed to send response"))
//     }
// }

// pub struct Receiver<T> {
//     output: tokio::sync::mpsc::Receiver<Result<T>>,
// }
// impl<T> Receiver<T> {
//     pub async fn recv(mut self) -> Result<T> {
//         match self.output.recv().await {
//             Some(Result::Ok(res)) => Ok(res),
//             Some(Result::Err(e)) => Err(e)?,
//             None => Err(Error::disconnected("can't receive result")),
//         }
//     }
// }
