use std::{borrow::Cow, collections::HashMap};

use serde::{
    de::{value::I32Deserializer, IntoDeserializer},
    Deserialize, Deserializer,
};

use crate::{
    format::{DynFormat, Format},
    runtime::Runtime,
    Procedure,
};

/// TODO
pub struct RouterBuilder<TCtx>(Vec<(Cow<'static, str>, Procedure<TCtx>)>);

impl<TCtx> RouterBuilder<TCtx> {
    /// TODO
    pub fn procedure(mut self, key: &'static str, procedure: Procedure<TCtx>) -> Self {
        self.0.push((Cow::Borrowed(key), procedure));
        self
    }

    /// TODO: Register custom types to export

    /// TODO
    pub fn build(self) -> Result<Router<TCtx>, ()> {
        // TODO: Check for invalid or duplicate procedure names.

        let mut router = Router(Default::default());
        for (key, procedure) in self.0 {
            (procedure.build)(key, &mut router);
        }
        Ok(router)
    }
}

pub(crate) type ExecutableProcedure<TCtx> =
    Box<dyn Fn(&mut dyn DynFormat, TCtx, &mut dyn erased_serde::Deserializer)>;

/// TODO
pub struct Router<TCtx = ()>(pub(crate) HashMap<Cow<'static, str>, ExecutableProcedure<TCtx>>);

impl<TCtx> Router<TCtx> {
    /// TODO
    pub fn new() -> RouterBuilder<TCtx> {
        RouterBuilder(Vec::new())
    }

    // TODO: Specta exporting but support any Specta exporter.

    /// TODO: Dump the router for custom integrations

    /// TODO
    pub fn exec<'de, F: Format, R: Runtime>(
        &self,
        key: &str,
        ctx: TCtx,
        input: impl Deserializer<'de>,
        // TODO: Custom error type
    ) -> R::Output<Result<F::Output, ()>> {
        let procedure = self.0.get(key).unwrap(); // TODO: .ok_or(())?;
        let mut serializer = F::new();
        procedure(
            &mut serializer,
            ctx,
            &mut <dyn erased_serde::Deserializer>::erase(input),
        );

        // TODO: With `async` in batches we need to ensure the `Serialize` is put somewhere.

        // TODO: Result before or after async runtime???

        // Ok(serializer.take())
        todo!();
    }
}
