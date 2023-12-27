use std::borrow::Cow;

use serde_json::Value;

use super::{Executable2Placeholder, MwResultWithCtx};

pub fn new_mw_ctx(input: serde_json::Value, req: RequestContext) -> MiddlewareContext {
    MiddlewareContext { input, req }
}

#[non_exhaustive]
pub struct MiddlewareContext {
    pub input: Value,
    pub req: RequestContext,
}

impl MiddlewareContext {
    #[cfg(feature = "tracing")]
    pub fn with_span(mut self, span: Option<tracing::Span>) -> Self {
        self.req.span = Some(span);
        self
    }

    pub fn next<TNCtx>(self, ctx: TNCtx) -> MwResultWithCtx<TNCtx, Executable2Placeholder> {
        MwResultWithCtx {
            input: self.input,
            req: self.req,
            resp: None,
            ctx,
        }
    }
}

/// TODO
// TODO: Is this a duplicate of any type?
// TODO: Move into public API cause it might be used in middleware
#[derive(Debug, Clone, Copy)]
pub enum ProcedureKind {
    Query,
    Mutation,
    Subscription,
}

impl ProcedureKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            ProcedureKind::Query => "query",
            ProcedureKind::Mutation => "mutation",
            ProcedureKind::Subscription => "subscription",
        }
    }
}

/// TODO
// TODO: Maybe rename to `Request` or something else. Also move into Public API cause it might be used in middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub id: u32,
    pub kind: ProcedureKind,
    pub path: Cow<'static, str>,
    #[cfg(feature = "tracing")]
    span: Option<Option<tracing::Span>>,
    // Prevents downstream user constructing type
    _priv: (),
}

impl RequestContext {
    pub(crate) fn new(id: u32, kind: ProcedureKind, path: Cow<'static, str>) -> Self {
        Self {
            id,
            #[cfg(feature = "tracing")]
            span: None,
            kind,
            path,
            _priv: (),
        }
    }

    #[cfg(feature = "tracing")]
    pub fn span(&self) -> Option<tracing::Span> {
        self.span.clone().unwrap_or_else(|| {
            Some(match self.kind {
                ProcedureKind::Query => {
                    let query = self.path.as_ref();
                    tracing::info_span!("rspc", query)
                }
                ProcedureKind::Mutation => {
                    let mutation = self.path.as_ref();
                    tracing::info_span!("rspc", mutation)
                }
                ProcedureKind::Subscription => {
                    let subscription = self.path.as_ref();
                    tracing::info_span!("rspc", subscription)
                }
            })
        })
    }
}
