//! Helpers for finite response streams and request deadlines.

use std::future::Future;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

pub fn one_reply<T: Send + 'static>(
    work: impl Future<Output = Result<T, Status>> + Send + 'static,
) -> ReceiverStream<Result<T, Status>> {
    let (sender, receiver) = mpsc::channel(8);
    tokio::spawn(async move {
        tokio::select! {
            _ = sender.closed() => {},
            reply = work => { let _ = sender.send(reply).await; }
        }
    });
    ReceiverStream::new(receiver)
}

pub async fn deadline<T>(work: impl Future<Output = Result<T, Status>>) -> Result<T, Status> {
    tokio::select! {
        reply = work => reply,
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            Err(Status::deadline_exceeded("downstream deadline exceeded"))
        }
    }
}
