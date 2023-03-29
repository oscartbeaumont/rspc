use std::{
    any::Any,
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project::pin_project;
use serde_json::Value;

use crate::{ExecError, MiddlewareContext, MiddlewareFutOrSomething, MiddlewareLike, PinnedOption};

pub trait MiddlewareBuilderLike<TCtx: 'static> {
    type LayerContext: 'static;
    type LayerResult<T>: Layer<TCtx>
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>;
}

pub struct MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TCtx: 'static,
    TLayerCtx: 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub middleware2: TIncomingMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware> MiddlewareBuilderLike<TCtx>
    for MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>
where
    TCtx: 'static,
    TLayerCtx: 'static,
    TNewLayerCtx: 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx>,
    TIncomingMiddleware: MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx>,
{
    type LayerContext = TNewLayerCtx;
    type LayerResult<T> = TMiddleware::LayerResult<TIncomingMiddleware::LayerResult<T>>
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>,
    {
        self.middleware.build(self.middleware2.build(next))
    }
}

pub struct MiddlewareLayerWithNext();

pub struct MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx>,
{
    pub middleware: TMiddleware,
    pub mw: TNewMiddleware,
    pub phantom: PhantomData<(TCtx, TLayerCtx, TNewLayerCtx)>,
}

impl<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware> MiddlewareBuilderLike<TCtx>
    for MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TCtx: Send + Sync + 'static,
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    type LayerContext = TNewLayerCtx;
    type LayerResult<T> = TMiddleware::LayerResult<MiddlewareLayer<TLayerCtx, TNewLayerCtx, T, TNewMiddleware>> // TODO
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext> + Sync,
    {
        let func = self.mw.explode();

        self.middleware.build(MiddlewareLayer {
            next: Arc::new(next), // Avoiding `Arc`
            mw: func, // self.mw.clone(),  // TODO: Avoid `Clone` bound when `build` takes `self`
            phantom: PhantomData,
        })
    }
}

pub struct MiddlewareLayer<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + 'static,
    TNewLayerCtx: Send + 'static,
    TMiddleware: Layer<TNewLayerCtx> + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    next: Arc<TMiddleware>, // TODO: Avoid arcing this if possible
    // mw: TNewMiddleware,
    mw: TNewMiddleware::Fn,
    phantom: PhantomData<(TLayerCtx, TNewLayerCtx)>,
}

impl<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware> Layer<TLayerCtx>
    for MiddlewareLayer<TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TNewLayerCtx: Send + Sync + 'static,
    TMiddleware: Layer<TNewLayerCtx> + Sync + 'static,
    TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
{
    type Fut<'a> = MiddlewareFutOrSomething<
        'a,
        TNewMiddleware::State,
        TLayerCtx,
        TNewLayerCtx,
        TNewMiddleware::Fut2,
        TMiddleware,
    >; // TNewMiddleware::Fut<TMiddleware>;
    type Middleware = TMiddleware; // TODO
    type NewCtx = TNewLayerCtx;

    // type Call2Fut = Ready<Result<ValueOrStreamOrFut2<'a, , ExecError>>; // TODO: NoShot<TNewLayerCtx, TMiddleware>;

    fn call<'a>(&'a self, ctx: TLayerCtx, input: Value, req: RequestContext) -> Self::Fut<'a> {
        // TODO: Don't take ownership of `self.next` to avoid needing it to be `Arc`ed

        // self.mw.handle(ctx, input, req, &self.next)

        let handler = (self.mw)(MiddlewareContext {
            state: (),
            ctx,
            input,
            req,
            phantom: PhantomData,
        });

        // TODO: Avoid taking ownership of `next`
        MiddlewareFutOrSomething(PinnedOption::Some(handler), &self.next, PinnedOption::None)
    }

    // type ResolveFutFut = Ready<Result<ValueOrStream, ExecError>>;

    // fn resolve_fut<'a>(
    //     &self,
    //     input: ValueOrStreamOrFut2<'a, Self::Middleware, Self::NewCtx>,
    // ) -> Self::ResolveFutFut {
    //     // match input {
    //     //     ValueOrStreamOrFut2::Value(v) => Ok(ValueOrStream::Value(v)),
    //     //     ValueOrStreamOrFut2::Stream(s) => Ok(ValueOrStream::Stream(s)),
    //     //     ValueOrStreamOrFut2::A(mw, ctx, input, req) => {
    //     //         let fut = mw.call(ctx, input, req);

    //     //         // let fut = fut.map(|v| ValueOrStream::Value(v));
    //     //         // Ok(ValueOrStream::Fut(fut))
    //     //         todo!();
    //     //     }
    //     // };

    //     todo!();
    // }

    // TODO: Removing this
    // fn call2(
    //     &self,
    //     ctx: Box<dyn Any + Send + 'static>,
    //     value: Value,
    //     req: RequestContext,
    // ) -> Self::Call2Fut {
    //     // let fut = self.next.call(*ctx.downcast().unwrap(), value, req);
    //     // NoShot(PinnedOption::Some(fut))
    //     todo!();
    // }
}

