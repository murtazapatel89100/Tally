// `ParseError` deliberately carries the full journal source so it can render a
// caret pointing at the offending token. That makes it larger than clippy's
// `result_large_err` threshold; boxing every `Result` on the (rare) error path
// would only hurt readability, so we opt out of the lint crate-wide.
#![allow(clippy::result_large_err)]

pub mod error;
pub mod journal;
pub mod model;
pub mod parser;
