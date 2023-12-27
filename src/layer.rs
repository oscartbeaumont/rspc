use futures::{future::ready, stream::once};
use serde_json::Value;
use specta::{ts, TypeMap};
use std::borrow::Cow;

use crate::{
    body::Body, error::ExecError, middleware_from_core::RequestContext,
    procedure_store::ProcedureDef,
};

// TODO: Remove `SealedLayer`

// TODO: Make this an enum so it can be `Value || Pin<Box<dyn Stream>>`?

#[doc(hidden)]
pub trait Layer<TLayerCtx: 'static>: Send + Sync + 'static {
    type Stream<'a>: Body + Send + 'a;

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError>;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError>;
}

pub trait DynLayer<TLCtx: 'static>: Send + Sync + 'static {
    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError>;

    fn dyn_call(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Box<dyn Body + Send + '_>;
}

impl<TLCtx: Send + 'static, L: Layer<TLCtx>> DynLayer<TLCtx> for L {
    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError> {
        Layer::into_procedure_def(self, key, ty_store)
    }

    fn dyn_call(&self, ctx: TLCtx, input: Value, req: RequestContext) -> Box<dyn Body + Send + '_> {
        match self.call(ctx, input, req) {
            Ok(stream) => Box::new(stream),
            // TODO: Avoid allocating error future here
            Err(err) => Box::new(once(ready(Err(err)))),
        }
    }
}

impl<TLCtx: Send + 'static> Layer<TLCtx> for Box<dyn DynLayer<TLCtx>> {
    type Stream<'a> = Box<dyn Body + Send + 'a>;

    fn into_procedure_def(
        &self,
        key: Cow<'static, str>,
        ty_store: &mut TypeMap,
    ) -> Result<ProcedureDef, ts::ExportError> {
        (&**self).into_procedure_def(key, ty_store)
    }

    fn call(
        &self,
        ctx: TLCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        Ok(self.dyn_call(ctx, input, req))
    }
}

pub(crate) fn boxed<TLCtx: Send + 'static>(layer: impl Layer<TLCtx>) -> Box<dyn DynLayer<TLCtx>> {
    Box::new(layer)
}
