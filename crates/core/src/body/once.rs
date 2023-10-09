use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use pin_project_lite::pin_project;
use serde_json::Value;

use crate::error::ExecError;

use super::Body;

// This type was taken from futures_util so all credit to it's original authors!
pin_project! {
    /// A stream which emits single element and then EOF.
    #[must_use = "streams do nothing unless polled"]
    pub(crate) struct Once<Fut> {
        #[pin]
        future: Option<Fut>
    }
}

impl<Fut> Once<Fut> {
    pub fn new(future: Fut) -> Self {
        Self {
            future: Some(future),
        }
    }
}

impl<Fut: Future<Output = Result<Value, ExecError>>> Body for Once<Fut> {
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Value, ExecError>>> {
        let mut this = self.project();
        let v = match this.future.as_mut().as_pin_mut() {
            Some(fut) => ready!(fut.poll(cx)),
            None => return Poll::Ready(None),
        };

        this.future.set(None);
        Poll::Ready(Some(v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.future.is_some() {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}
