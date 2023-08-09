//! Internal types which power rspc. The module provides no guarantee of compatibility between updates, so you should be careful relying on types from it.
//!
//! WARNING: Anything in this module or submodules does not follow semantic versioning as it's considered an implementation detail.
//!

pub mod exec;
pub mod middleware;
pub mod procedure;
pub mod resolver;

mod layer;

pub use layer::*;

mod private {
    pin_project_lite::pin_project! {
        #[project = PinnedOptionProj]
        pub enum PinnedOption<T> {
            Some {
                #[pin]
                v: T,
            },
            None,
        }
    }

    impl<T> From<T> for PinnedOption<T> {
        fn from(value: T) -> Self {
            Self::Some { v: value }
        }
    }
}

pub(crate) use private::{PinnedOption, PinnedOptionProj};

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::PathBuf};

    use specta::{ts::export_datatype, DefOpts, Type, TypeDefs};

    use crate::internal::exec;

    macro_rules! collect_datatypes {
        ($( $i:path ),* $(,)? ) => {{
            use specta::DataType;

            let mut tys = TypeDefs::default();

            $({
                let def = <$i as Type>::definition(DefOpts {
                    parent_inline: true,
                    type_map: &mut tys,
                });

                if let Ok(def) = def {
                    if let DataType::Named(n) = def {
                        if let Some(sid) = n.sid {
                            tys.insert(sid, Some(n));
                        }
                    }
                }
            })*
            tys
        }};
    }

    // rspc has internal types that are shared between the frontend and backend. We use Specta directly to share these to avoid a whole class of bugs within the library itself.
    #[test]
    fn export_internal_types() {
        let mut file = File::create(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./packages/client/src/bindings.ts"),
        )
        .unwrap();

        file.write_all(
            b"// DO NOT MODIFY. This file was generated by Specta and is used to keep rspc internally type safe.\n// Checkout the unit test 'export_internal_types' to see where this files comes from!",
        )
        .unwrap();

        let tys = collect_datatypes! {
            super::procedure::ProcedureDataType,
            // crate::Procedures, // TODO
            exec::Request,
            exec::Response,
        };

        for (_, ty) in tys
            .iter()
            .filter_map(|(sid, v)| v.as_ref().map(|v| (sid, v)))
        {
            file.write_all(b"\n\n").unwrap();
            file.write_all(
                export_datatype(&Default::default(), &ty, &tys)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
        }
    }
}
