use std::{
    future::Future,
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{stream::Once, Stream};
use serde_json::Value;

use crate::error::ExecError;

use super::Body;

impl Body for Box<dyn Body + Send + '_> {
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Value, ExecError>>> {
        let inner = Pin::into_inner(self).deref_mut();

        // SAFETY: This is the same implementation as std::pin::pin!().
        // We're not using it as it uses a block rather than &mut-ing the inner value directly.
        #[allow(unsafe_code)]
        let inner = unsafe { Pin::new_unchecked(inner) };

        inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (&**self).size_hint()
    }
}

impl<Fut: Future<Output = Result<Value, ExecError>>> Body for Once<Fut> {
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Value, ExecError>>> {
        Stream::poll_next(self, cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        Stream::size_hint(self)
    }
}
