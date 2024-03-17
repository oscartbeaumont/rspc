//! rspc: A blazingly fast and easy to use tRPC-like server for Rust.
//!
//! Checkout the official docs <https://rspc.dev>
//!

// TODO: Clippy lints + `Cargo.toml`

mod format;
mod procedure;
mod router;
mod runtime;

pub use format::Format;
pub use procedure::{Procedure, ProcedureBuilder, ProcedureFunc};
pub use router::{Router, RouterBuilder};

// TODO: Remove this from public API
pub use format::JsonValue;
pub use runtime::TokioRuntime;
pub use serde;
pub use serde_json;
