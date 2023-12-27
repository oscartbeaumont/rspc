use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

use crate::error::ProcedureError;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Type)]
pub struct RequestData {
    /// A unique ID used to identify the request
    /// It is the client's responsibility to ensure that this ID is unique.
    /// When using the HTTP Link this will always be `0`.
    pub id: u32,
    pub path: Cow<'static, str>,
    pub input: Option<Value>,
}

/// The type of a request to rspc.
///
/// @internal
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Type)]
#[serde(tag = "method", rename_all = "camelCase")]
pub enum Request {
    Query(RequestData),
    Mutation(RequestData),
    Subscription(RequestData),
    SubscriptionStop { id: u32 },
}

/// A value that can be a successful result or an error.
///
/// @internal
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Type)]
// #[cfg_attr(test, derive(specta::Type))]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum ResponseInner {
    /// The result of a successful operation.
    Value(Value),
    /// The result of a failed operation.
    Error(ProcedureError),
    /// A message to indicate that the operation is complete.
    Complete,
}

/// The type of a response from rspc.
///
/// @internal
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Type)]
// #[cfg_attr(test, derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub id: u32,
    #[serde(flatten)]
    pub inner: ResponseInner,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Type)]
pub enum Requests {
    One(Request),
    Many(Vec<Request>),
}

/// The type of an incoming message to the [`Connection`] abstraction.
///
/// This allows it to be used with any socket that can convert into this type.
#[derive(Debug)]
#[allow(dead_code)]
pub enum IncomingMessage {
    Msg(Result<serde_json::Value, serde_json::Error>),
    Close,
    Skip,
}