// pub enum

pub struct BaseMiddleware<TCtx>(PhantomData<TCtx>)
where
    TCtx: 'static;

impl<TCtx> Default for BaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<TCtx> BaseMiddleware<TCtx>
where
    TCtx: 'static,
{
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> MiddlewareBuilderLike<TCtx> for BaseMiddleware<TCtx>
where
    TCtx: Send + 'static,
{
    type LayerContext = TCtx;
    type LayerResult<T> = T
    where
        T: Layer<Self::LayerContext>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerContext>,
    {
        next
    }
}

// TODO: Rename this so it doesn't conflict with the middleware builder struct
// TODO: Document the types and functions so they make sense
pub trait Layer<TLayerCtx: 'static>: DynLayer<TLayerCtx> + Send + Sync + 'static {
    type Fut<'a>: Future<Output = Result<ValueOrStreamOrFut2<'a, Self::Middleware, Self::NewCtx>, ExecError>>
        + Send
        + 'a;
    type Middleware: Layer<Self::NewCtx> + 'static;
    type NewCtx: Send + 'static;

    // TODO: Remove it unused
    // type Call2Fut: Future<Output = Result<ValueOrStreamOrFut2, ExecError>> + Send + 'static;

    fn call<'a>(&'a self, a: TLayerCtx, b: Value, c: RequestContext) -> Self::Fut<'a>;

    // type ResolveFutFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static;

    // // This works around the recursive async function problem.
    // fn resolve_fut<'a>(
    //     &self,
    //     input: ValueOrStreamOrFut2<'a, Self::Middleware, Self::NewCtx>,
    // ) -> Self::ResolveFutFut;

    fn erase(self) -> Box<dyn DynLayer<TLayerCtx>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// TODO: Remove this
impl Layer<()> for () {
    type Fut<'a> = Ready<Result<ValueOrStreamOrFut2<'a, (), ()>, ExecError>>;
    type Middleware = ();
    type NewCtx = ();

    fn call<'a>(&'a self, a: (), b: Value, c: RequestContext) -> Self::Fut<'a> {
        unreachable!()
    }

    // type ResolveFutFut = Ready<Result<ValueOrStream, ExecError>>; // Unused

    // fn resolve_fut<'a>(
    //     &self,
    //     input: ValueOrStreamOrFut2<'a, Self::Middleware, Self::NewCtx>,
    // ) -> Self::ResolveFutFut {
    //     unreachable!();
    // }
}

// TODO: Does this need lifetime?
pub type FutureValueOrStream<'a> =
    Pin<Box<dyn Future<Output = Result<ValueOrStream, ExecError>> + Send + 'a>>;

pub trait DynLayer<TLayerCtx: 'static>: Send + Sync + 'static {
    fn dyn_call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream<'a>, ExecError>;
}

impl<TLayerCtx: Send + 'static, L: Layer<TLayerCtx>> DynLayer<TLayerCtx> for L {
    fn dyn_call<'a>(
        &'a self,
        a: TLayerCtx,
        b: Value,
        c: RequestContext,
    ) -> Result<FutureValueOrStream<'a>, ExecError> {
        Ok(Box::pin(async move {
            // recursive_resolve(Layer::call(self, a, b, c).await?).await;

            // let y = self.resolve_fut(Layer::call(self, a, b, c).await?);

            match Layer::call(self, a, b, c).await? {
                ValueOrStreamOrFut2::Value(x) => Ok(ValueOrStream::Value(x)),
                ValueOrStreamOrFut2::A(mw, ctx, input, req) => {
                    todo!();
                }
                ValueOrStreamOrFut2::Stream(x) => Ok(ValueOrStream::Stream(x)),
            }
        }))
    }
}

