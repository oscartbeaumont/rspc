use std::{borrow::Cow, future::Future, marker::PhantomData};

use crate::Router;

/// TODO
pub struct Procedure<TCtx = ()> {
    #[cfg(debug_assertions)]
    pub(crate) location: std::panic::Location<'static>,
    pub(crate) build: Box<dyn FnOnce(Cow<'static, str>, &mut Router<TCtx>)>,
}

impl<TCtx> Procedure<TCtx> {
    pub fn new<TError>() -> ProcedureBuilder<Generics<TCtx, TError, ()>> {
        ProcedureBuilder(Generics(PhantomData))
    }
}

mod private {
    use super::*;
    pub struct Generics<TCtx, TError, TStream>(pub(super) PhantomData<(TCtx, TError, TStream)>);
}
use private::Generics;

/// TODO
pub struct ProcedureBuilder<G>(G);

impl<TCtx, TError, TStream> ProcedureBuilder<Generics<TCtx, TError, TStream>> {
    /// Manually override the error type of this procedure.
    /// By default the error type will be infered from the [Router](rspc::Router).
    pub fn error<T>(self) -> ProcedureBuilder<Generics<TCtx, T, TStream>> {
        ProcedureBuilder(Generics(PhantomData))
    }

    // TODO: Middleware working
    pub fn with(self) -> Self {
        todo!();
    }

    #[track_caller]
    pub fn query<F, I>(self, handler: F) -> Procedure<TCtx>
    where
        F: ProcedureFunc<TCtx, TError, I>,
    {
        Procedure {
            #[cfg(debug_assertions)]
            location: *std::panic::Location::caller(),
            build: Box::new(move |key, router| {
                router.0.insert(
                    key,
                    // TODO: Async procedures
                    Box::new(move |serializer, ctx, input| {
                        let input: i32 = erased_serde::deserialize(input).unwrap();

                        serializer.serialize(&format!("Hello {:?}", input));
                    }),
                );
            }),
        }
    }

    #[track_caller]
    pub fn mutation<F, I>(self, handler: F) -> Procedure<TCtx>
    where
        F: ProcedureFunc<TCtx, TError, I>,
    {
        todo!();
    }
}

// TODO: Prevent downstream impls by inherinting from sealed trait.
/// TODO
// `TInput` mostly exists as a generic to contrain the impl.
pub trait ProcedureFunc<TCtx, TError, TInput> {
    type Result;
    type Future: Future<Output = Result<Self::Result, TError>>;

    fn exec(&self, ctx: TCtx, input: TInput) -> Self::Future;
}
impl<
        TCtx,
        TInput,
        TResult,
        TError,
        F: Fn(TCtx, TInput) -> Fu,
        Fu: Future<Output = Result<TResult, TError>>,
    > ProcedureFunc<TCtx, TError, TInput> for F
{
    type Result = TResult;
    type Future = Fu;

    fn exec(&self, ctx: TCtx, input: TInput) -> Self::Future {
        (self)(ctx, input)
    }
}
