use std::{
    pin::Pin,
    task::{Context, Poll},
};

use serde_json::Value;

use crate::error::ExecError;

/// The resulting body from an rspc operation.
///
/// This can mean different things in different contexts.
/// For a query or mutation each frame is a part of the resulting single "message". Eg. part of the json, or part of a file.
/// For a subscription each frame is a discrete websocket message. Eg. the json for a single procedure's result
///
#[must_use = "`Body` do nothing unless polled"]
#[deprecated]
pub trait Body {
    // TODO: Return `bytes::Bytes` instead
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Value, ExecError>>>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
