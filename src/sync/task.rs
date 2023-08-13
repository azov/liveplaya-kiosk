use crate::err::{Error, Result};
use crate::svc::JoinHandle;
use crate::util::*;

pub trait TaskProcessor<T>: Sized + Send + 'static
where
    T: Debug + Send + 'static,
{
    fn process(&mut self, task: T) -> Result<()>;

    fn spawn(mut self, bufsize: usize) -> (TaskQueue<T>, JoinHandle) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<T>(bufsize);
        let client = TaskQueue(tx.clone());
        let joiner = std::thread::spawn(move || {
            while let Some(task) = rx.blocking_recv() {
                if let Err(e) = self.process(task) {
                    log::error!("task processing failed:{}", e);
                    break;
                }
            }
            log::debug!("exiting worker thread");
            Ok(())
        });
        (client, joiner)
    }
}

#[derive(Debug, Clone)]
pub struct TaskQueue<T>(tokio::sync::mpsc::Sender<T>);
impl<T> TaskQueue<T>
where
    T: Debug + Send,
{
    pub fn submit(&self, task: T) -> Result<()> {
        match self.0.try_send(task) {
            Ok(_) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                Err(Error::disconnected("can't submit task"))
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                Err(Error::busy("can't submit task"))
            }
        }
    }
}
