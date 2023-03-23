use std::{
    future::{Future, Ready},
    marker::PhantomData,
};

use serde_json::Value;

use crate::alpha::MiddlewareArgMapper;

use super::{Fut, Ret};

pub trait Executable2 {
    type Fut: Future<Output = Value>;

    fn call(&self, v: Value) -> Self::Fut;
}

impl<TFut: Fut<Value>, TFunc: Fn(Value) -> TFut + 'static> Executable2 for TFunc {
    type Fut = TFut;

    fn call(&self, v: Value) -> Self::Fut {
        (self)(v)
    }
}

pub struct Executable2Placeholder {}

impl Executable2 for Executable2Placeholder {
    type Fut = Ready<Value>;

    fn call(&self, v: Value) -> Self::Fut {
        unreachable!();
    }
}

pub trait MwV2Result {
    type Ctx;
    type MwMapper: MiddlewareArgMapper;
    type Resp: Executable2;

    fn into_executable(self) -> Option<Self::Resp>;
}

pub struct MwResultWithCtx<TLCtx, M, TResp>
where
    M: MiddlewareArgMapper,
    TResp: Executable2,
{
    pub(crate) ctx: TLCtx,
    pub(crate) resp: Option<TResp>,
    pub(crate) phantom: PhantomData<M>,
}

impl<TLCtx, M, TResp> MwResultWithCtx<TLCtx, M, TResp>
where
    M: MiddlewareArgMapper,
    TResp: Executable2,
{
    pub fn resp<E: Executable2>(self, handler: E) -> MwResultWithCtx<TLCtx, M, E> {
        MwResultWithCtx {
            ctx: self.ctx,
            resp: Some(handler),
            phantom: PhantomData,
        }
    }
}

impl<TLCtx, M, TResp> MwV2Result for MwResultWithCtx<TLCtx, M, TResp>
where
    M: MiddlewareArgMapper,
    TResp: Executable2,
{
    type Ctx = TLCtx;
    type MwMapper = M;
    type Resp = TResp;

    fn into_executable(self) -> Option<Self::Resp> {
        self.resp
    }
}

#[cfg(test)]
mod tests {
    use crate::alpha::{middleware::AlphaMiddlewareContext, MwV2};

    use super::*;

    fn mw<
        TMarker,
        Mw: MwV2<(), TMarker>
            + Fn(
                AlphaMiddlewareContext<
                    <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
                >,
                (),
            ) -> Mw::Fut,
    >(
        m: Mw,
    ) {
    }

    #[tokio::test]
    async fn test_mw_results() {
        // Pass through ctx
        mw(|mw, ctx| async move { mw.next(ctx) });

        // Switch ctx
        mw(|mw, ctx| async move { mw.next(()) });

        // Handle response
        mw(|mw, ctx| async move { mw.next(()).resp(|result| async move { result }) });

        // Middleware args
        mw(|mw, ctx| async move {
            let my_mappers_state = mw.state;
            mw.args::<()>().next(())
        });

        // TODO: Handle only query/mutation response
        // mw(|mw, ctx| async move {
        //     mw.args::<()>().next(()).raw_resp(|resp| {
        //         match resp {
        //             ValueOrStream::Value(_) => {},
        //             ValueOrStream::Stream(_) => {},
        //         }
        //     })
        // });

        // TODO: Replace stream
        // mw(|mw, ctx| async move {
        //     mw.args::<()>().next(()).stream(|stream| {
        //         async_stream::stream! {
        //             while let Some(msg) = stream.next().await {
        //                 yield msg;
        //             }
        //         }
        //     })
        // });
    }
}