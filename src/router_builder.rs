use std::{borrow::Cow, panic::Location};

use crate::{
    internal::{
        build::build, middleware::MiddlewareBuilder, procedure::Procedure,
        procedure_store::is_valid_name, resolver::HasResolver,
    },
    layer::Layer,
    router::Router,
    router_builder2::{edit_build_error_name, new_build_error, BuildError, BuildResult},
};

type ProcedureBuildFn<TCtx> = Box<dyn FnOnce(Cow<'static, str>, &mut Router<TCtx>)>;

pub struct RouterBuilder<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    procedures: Vec<(Cow<'static, str>, ProcedureBuildFn<TCtx>)>,
    errors: Vec<BuildError>,
}

impl<TCtx> RouterBuilder<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    /// Constructs a new `Router`.
    /// Avoid using this directly, use [`Rspc::router`] instead so the types can be inferred.
    pub(crate) fn _internal_new() -> Self {
        Self {
            procedures: Vec::new(),
            errors: Vec::new(),
        }
    }

    #[track_caller]
    pub fn procedure<F, TError, TMiddleware, M1, M2>(
        mut self,
        key: &'static str,
        procedure: Procedure<HasResolver<F, TError, M1, M2>, TMiddleware>,
    ) -> Self
    where
        HasResolver<F, TError, M1, M2>: Layer<TMiddleware::LayerCtx>,
        TMiddleware: MiddlewareBuilder<Ctx = TCtx>,
    {
        if let Some(cause) = is_valid_name(key) {
            self.errors.push(new_build_error(
                cause,
                #[cfg(debug_assertions)]
                Cow::Borrowed(key),
                #[cfg(debug_assertions)]
                Location::caller(),
            ));
        }

        let Procedure { resolver, mw } = procedure;

        let kind = resolver.kind;

        self.procedures.push((
            Cow::Borrowed(key),
            Box::new(move |key, ctx| build(key, ctx, kind, mw.build(resolver))),
        ));

        self
    }

    #[track_caller]
    #[allow(unused_mut)]
    pub fn merge(mut self, prefix: &'static str, mut r: RouterBuilder<TCtx>) -> Self {
        if let Some(cause) = is_valid_name(prefix) {
            self.errors.push(new_build_error(
                cause,
                #[cfg(debug_assertions)]
                Cow::Borrowed(prefix),
                #[cfg(debug_assertions)]
                Location::caller(),
            ));
        }

        #[cfg(not(debug_assertions))]
        {
            self.errors.append(&mut r.errors);
        }

        #[cfg(debug_assertions)]
        {
            self.errors.extend(&mut r.errors.into_iter().map(|mut err| {
                edit_build_error_name(&mut err, |name| Cow::Owned(format!("{}.{}", prefix, name)));
                err
            }));
        }

        self.procedures.extend(
            r.procedures
                .into_iter()
                .map(|(name, p)| (Cow::Owned(format!("{}.{}", prefix, name)), p)),
        );

        self
    }

    pub fn build(self) -> BuildResult<TCtx> {
        if !self.errors.is_empty() {
            return BuildResult::Err(self.errors);
        }

        let mut router = Router::default();

        for (key, build_fn) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            (build_fn)(key, &mut router);
        }

        BuildResult::Ok(router)
    }
}
