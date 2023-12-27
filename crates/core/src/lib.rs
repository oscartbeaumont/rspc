//! Core traits and types for [rspc](https://docs.rs/rspc)
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]
#![warn(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod executor;
mod format;
#[doc(hidden)]
pub mod internal;
mod router;
mod serializer;
mod task;

pub use executor::{Executor, Procedure};
pub use format::{Format, TODOSerializer};
pub use router::Router;
pub use serializer::Serializer;
pub use task::Task;
