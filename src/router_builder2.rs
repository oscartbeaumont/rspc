use std::{borrow::Cow, collections::BTreeMap, panic::Location};

use thiserror::Error;

use crate::{procedure_store::ProcedureTodo, Router};

// TODO: Move into `procedure_store` module???
pub type ProcedureMap<TCtx> = BTreeMap<String, ProcedureTodo<TCtx>>;

/// TODO
#[derive(Debug, PartialEq, Eq)]
pub struct BuildError {
    pub(crate) cause: BuildErrorCause,
    #[cfg(debug_assertions)]
    pub(crate) name: Cow<'static, str>,
    #[cfg(debug_assertions)]
    pub(crate) loc: &'static Location<'static>,
}

/// TODO
#[derive(Debug, PartialEq, Eq)]
pub struct BuildResultBuildError {
    pub(crate) cause: BuildErrorCause,
    #[cfg(debug_assertions)]
    pub(crate) name: Cow<'static, str>,
    #[cfg(debug_assertions)]
    pub(crate) loc: &'static Location<'static>,
}

#[cfg(debug_assertions)]
pub fn edit_build_error_name(
    error: &mut BuildError,
    func: impl FnOnce(Cow<'static, str>) -> Cow<'static, str>,
) {
    error.name = func(std::mem::replace(&mut error.name, Cow::Borrowed("")));
}

pub fn new_build_error(
    cause: BuildErrorCause,
    #[cfg(debug_assertions)] name: Cow<'static, str>,
    #[cfg(debug_assertions)] loc: &'static Location<'static>,
) -> BuildError {
    BuildError { cause, name, loc }
}

impl BuildError {
    /// DO NOT USE IT, it's for unit testing only and may change without a major version bump.
    #[doc(hidden)]
    #[cfg(debug_assertions)]
    pub fn expose(&self) -> (String, String) {
        (self.name.to_string(), self.cause.to_string())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BuildErrorCause {
    #[error(
        "a procedure or router name must be more than 1 character and less than 255 characters"
    )]
    InvalidName,
    #[error("a procedure or router name contains the character '{0}' which is not allowed. Names must be alphanumeric or have '_' or '-'")]
    InvalidCharInName(char),
    #[error(
        "a procedure or router name is using the name '{0}' which is reserved for internal use."
    )]
    ReservedName(String),
}

/// TODO
pub enum BuildResult<TCtx: 'static> {
    Ok(Router<TCtx>),
    Err(Vec<BuildError>),
}

impl<TCtx: 'static> PartialEq for BuildResult<TCtx> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ok(_), Self::Ok(_)) => true,
            (Self::Err(e), Self::Err(e2)) => e == e2,
            _ => false,
        }
    }
}

impl<TCtx: 'static> BuildResult<TCtx> {
    pub fn unwrap(self) -> Router<TCtx> {
        match self {
            Self::Ok(router) => router,
            Self::Err(errors) => {
                #[cfg(debug_assertions)]
                {
                    eprintln!("Error building rspc router\n");

                    for error in &errors {
                        eprintln!("Error at '{}' with procedure '{}':", error.loc, error.name);
                        eprintln!("\t error: {}\n", error.cause);
                    }
                }

                #[cfg(not(debug_assertions))]
                {
                    eprintln!("Error building rspc router. Please use a debug build for proper error reporting.\n");
                }

                std::process::exit(1);
            }
        }
    }
}
