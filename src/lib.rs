//! cacache is a Rust library for managing local key and content address
//! caches. It's really fast, really good at concurrency, and it will never
//! give you corrupted data, even if cache files get corrupted or manipulated.

#![warn(missing_docs, missing_doc_code_examples)]

pub use serde_json::Value;
pub use ssri::Algorithm;

mod content;
mod errors;
mod index;

pub mod get;
pub mod ls;
pub mod put;
pub mod rm;

pub use errors::Error;
pub use index::Entry;
