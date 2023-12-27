use std::{
    fmt::Debug,
    future::{Future, Ready},
};

use serde_json::Value;

use super::RequestContext;
use crate::error::{private::IntoResolverError, ExecError};

pub trait Ret: Debug + Send + Sync + 'static {}
impl<T: Debug + Send + Sync + 'static> Ret for T {}

pub trait Fut<TRet: Ret>: Future<Output = TRet> + Send + 'static {}
impl<TRet: Ret, TFut: Future<Output = TRet> + Send + 'static> Fut<TRet> for TFut {}

// TODO: Remove this if possible it's something to do with optional function callback but can we cheat it with two different impls for `.call` based on generic?
pub trait Executable2: Send + Sync + 'static {
    type Fut: Future<Output = Value> + Send;

    fn call(&self, v: Value) -> Self::Fut;
}

impl<TFut: Fut<Value>, TFunc: Fn(Value) -> TFut + Send + Sync + 'static> Executable2 for TFunc {
    type Fut = TFut;

    fn call(&self, v: Value) -> Self::Fut {
        (self)(v)
    }
}

pub struct Executable2Placeholder {}

impl Executable2 for Executable2Placeholder {
    type Fut = Ready<Value>;

    fn call(&self, _: Value) -> Self::Fut {
        unreachable!();
    }
}

// #[deprecated = "TODO: We probs have to remove this. Sadge!"] // TODO: Deal with this type and seal it
pub trait MwV2Result {
    type Ctx: Send + Sync + 'static;
    type Resp: Executable2;

    // TODO: Seal this and make it private
    fn explode(self) -> Result<(Self::Ctx, Value, RequestContext, Option<Self::Resp>), ExecError>;
}

// TODO: Seal this and rename it
pub struct MwResultWithCtx<TLCtx, TResp> {
    pub(crate) input: Value,
    pub(crate) req: RequestContext,
    pub(crate) ctx: TLCtx,
    pub(crate) resp: Option<TResp>,
}

impl<TLCtx, TResp: Executable2> MwResultWithCtx<TLCtx, TResp> {
    pub fn map<E: Executable2>(self, handler: E) -> MwResultWithCtx<TLCtx, E> {
        MwResultWithCtx {
            input: self.input,
            req: self.req,
            ctx: self.ctx,
            resp: Some(handler),
        }
    }
}

impl<TLCtx, TResp> MwV2Result for MwResultWithCtx<TLCtx, TResp>
where
    TLCtx: Send + Sync + 'static,
    TResp: Executable2,
{
    type Ctx = TLCtx;
    type Resp = TResp;

    fn explode(self) -> Result<(Self::Ctx, Value, RequestContext, Option<Self::Resp>), ExecError> {
        Ok((self.ctx, self.input, self.req, self.resp))
    }
}

impl<TLCtx, TResp, TError> MwV2Result for Result<MwResultWithCtx<TLCtx, TResp>, TError>
where
    TLCtx: Send + Sync + 'static,
    TResp: Executable2,
    TError: IntoResolverError,
{
    type Ctx = TLCtx;
    type Resp = TResp;

    fn explode(self) -> Result<(Self::Ctx, Value, RequestContext, Option<Self::Resp>), ExecError> {
        self.map_err(|e| e.into_resolver_error().into())
            .and_then(|r| r.explode())
    }
}