// async fn recursive_resolve<'a, TMiddleware, TNewCtx>(
//     result: ValueOrStreamOrFut2<'a, TMiddleware, TNewCtx>,
// ) -> Result<ValueOrStream, ExecError>
// where
//     TMiddleware: Layer<TNewCtx> + 'static,
//     TNewCtx: 'static,
// {
//     match result {
//         ValueOrStreamOrFut2::Value(x) => Ok(ValueOrStream::Value(x)),
//         ValueOrStreamOrFut2::A(mw, ctx, input, req) => {
//             // recursive_resolve(mw.call(ctx, input, req).await?).await

//             match mw.call(ctx, input, req).await? {
//                 ValueOrStreamOrFut2::Value(x) => Ok(ValueOrStream::Value(x)),
//                 ValueOrStreamOrFut2::A(mw, ctx, input, req) => {
//                     // recursive_resolve(mw.call(ctx, input, req).await?).await
//                     todo!();
//                 }
//                 ValueOrStreamOrFut2::Stream(x) => Ok(ValueOrStream::Stream(x)),
//             }
//         }
//         ValueOrStreamOrFut2::Stream(x) => Ok(ValueOrStream::Stream(x)),
//     }
// }

pub struct ResolverLayer<TLayerCtx, T, TFut>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static,
{
    pub func: T,
    pub phantom: PhantomData<TLayerCtx>,
}

impl<T, TLayerCtx, TFut> Layer<TLayerCtx> for ResolverLayer<TLayerCtx, T, TFut>
where
    TLayerCtx: Send + Sync + 'static,
    T: Fn(TLayerCtx, Value, RequestContext) -> TFut + Send + Sync + 'static,
    TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static,
{
    type Fut<'a> = ResolverLayerFut<'a, TFut>;
    type Middleware = (); // TODO
    type NewCtx = ();

    fn call<'a>(&'a self, a: TLayerCtx, b: Value, c: RequestContext) -> Self::Fut<'a> {
        ResolverLayerFut((self.func)(a, b, c), PhantomData)
    }

    // type ResolveFutFut = Ready<Result<ValueOrStream, ExecError>>; // Unused

    // fn resolve_fut<'a>(
    //     &self,
    //     input: ValueOrStreamOrFut2<'a, Self::Middleware, Self::NewCtx>,
    // ) -> Self::ResolveFutFut {
    //     todo!();
    // }
}

#[pin_project(project = ResolverLayerFutProj)]
pub struct ResolverLayerFut<
    'a,
    TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static,
>(#[pin] TFut, PhantomData<&'a ()>);

impl<'a, TFut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static> Future
    for ResolverLayerFut<'a, TFut>
{
    type Output = Result<ValueOrStreamOrFut2<'a, (), ()>, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            // TODO: Simplify this a bit
            Poll::Ready(Ok(ValueOrStream::Value(v))) => {
                Poll::Ready(Ok(ValueOrStreamOrFut2::Value(v)))
            }
            Poll::Ready(Ok(ValueOrStream::Stream(s))) => {
                Poll::Ready(Ok(ValueOrStreamOrFut2::Stream(s)))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

// TODO: Is this a duplicate of any type?
// TODO: Move into public API cause it might be used in middleware
#[derive(Debug, Clone)]
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

// TODO: Maybe rename to `Request` or something else. Also move into Public API cause it might be used in middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub kind: ProcedureKind,
    pub path: String, // TODO: String slice??
}

// TODO: Avoid using `Ready<T>` for top layer and instead store as `Value` so the procedure can be quick as fuck???

// TODO: Replace this with `ValueOrStreamOrFut2`
pub enum ValueOrStream {
    Value(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
    // TODO: Rename this
    // TheSolution(Box<dyn Any + Send + 'static>, Value, RequestContext),
}

pub enum ValueOrStreamOrFut2<'a, TMiddleware, TNewCtx>
where
    TMiddleware: Layer<TNewCtx> + 'static,
    TNewCtx: 'static,
{
    Value(Value),
    A(&'a TMiddleware, TNewCtx, Value, RequestContext),
    // TODO: Take this type in as a generic
    Stream(Pin<Box<dyn Stream<Item = Result<Value, ExecError>> + Send>>),
}
