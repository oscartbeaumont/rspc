//! TODO
//!
//! TODO: Rename this file

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use serde::Serialize;
use serde_json::Value;

#[cfg(feature = "tracing")]
type Inner = tracing::Span;

#[cfg(not(feature = "tracing"))]
type Inner = ();

#[allow(unused_imports)] // TODO: Fix this
pub(crate) use private::*;

pub(crate) mod private {
    use pin_project_lite::pin_project;

    use crate::{
        body::Body,
        error::{private::IntoResolverError, ExecError},
    };

    use super::*;

    pin_project! {
        pub struct StreamToBody<S> {
            #[pin]
            pub(crate) stream: S,
            pub(crate) span: Option<Inner>
        }
    }

    impl<
            S: Stream<Item = Result<T, TErr>> + Send + 'static,
            T: Serialize + 'static,
            TErr: IntoResolverError,
        > Body for StreamToBody<S>
    {
        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Value, ExecError>>> {
            let this = self.project();

            #[cfg(feature = "tracing")]
            let _span = this.span.as_ref().map(|s| s.enter());

            match ready!(this.stream.poll_next(cx)) {
                Some(Ok(v)) => Poll::Ready(Some(
                    serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
                )),
                Some(Err(e)) => {
                    Poll::Ready(Some(Err(ExecError::Resolver(e.into_resolver_error()))))
                }
                None => Poll::Ready(None),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.stream.size_hint()
        }
    }
}
