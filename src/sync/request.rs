use crate::err::{Error, Result};
use crate::util::*;

#[derive(Debug, Clone)]
pub struct Request<In, Out> {
    input: In,
    output: tokio::sync::mpsc::Sender<Result<Out>>,
}
impl<In, Out> Request<In, Out>
where
    In: Debug + Send + 'static,
    Out: Debug + Send + 'static,
{
    pub fn new(input: In) -> (Self, Receiver<Out>) {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<Out>>(1);
        (
            Self { input, output: tx },
            Receiver::<Out> { output: rx },
        )
    }
    pub fn fulfill(self, f: impl FnOnce(In) -> Result<Out>) -> Result<()>
    {
        let Self { input, output } = self;
        let res = f(input);
        output
            .try_send(res)
            .map_err(|_| Error::disconnected("failed to send response"))
    }
}

pub struct Receiver<T> {
    output: tokio::sync::mpsc::Receiver<Result<T>>,
}
impl<T> Receiver<T> {
    pub async fn recv(mut self) -> Result<T> {
        match self.output.recv().await {
            Some(Result::Ok(res)) => Ok(res),
            Some(Result::Err(e)) => Err(e)?,
            None => Err(Error::disconnected("can't receive result")),
        }
    }
}
