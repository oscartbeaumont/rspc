// //! Integrate rspc with a http server so it can be accessed from your frontend.
// //!
// //! This is done through [httpz](https://github.com/oscartbeaumont/httpz).
// #![warn(
//     clippy::all,
//     clippy::cargo,
//     clippy::unwrap_used,
//     clippy::panic,
//     clippy::todo,
//     clippy::panic_in_result_fn,
//     // missing_docs
// )]
// #![forbid(unsafe_code)]
// #![allow(clippy::module_inception)]
// #![cfg_attr(docsrs, feature(doc_cfg))]

// mod cookie_jar;
// mod extractors;
// mod httpz_endpoint;
// mod request;
// mod websocket;

// pub use cookie_jar::*;
// pub use extractors::*;
// pub use httpz_endpoint::*;
// pub use request::*;
// pub(crate) use websocket::*;
