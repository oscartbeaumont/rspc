///! These structures are the Typescript // TODO
use std::borrow::Cow;

use specta::DataType;

/// @internal
#[cfg_attr(test, derive(specta::DataTypeFrom))]
#[cfg_attr(test, derive(specta::Type))]
pub(crate) struct ProceduresDef {
    #[cfg_attr(test, specta(type = ProcedureDef))]
    pub queries: Vec<ProcedureDef>,
    #[cfg_attr(test, specta(type = ProcedureDef))]
    pub mutations: Vec<ProcedureDef>,
    #[cfg_attr(test, specta(type = ProcedureDef))]
    pub subscriptions: Vec<ProcedureDef>,
}

// impl ProceduresDef {
//     pub fn new<'a, TCtx: 'a>(
//         queries: impl Iterator<Item = &'a ProcedureTodo<TCtx>>,
//         mutations: impl Iterator<Item = &'a ProcedureTodo<TCtx>>,
//         subscriptions: impl Iterator<Item = &'a ProcedureTodo<TCtx>>,
//     ) -> Self {
//         ProceduresDef {
//             queries: queries.map(|i| &i.ty).cloned().collect(),
//             mutations: mutations.map(|i| &i.ty).cloned().collect(),
//             subscriptions: subscriptions.map(|i| &i.ty).cloned().collect(),
//         }
//     }

//     pub fn to_named(self) -> NamedDataType {
//         let struct_type: StructType = self.into();
//         struct_type.to_named("Procedures")
//     }
// }

/// Represents a Typescript procedure file which is generated by the Rust code.
/// This is codegenerated Typescript file is how we can validate the types on the frontend match Rust.
///
/// @internal
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(specta::DataTypeFrom))]
#[cfg_attr(test, derive(specta::Type))]
pub(crate) struct ProcedureDef {
    pub key: Cow<'static, str>,
    #[cfg_attr(test, specta(type = serde_json::Value))]
    pub input: DataType,
    #[cfg_attr(test, specta(type = serde_json::Value))]
    pub result: DataType,
    #[cfg_attr(test, specta(type = serde_json::Value))]
    pub error: DataType,
}

// impl ProcedureDef {
//     pub fn from_tys<TArg, TResult, TError>(
//         key: Cow<'static, str>,
//         type_map: &mut TypeMap,
//     ) -> Result<Self, ts::ExportError>
//     where
//         TArg: Type,
//         TResult: Type,
//         TError: Type,
//     {
//         Ok(ProcedureDef {
//             key,
//             input: match TArg::reference(type_map, &[]).inner {
//                 DataType::Tuple(tuple) if tuple.elements().is_empty() => never(),
//                 t => t,
//             },
//             result: TResult::reference(type_map, &[]).inner,
//             error: TError::reference(type_map, &[]).inner,
//         })
//     }
// }

// fn never() -> DataType {
//     Infallible::inline(&mut Default::default(), &[])
// }

// TODO: Export types in a unit test