use std::{
    fmt,
    future::Future,
    marker::PhantomPinned,
    mem::{self, transmute},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_core::{FusedStream, Stream};

use crate::{executor::RequestContext, Format, Procedure, Serializer};

/// TODO
pub struct Task<F: Format> {
    /// The unique identifier of the task.
    /// Be careful storing these as they may be reused after the task is completed.
    pub id: u32,
    /// Whether the task requires queuing.
    /// If this is `false` you can safely block inline until it's done.
    /// For example a streaming query would be `false` while a subscription would be `true`.
    pub requires_queuing: bool,
    // Task must be `'static` so it can be queued onto an async runtime like Tokio.
    // However this borrows from the `Arc` in `Executor`.
    // TODO: Finish the explanation
    inner: TaskRepr<F::Serializer>,
    format: Arc<F>,
}

enum TaskRepr<S> {
    Procedure(Procedure),
    Future {
        serializer: S,
        fut: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
        // We have a reference to `serializer` inside `fut` so it would be unsafe to move pull this struct out of it's `Pin`.
        phantom: PhantomPinned,
    },
    Done,
}

impl<F: Format> Task<F> {
    pub(crate) fn new(procedure: Procedure, format: Arc<F>) -> Self {
        Self {
            id: 0,                   // TODO: Get id from somewhere
            requires_queuing: false, // TODO: Make this work
            inner: TaskRepr::Procedure(procedure),
            format,
        }
    }
}

impl<F: Format> fmt::Debug for Task<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl<F: Format> Stream for Task<F> {
    type Item = F::Result;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match &self.as_mut().inner {
                TaskRepr::Procedure(_) => {
                    let y = TaskRepr::Future {
                        serializer: self.format.serializer(),
                        fut: None,
                        phantom: PhantomPinned,
                    };

                    let procedure = {
                        // TODO: Discuss why this is safe and required (PhantomPinned)
                        let inner = unsafe { &mut self.as_mut().get_unchecked_mut().inner };

                        match mem::replace(inner, y) {
                            TaskRepr::Procedure(procedure) => procedure,
                            _ => unreachable!(),
                        }
                    };

                    // TODO: Make this unsafe block wayyyy smaller
                    unsafe {
                        // TODO: Can we do this with safe code
                        let (serializer, fut) = match &mut self.as_mut().get_unchecked_mut().inner {
                            TaskRepr::Future {
                                serializer, fut, ..
                            } => (serializer, fut),
                            _ => unreachable!(),
                        };

                        // TODO: Is this safe???
                        // let y: Pin<&mut dyn TODOSerializer> = Pin::new_unchecked(serializer);

                        let result: Serializer<'_> =
                            Serializer::new(Pin::new_unchecked(serializer));

                        // let result = Serializer::new(Pin::new_unchecked(serializer));
                        *fut = Some((procedure)(RequestContext {
                            result: transmute(result), // TODO: Make sure we hardcode the output to ensure it's only faking lifetimes
                        }));
                    };
                }
                TaskRepr::Future { .. } => {
                    // TODO: Safety
                    let (fut, serializer) = unsafe {
                        match &mut self.as_mut().get_unchecked_mut().inner {
                            TaskRepr::Future {
                                fut, serializer, ..
                            } => (
                                match fut {
                                    Some(fut) => fut,
                                    None => unreachable!(),
                                },
                                serializer,
                            ),
                            _ => unreachable!(),
                        }
                    };

                    let mut done = false; // TODO: Remove `done` flag and use `self`
                    let mut pending = false;

                    match fut.as_mut().poll(cx) {
                        Poll::Ready(()) => done = true,
                        Poll::Pending => pending = true,
                    };

                    let result = match F::into_result(serializer) {
                        Some(result) => Poll::Ready(Some(result)),
                        None => {
                            if done {
                                Poll::Ready(None)
                            } else if pending {
                                Poll::Pending
                            } else {
                                continue; // Will skip `if done` check and that's ok
                            }
                        }
                    };

                    if done {
                        unsafe { self.as_mut().get_unchecked_mut().inner = TaskRepr::Done };
                    }

                    return result;
                }
                TaskRepr::Done => return Poll::Ready(None),
            }
        }
    }
}

impl<F: Format> FusedStream for Task<F> {
    fn is_terminated(&self) -> bool {
        matches!(self.inner, TaskRepr::Done)
    }
}
