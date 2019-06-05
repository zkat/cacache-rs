//! cacache is a Rust library for managing local key and content address
//! caches. It's really fast, really good at concurrency, and it will never
//! give you corrupted data, even if cache files get corrupted or manipulated.

mod content;
mod errors;
mod index;

pub mod get;
pub mod put;
pub mod rm;
pub mod ls;

pub use errors::Error;
pub use index::Entry;
