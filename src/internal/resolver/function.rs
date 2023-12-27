use std::{borrow::Cow, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use specta::{reference::Reference, ts, DataType, Type, TypeMap};

use crate::{
    internal::resolver::{result::private::StreamToBody, IntoResolverResponse},
    middleware_from_core::ProcedureKind,
};

pub struct QueryOrMutation<M>(PhantomData<M>);
pub struct Subscription<M>(PhantomData<M>);

type ArgTy = fn(&mut TypeMap) -> Reference;

// TODO: Rename `Resolver`?
pub struct HasResolver<F, TErr, TResultMarker, M> {
    pub(crate) resolver: F,
    pub(crate) kind: ProcedureKind,
    pub(crate) arg_ty: ArgTy,
    phantom: PhantomData<fn() -> (TErr, TResultMarker, M)>,
}

mod private {
    use crate::{
        error::{private::IntoResolverError, ExecError},
        layer::Layer,
        middleware_from_core::RequestContext,
        procedure_store::ProcedureDef,
    };

    use super::*;

    impl<F, TErr, TResultMarker, M> HasResolver<F, TErr, TResultMarker, M> {
        pub(crate) fn new(resolver: F, kind: ProcedureKind, arg_ty: ArgTy) -> Self {
            Self {
                resolver,
                kind,
                arg_ty,
                phantom: PhantomData,
            }
        }
    }

    pub struct M<TArg>(PhantomData<TArg>);
    impl<F, TLCtx, TArg, TResult, TResultMarker> Layer<TLCtx>
        for HasResolver<F, TResult::Err, TResultMarker, M<TArg>>
    where
        F: Fn(TLCtx, TArg) -> TResult + Send + Sync + 'static,
        TArg: DeserializeOwned + Type + 'static,
        TLCtx: Send + Sync + 'static,
        TResult: IntoResolverResponse<'static, TResultMarker>,
        TResult::Ok: Serialize + Type + 'static,
        TResult::Err: IntoResolverError + 'static,
        TResultMarker: 'static,
    {
        type Stream<'a> = StreamToBody<TResult::Stream>;

        fn into_procedure_def(
            &self,
            key: Cow<'static, str>,
            ty_store: &mut TypeMap,
        ) -> Result<ProcedureDef, ts::ExportError> {
            let mut result =
                ProcedureDef::from_tys::<TArg, TResult::Ok, TResult::Err>(key, ty_store)?;
            // TODO: Bruh this is soooo bad
            result.input = match (self.arg_ty)(ty_store).inner {
                DataType::Tuple(tuple) if tuple.elements().is_empty() => never(),
                t => t,
            };
            Ok(result)
        }

        fn call(
            &self,
            ctx: TLCtx,
            input: Value,
            req: RequestContext,
        ) -> Result<Self::Stream<'_>, ExecError> {
            let stream = (self.resolver)(
                ctx,
                serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
            )
            .to_stream();

            Ok(StreamToBody {
                stream,
                #[cfg(feature = "tracing")]
                span: req.span(),
                #[cfg(not(feature = "tracing"))]
                span: None,
            })
        }
    }
}

pub(crate) use private::M;

fn never() -> DataType {
    std::convert::Infallible::inline(&mut Default::default(), &[])
}
