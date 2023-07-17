use std::{borrow::Cow, panic::Location};

use specta::TypeDefs;

use crate::{
    internal::{
        is_valid_name,
        middleware::{MiddlewareBuilder, SealedMiddlewareBuilder},
        procedure::{BuildProceduresCtx, DynProcedure, Procedure},
        ProcedureStore,
    },
    BuildError, BuildResult, CompiledRouter,
};

pub struct Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    procedures: Vec<(Cow<'static, str>, Box<dyn DynProcedure<TCtx>>)>,
    errors: Vec<BuildError>,
}

impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    /// Constructs a new `Router`.
    /// Avoid using this directly, use `Rspc::router` instead so the types can be inferred.
    pub(crate) fn new() -> Self {
        Self {
            procedures: Vec::new(),
            errors: Vec::new(),
        }
    }

    #[track_caller]
    pub fn procedure<T, TMiddleware>(
        mut self,
        key: &'static str,
        procedure: Procedure<T, TMiddleware>,
    ) -> Self
    where
        Procedure<T, TMiddleware>: DynProcedure<TCtx>,
        TMiddleware: MiddlewareBuilder + SealedMiddlewareBuilder<Ctx = TCtx, LayerCtx = TCtx>,
    {
        if let Some(cause) = is_valid_name(key) {
            self.errors.push(BuildError {
                cause,
                #[cfg(debug_assertions)]
                name: Cow::Borrowed(key),
                #[cfg(debug_assertions)]
                loc: Location::caller(),
            });
        }

        let procedure: Box<dyn DynProcedure<TMiddleware::Ctx>> = Box::new(procedure);
        self.procedures.push((Cow::Borrowed(key), procedure));
        self
    }

    #[track_caller]
    #[allow(unused_mut)]
    pub fn merge(mut self, prefix: &'static str, mut r: Router<TCtx>) -> Self {
        if let Some(cause) = is_valid_name(prefix) {
            self.errors.push(BuildError {
                cause,
                #[cfg(debug_assertions)]
                name: Cow::Borrowed(prefix),
                #[cfg(debug_assertions)]
                loc: Location::caller(),
            });
        }

        #[cfg(not(debug_assertions))]
        {
            self.errors.append(&mut r.errors);
        }

        #[cfg(debug_assertions)]
        {
            self.errors.extend(&mut r.errors.into_iter().map(|mut err| {
                err.name = Cow::Owned(format!("{}.{}", prefix, err.name));
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

        // TODO: Eventually take these as an argument so we can access the plugin store from the parent router -> For this we do this for compat
        let mut queries = ProcedureStore::new("queries"); // TODO: Take in as arg
        let mut mutations = ProcedureStore::new("mutations"); // TODO: Take in as arg
        let mut subscriptions = ProcedureStore::new("subscriptions"); // TODO: Take in as arg
        let mut typ_store = TypeDefs::new(); // TODO: Take in as arg

        let mut ctx = BuildProceduresCtx {
            ty_store: &mut typ_store,
            queries: &mut queries,
            mutations: &mut mutations,
            subscriptions: &mut subscriptions,
        };

        for (key, mut procedure) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            procedure.build(key, &mut ctx);
        }

        let router = CompiledRouter {
            queries,
            mutations,
            subscriptions,
            typ_store,
        };

        BuildResult::Ok(router)
    }
}
