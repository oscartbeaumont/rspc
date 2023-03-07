use std::{error, fmt, sync::Arc};

use serde::Serialize;
use specta::{ts::TsExportError, Type};

use crate::internal::jsonrpc::JsonRPCError;

#[derive(thiserror::Error, Debug)]
pub enum ExecError {
    #[error("the requested operation '{0}' is not supported by this server")]
    OperationNotFound(String),
    #[error("error deserializing procedure arguments: {0}")]
    DeserializingArgErr(serde_json::Error),
    #[error("error serializing procedure result: {0}")]
    SerializingResultErr(serde_json::Error),
    #[error("error in axum extractor")]
    AxumExtractorError,
    #[error("invalid JSON-RPC version")]
    InvalidJsonRpcVersion,
    #[error("method '{0}' is not supported by this endpoint.")] // TODO: Better error message
    UnsupportedMethod(String),
    #[error("resolver threw error")]
    ErrResolverError(#[from] Error),
    #[error("error creating subscription with null id")]
    ErrSubscriptionWithNullId,
    #[error("error creating subscription with duplicate id")]
    ErrSubscriptionDuplicateId,
}

impl From<ExecError> for Error {
    fn from(v: ExecError) -> Error {
        match v {
            ExecError::OperationNotFound(_) => Error {
                code: ErrorCode::NotFound,
                message: "the requested operation is not supported by this server".to_string(),
                cause: None,
            },
            ExecError::DeserializingArgErr(err) => Error {
                code: ErrorCode::BadRequest,
                message: "error deserializing procedure arguments".to_string(),
                cause: Some(Arc::new(err)),
            },
            ExecError::SerializingResultErr(err) => Error {
                code: ErrorCode::InternalServerError,
                message: "error serializing procedure result".to_string(),
                cause: Some(Arc::new(err)),
            },
            ExecError::AxumExtractorError => Error {
                code: ErrorCode::BadRequest,
                message: "Error running Axum extractors on the HTTP request".into(),
                cause: None,
            },
            ExecError::InvalidJsonRpcVersion => Error {
                code: ErrorCode::BadRequest,
                message: "invalid JSON-RPC version".into(),
                cause: None,
            },
            ExecError::ErrResolverError(err) => err,
            ExecError::UnsupportedMethod(_) => Error {
                code: ErrorCode::BadRequest,
                message: "unsupported metho".into(),
                cause: None,
            },
            ExecError::ErrSubscriptionWithNullId => Error {
                code: ErrorCode::BadRequest,
                message: "error creating subscription with null request id".into(),
                cause: None,
            },
            ExecError::ErrSubscriptionDuplicateId => Error {
                code: ErrorCode::BadRequest,
                message: "error creating subscription with duplicate id".into(),
                cause: None,
            },
        }
    }
}

impl From<ExecError> for JsonRPCError {
    fn from(err: ExecError) -> Self {
        let x: Error = err.into();
        x.into()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExportError {
    #[error("IO error exporting bindings: {0}")]
    IOErr(#[from] std::io::Error),
    #[error("error exporting ts bindings: {0}")]
    Error(#[from] TsExportError),
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
#[allow(dead_code)]
pub struct Error {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    #[serde(skip)]
    #[cfg_attr(test, specta(type = Option<serde_json::Value>))]
    pub(crate) cause: Option<Arc<dyn std::error::Error + Send + Sync>>, // We are using `Arc` instead of `Box` so we can clone the error cause `Clone` isn't dyn safe.
}

#[cfg(feature = "workers")]
impl From<worker::Error> for Error {
    fn from(err: worker::Error) -> Self {
        Error {
            code: ErrorCode::InternalServerError,
            message: err.to_string(),
            cause: None, // We can't store the original error because it's not `Send + Sync` as it holds raw pointers. We could probs just lie to the compiler about `Send + Sync` but I can't ensure that's safe so it shouldn't be implicit.
        }
    }
}

impl From<Error> for JsonRPCError {
    fn from(err: Error) -> Self {
        JsonRPCError {
            code: err.code.to_status_code() as i32,
            message: err.message,
            data: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rspc::Error {{ code: {:?}, message: {} }}",
            self.code, self.message
        )
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl Error {
    pub const fn new(code: ErrorCode, message: String) -> Self {
        Error {
            code,
            message,
            cause: None,
        }
    }

    pub fn with_cause<TErr>(code: ErrorCode, message: String, cause: TErr) -> Self
    where
        TErr: std::error::Error + Send + Sync + 'static,
    {
        Self {
            code,
            message,
            cause: Some(Arc::new(cause)),
        }
    }
}

/// In According to error codes fot http (4xx, 5xx). There are codes specific to this crate.
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    #[deprecated(note = "MethodNotSupported is not in accordance to RFC 9110  MethodNotAllowed Should be used instead")]
    MethodNotSupported,
    NotAcceptable,
    ProxyAuthenticationRequired,
    Timeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    ImATeapot,
    MisdirectedRequest,
    UnprocessableEntity,
    Locked,
    FailedDependency,
    TooEarly,
    UpgradeRequired,
    PreconditionRequired,
    TooManyRequests,
    RequestHeaderFieldsTooLarge,
    UnavailableForLegalReasons,
    ClientClosedRequest, ///< Not part of http error codes

    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
    VariantAlsoNegotiates,
    InsufficientStorage,
    LoopDetected,
    NotExtended,
    NetworkAuthenticationRequired,
}

impl ErrorCode {
    pub fn to_status_code(&self) -> u16 {
        match self {
            ErrorCode::BadRequest => 400,
            ErrorCode::Unauthorized => 401,
            ErrorCode::PaymentRequired => 402,
            ErrorCode::Forbidden => 403,
            ErrorCode::NotFound => 404,
            ErrorCode::MethodNotSupported => 405,
            ErrorCode::MethodNotAllowed => 405,
            ErrorCode::NotAcceptable => 406,
            ErrorCode::ProxyAuthenticationRequired => 407,
            ErrorCode::Timeout => 408,
            ErrorCode::Conflict => 409,
            ErrorCode::Gone => 410,
            ErrorCode::LengthRequired => 411,
            ErrorCode::PreconditionFailed => 412,
            ErrorCode::PayloadTooLarge => 413,
            ErrorCode::URITooLong => 414,
            ErrorCode::UnsupportedMediaType => 415,
            ErrorCode::RangeNotSatisfiable => 416,
            ErrorCode::ExpectationFailed => 417,
            ErrorCode::ImATeapot => 418,
            ErrorCode::MisdirectedRequest => 421,
            ErrorCode::UnprocessableEntity => 422,
            ErrorCode::Locked => 423,
            ErrorCode::FailedDependency => 424,
            ErrorCode::TooEarly => 425,
            ErrorCode::UpgradeRequired => 426,
            ErrorCode::PreconditionRequired => 428,
            ErrorCode::TooManyRequests => 429,
            ErrorCode::RequestHeaderFieldsTooLarge => 431,
            ErrorCode::UnavailableForLegalReasons => 451,
            ErrorCode::ClientClosedRequest => 499,

            ErrorCode::InternalServerError => 500,
            ErrorCode::NotImplemented => 501,
            ErrorCode::BadGateway => 502,
            ErrorCode::ServiceUnavailable => 503,
            ErrorCode::GatewayTimeout => 504,
            ErrorCode::HTTPVersionNotSupported => 505,
            ErrorCode::VariantAlsoNegotiates => 506,
            ErrorCode::InsufficientStorage => 507,
            ErrorCode::LoopDetected => 508,
            ErrorCode::NotExtended => 510,
            ErrorCode::NetworkAuthenticationRequired => 511,
        }
    }

    pub const fn from_status_code(status_code: u16) -> Option<Self> {
        match status_code {
            400 => Some(ErrorCode::BadRequest),
            401 => Some(ErrorCode::Unauthorized),
            402 => Some(ErrorCode::PaymentRequired),
            403 => Some(ErrorCode::Forbidden),
            404 => Some(ErrorCode::NotFound),
            405 => Some(ErrorCode::MethodNotAllowed),
            406 => Some(ErrorCode::NotAcceptable),
            407 => Some(ErrorCode::ProxyAuthenticationRequired),
            408 => Some(ErrorCode::Timeout),
            409 => Some(ErrorCode::Conflict),
            410 => Some(ErrorCode::Gone),
            411 => Some(ErrorCode::LengthRequired),
            412 => Some(ErrorCode::PreconditionFailed),
            413 => Some(ErrorCode::PayloadTooLarge),
            414 => Some(ErrorCode::URITooLong),
            415 => Some(ErrorCode::UnsupportedMediaType),
            416 => Some(ErrorCode::RangeNotSatisfiable),
            417 => Some(ErrorCode::ExpectationFailed),
            418 => Some(ErrorCode::ImATeapot),
            421 => Some(ErrorCode::MisdirectedRequest),
            422 => Some(ErrorCode::UnprocessableEntity),
            423 => Some(ErrorCode::Locked),
            424 => Some(ErrorCode::FailedDependency),
            425 => Some(ErrorCode::TooEarly),
            426 => Some(ErrorCode::UpgradeRequired),
            428 => Some(ErrorCode::PreconditionRequired),
            429 => Some(ErrorCode::TooManyRequests),
            431 => Some(ErrorCode::RequestHeaderFieldsTooLarge),
            451 => Some(ErrorCode::UnavailableForLegalReasons),
            499 => Some(ErrorCode::ClientClosedRequest),

            500 => Some(ErrorCode::InternalServerError),
            501 => Some(ErrorCode::NotImplemented),
            502 => Some(ErrorCode::BadGateway),
            503 => Some(ErrorCode::ServiceUnavailable),
            504 => Some(ErrorCode::GatewayTimeout),
            505 => Some(ErrorCode::HTTPVersionNotSupported),
            506 => Some(ErrorCode::VariantAlsoNegotiates),
            507 => Some(ErrorCode::InsufficientStorage),
            508 => Some(ErrorCode::LoopDetected),
            510 => Some(ErrorCode::NotExtended),
            511 => Some(ErrorCode::NetworkAuthenticationRequired),
            _ => None,
        }
    }
}
