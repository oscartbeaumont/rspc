use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use futures::Stream;
use serde_json::Value;

use crate::ExecError;

pub trait MiddlewareBuilder<TCtx> {
    type LayerContext: 'static;

    fn build(&self, next: Box<dyn Middleware<Self::LayerContext>>) -> Box<dyn Middleware<TCtx>>;
}

pub struct Demo<TCtx, TLayerCtx>
where
    TLayerCtx: 'static,
{
    pub bruh: Box<dyn Fn(Box<dyn Middleware<TLayerCtx>>) -> Box<dyn Middleware<TCtx>>>,
}

impl<TCtx, TLayerCtx> MiddlewareBuilder<TCtx> for Demo<TCtx, TLayerCtx>
where
    TLayerCtx: 'static,
{
    type LayerContext = TLayerCtx;

    fn build(&self, next: Box<dyn Middleware<TLayerCtx>>) -> Box<dyn Middleware<TCtx>> {
        (self.bruh)(next)
    }
}

pub struct BaseMiddleware<TCtx: 'static>(PhantomData<TCtx>);

impl<TCtx> BaseMiddleware<TCtx> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> MiddlewareBuilder<TCtx> for BaseMiddleware<TCtx> {
    type LayerContext = TCtx;

    fn build(&self, next: Box<dyn Middleware<TCtx>>) -> Box<dyn Middleware<TCtx>> {
        Box::new(move |ctx, args, kak| next.call(ctx, args, kak))
    }
}

pub trait Middleware<TLayerCtx: 'static>: Send + Sync {
    fn call(&self, a: TLayerCtx, b: Value, c: KindAndKey) -> Result<LayerResult, ExecError>;
}

impl<T, TLayerCtx> Middleware<TLayerCtx> for T
where
    T: Fn(TLayerCtx, Value, KindAndKey) -> Result<LayerResult, ExecError> + Send + Sync,
    TLayerCtx: 'static,
{
    fn call(&self, a: TLayerCtx, b: Value, c: KindAndKey) -> Result<LayerResult, ExecError> {
        self(a, b, c)
    }
}

// BREAK

// #[deprecated]
pub struct OperationKey();

// #[deprecated]
pub struct OperationKind();

// #[deprecated]
pub type KindAndKey = (OperationKind, OperationKey);

pub enum LayerResult {
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    Future(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    FutureStreamOrValue(Pin<Box<dyn Future<Output = Result<Value, ExecError>> + Send>>),
    Ready(Result<Value, ExecError>),
}

impl LayerResult {
    // TODO: Probs just use `Into<Value>` trait instead
    pub(crate) async fn into_value(self) -> Result<Value, ExecError> {
        match self {
            LayerResult::Stream(stream) => todo!(), // Ok(StreamOrValue::Stream(stream)),
            LayerResult::Future(fut) => Ok(fut.await?),
            LayerResult::FutureStreamOrValue(fut) => Ok(fut.await?),
            LayerResult::Ready(res) => Ok(res?),
        }
    }
}

pub struct MiddlewareContext<TLayerCtx, TNewLayerCtx>
where
    TNewLayerCtx: Send,
{
    pub key: OperationKey,
    pub kind: OperationKind,
    pub ctx: TLayerCtx,
    pub arg: Value,
    pub(crate) nextmw: Arc<Box<dyn Middleware<TNewLayerCtx>>>,
}

impl<TLayerCtx, TNewLayerCtx> MiddlewareContext<TLayerCtx, TNewLayerCtx>
where
    TLayerCtx: 'static,
    TNewLayerCtx: Send + 'static,
{
    pub async fn next(self, ctx: TNewLayerCtx) -> Result<Value, ExecError> {
        self.nextmw
            .call(ctx, self.arg, (self.kind, self.key))?
            .into_value()
            .await
    }
}