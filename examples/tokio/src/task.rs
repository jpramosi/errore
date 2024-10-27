use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use errore::*;

/// A wrapper around [`tokio::task::JoinError`]
/// to fulfil the trait implementation requirements.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct JoinError(#[from] tokio::task::JoinError);

impl Deref for Ec {
    type Target = tokio::task::JoinError;

    fn deref(&self) -> &Self::Target {
        &self.error().0
    }
}

/// A wrapper for [`tokio::task::JoinHandle`] that converts
/// a type of [`std::result::Result`] to [`errore::result::Result`].
///
/// This has the advantage that only one type of `Result` is used in the code.
///
/// However this is an optional construct and doesn't need to be used if not needed.
pub struct JoinHandle<T>(tokio::task::JoinHandle<T>);

impl<T> Future for JoinHandle<T> {
    type Output = errore::result::Result<T, Ec>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut Pin::into_inner(self).0)
            .poll(cx)
            .map(|f| match f {
                std::result::Result::Ok(ok) => Self::Output::Ok(ok),
                std::result::Result::Err(err) => Self::Output::Err(Ec::new(err.into())),
            })
    }
}

#[inline]
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    JoinHandle(tokio::task::spawn(future))
}
