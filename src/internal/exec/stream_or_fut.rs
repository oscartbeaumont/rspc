use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream};
use pin_project_lite::pin_project;

use crate::internal::{exec, PinnedOption, PinnedOptionProj};

use super::{ExecRequestFut, OwnedStream};

mod private {
    use super::*;

    pin_project! {
        /// TODO
        #[project = StreamOrFutProj]
        pub enum StreamOrFut<TCtx> {
            Stream {
                #[pin]
                stream: OwnedStream<TCtx>
            },
            Future {
                #[pin]
                fut: ExecRequestFut,
            },
            // When the underlying stream shutdowns we yield a shutdown message. Once it is yielded we need to yield a `None` to tell the poller we are done.
            PendingDone {
                id: u32
            },
            Done { id: u32 },
        }
    }

    impl<TCtx: 'static> StreamOrFut<TCtx> {
        pub fn id(&self) -> u32 {
            match self {
                StreamOrFut::Stream { stream } => stream.id,
                StreamOrFut::Future { fut } => fut.id,
                StreamOrFut::PendingDone { id } => *id,
                StreamOrFut::Done { id } => *id,
            }
        }
    }

    impl<TCtx: 'static> Stream for StreamOrFut<TCtx> {
        type Item = exec::Response;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.as_mut().project() {
                StreamOrFutProj::Stream { mut stream } => {
                    Poll::Ready(Some(match ready!(stream.as_mut().poll_next(cx)) {
                        Some(r) => exec::Response {
                            id: stream.id,
                            inner: match r {
                                Ok(v) => exec::ResponseInner::Value(v),
                                Err(err) => exec::ResponseInner::Error(err.into()),
                            },
                        },
                        None => {
                            let id = stream.id;
                            cx.waker().wake_by_ref(); // No wakers set so we set one
                            self.set(StreamOrFut::PendingDone { id });
                            exec::Response {
                                id,
                                inner: exec::ResponseInner::Complete,
                            }
                        }
                    }))
                }
                StreamOrFutProj::Future { fut } => {
                    let id = fut.id;
                    fut.poll(cx).map(|v| {
                        cx.waker().wake_by_ref(); // No wakers set so we set one
                        self.set(StreamOrFut::PendingDone { id });
                        Some(v)
                    })
                }
                StreamOrFutProj::PendingDone { id } => {
                    let id = *id;
                    self.set(StreamOrFut::Done { id });
                    Poll::Ready(None)
                }
                StreamOrFutProj::Done { .. } => {
                    #[cfg(debug_assertions)]
                    panic!("`StreamOrFut` polled after completion");

                    #[cfg(not(debug_assertions))]
                    Poll::Ready(None)
                }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                StreamOrFut::Stream { stream } => stream.size_hint(),
                StreamOrFut::Future { .. } => (0, Some(1)),
                StreamOrFut::PendingDone { .. } => (0, Some(0)),
                StreamOrFut::Done { .. } => (0, Some(0)),
            }
        }
    }
}

#[cfg(feature = "unstable")]
pub use private::StreamOrFut;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::StreamOrFut;
