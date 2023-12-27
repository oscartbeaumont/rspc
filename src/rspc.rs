use std::marker::PhantomData;

use crate::{
    error::{private::IntoResolverError, Infallible},
    internal::{
        middleware::{BaseMiddleware, Middleware, MiddlewareLayerBuilder},
        procedure::{resolvers, MissingResolver, Procedure},
        resolver::{HasResolver, QueryOrMutation, Subscription},
    },
    layer::Layer,
    middleware_from_core::ProcedureKind,
    RouterBuilder,
};

/// Rspc is a starting point for constructing rspc procedures or routers.
///
/// This method supports const contexts so it can be instantiated at the top level as reuse across the whole application.
///
/// ```rust
/// use rspc::Rspc;
///
/// const R: Rspc<()> = Rspc::new();
/// ```
pub struct Rspc<
    TCtx = (), // The is the context the current router was initialised with,
    TError = Infallible,
> where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    phantom: PhantomData<(TCtx, TError)>,
}

#[allow(clippy::new_without_default)]
impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    pub const fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    pub fn router(&self) -> RouterBuilder<TCtx> {
        RouterBuilder::_internal_new()
    }

    pub fn error<TNewError>(self) -> Procedure<MissingResolver<TNewError>, BaseMiddleware<TCtx>> {
        Procedure::new(MissingResolver::default(), BaseMiddleware::default())
    }

    pub fn with<Mw: Middleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError>, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                middleware: BaseMiddleware::default(),
                mw,
            },
        )
    }

    resolvers!(_, TCtx, BaseMiddleware<TCtx>, BaseMiddleware::default());
}
