use std::borrow::Cow;

use crate::{
    layer::{DynLayer, Layer},
    middleware_from_core::ProcedureKind,
    procedure_store::ProcedureTodo,
    Router,
};

fn boxed<TLCtx: Send + 'static>(layer: impl Layer<TLCtx>) -> Box<dyn DynLayer<TLCtx>> {
    Box::new(layer)
}

// TODO: Using track caller style thing for the panics in this function
pub fn build<TCtx>(
    key: Cow<'static, str>,
    ctx: &mut Router<TCtx>,
    kind: ProcedureKind,
    layer: impl Layer<TCtx> + 'static,
) where
    TCtx: Send + 'static,
{
    let (map, type_name) = match kind {
        ProcedureKind::Query => (&mut ctx.queries, "query"),
        // ProcedureKind::Mutation => (&mut ctx.mutations, "mutation"),
        // ProcedureKind::Subscription => (&mut ctx.subscriptions, "subscription"),
        _ => todo!(),
    };

    let key_org = key;
    let key = key_org.to_string();
    let type_def = layer
        .into_procedure_def(key_org, &mut ctx.typ_store)
        .expect("error exporting types");

    // TODO: Cleanup this logic and do better router merging
    #[allow(clippy::panic)]
    if key.is_empty() || key == "ws" || key.starts_with("rpc.") || key.starts_with("rspc.") {
        panic!("rspc error: attempted to create {type_name} operation named '{key}', however this name is not allowed.");
    }

    #[allow(clippy::panic)]
    if map.contains_key(&key) {
        panic!("rspc error: {type_name} operation already has resolver with name '{key}'");
    }

    map.insert(
        key,
        ProcedureTodo {
            exec: boxed(layer),
            ty: type_def,
        },
    );
}
