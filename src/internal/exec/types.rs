mod private {
    use std::borrow::Cow;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// The type of a request to rspc.
    ///
    /// @internal
    #[derive(Debug, Deserialize)]
    #[cfg_attr(test, derive(specta::Type))]
    #[serde(tag = "method", rename_all = "camelCase")]
    pub enum Request {
        Query {
            /// A unique ID used to identify the request
            /// It is the client's responsibility to ensure that this ID is unique.
            /// When using the HTTP Link this will always be `0`.
            id: u32,
            path: Cow<'static, str>,
            input: Option<Value>,
        },
        Mutation {
            /// A unique ID used to identify the request
            /// It is the client's responsibility to ensure that this ID is unique.
            /// When using the HTTP Link this will always be `0`.
            id: u32,
            path: Cow<'static, str>,
            input: Option<Value>,
        },
        Subscription {
            /// A unique ID used to identify the request
            /// It is the client's responsibility to ensure that this ID is unique.
            id: u32,
            path: Cow<'static, str>,
            input: Option<Value>,
        },
        SubscriptionStop {
            id: u32,
        },
    }

    /// An error that can be returned by rspc.
    ///
    /// @internal
    #[derive(Debug, Serialize, PartialEq, Eq)]
    #[cfg_attr(test, derive(specta::Type))]
    pub struct ResponseError {
        pub code: i32,
        pub message: String,
        pub data: Option<Value>,
    }

    /// A value that can be a successful result or an error.
    ///
    /// @internal
    #[derive(Debug, Serialize, PartialEq, Eq)]
    #[cfg_attr(test, derive(specta::Type))]
    #[serde(tag = "type", content = "value", rename_all = "camelCase")]
    pub enum ResponseInner {
        /// The result of a successful operation.
        Value(Value),
        /// The result of a failed operation.
        Error(ResponseError),
        /// A message to indicate that the operation is complete.
        Complete,
    }

    /// The type of a response from rspc.
    ///
    /// @internal
    #[derive(Debug, Serialize, PartialEq, Eq)]
    #[cfg_attr(test, derive(specta::Type))]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub id: u32,
        #[serde(flatten)]
        pub inner: ResponseInner,
    }

    /// TODO
    #[derive(Debug)]
    #[allow(dead_code)]
    pub(crate) enum IncomingMessage {
        Msg(Result<Value, serde_json::Error>),
        Close,
        Skip,
    }
}

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use private::*;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::*;
