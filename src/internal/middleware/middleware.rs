use std::{future::Future, marker::PhantomData};

use crate::middleware_from_core::{new_mw_ctx, MiddlewareContext, MwV2Result};

use super::{arg_mapper::ArgumentMapper, ArgumentMapperPassthrough, Middleware};

// TODO: These types need to move out of the `internal` module

// TODO: Rename `Middleware` unpon finsihing `new-mw-system`
pub struct Middleware2<TLCtx, A, M, Fu> {
    m: M,
    exec: fn(ctx: TLCtx, mw: MiddlewareContext, m: &M) -> Fu,
    phantom: PhantomData<(TLCtx, A)>,
}

impl<TLCtx, M, A, Fu> Middleware<TLCtx> for Middleware2<TLCtx, A, M, Fu>
where
    TLCtx: Send + Sync + 'static,
    M: Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result,
    A: ArgumentMapper,
{
    type Fut = Fu;
    type Result = Fu::Output;
    type NewCtx = <Fu::Output as MwV2Result>::Ctx;
    type Mapper = A;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
        (self.exec)(ctx, mw, &self.m)
    }
}

pub fn mw<TLCtx, M>(m: M) -> Middleware2<TLCtx, ArgumentMapperPassthrough, M, M::Fut>
where
    TLCtx: Send + Sync + 'static,
    M: Middleware<TLCtx> + Fn(MiddlewareContext, TLCtx) -> M::Fut,
{
    Middleware2 {
        m,
        exec: |ctx, mw, m| m.run_me(ctx, mw),
        phantom: PhantomData,
    }
}

pub struct ArgMapper<A: ArgumentMapper>(PhantomData<A>);

impl<A: ArgumentMapper> ArgMapper<A> {
    pub fn new<TLCtx, M, Fu>(
        m: M,
    ) -> impl Middleware<TLCtx, NewCtx = <Fu::Output as MwV2Result>::Ctx>
    where
        TLCtx: Send + Sync + 'static,
        M: Fn(MiddlewareContext, TLCtx, A::State) -> Fu + Send + Sync + 'static,
        Fu: Future + Send + 'static,
        Fu::Output: MwV2Result,
        A: ArgumentMapper,
    {
        Middleware2::<TLCtx, A, M, Fu> {
            m,
            exec: |ctx, mw, m| {
                let (out, state) =
                    // TODO:  Is the hardcoded generic a problem? Like what if using a trait to determine the correct type cause this won't match the procedure's arg type???
                    A::map::<serde_json::Value>(serde_json::from_value(mw.input).unwrap());
                (m)(new_mw_ctx(out, mw.req), ctx, state)
            },
            phantom: PhantomData,
        }
    }
}
