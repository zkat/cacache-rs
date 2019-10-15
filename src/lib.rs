//! cacache is a Rust library for managing local key and content address
//! caches. It's really fast, really good at concurrency, and it will never
//! give you corrupted data, even if cache files get corrupted or manipulated.
//!
//! ## Examples
//! ```
//! use cacache;
//! # use tempfile;
//! # let tmp = tempfile::tempdir().unwrap();
//! let dir = tmp.path().to_owned();
//! cacache::put::data(&dir, "key", b"my-data").unwrap();
//! let data = cacache::get::read(&dir, "key").unwrap();
//! assert_eq!(data, b"my-data");
//! ```
//!
//! You can also use the equivalent async APIs using async/await!
//! ```
//! # use tempfile;
//! use cacache;
//! # use async_std::task;
//! # let tmp = tempfile::tempdir().unwrap();
//! let dir = tmp.path().to_owned();
//! # task::block_on(async {
//! cacache::async_put::data(&dir, "key", b"my-async-data").await.unwrap();
//! let data = cacache::async_get::read(&dir, "key").await.unwrap();
//! assert_eq!(data, b"my-async-data");
//! # })
//! ```
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

pub mod async_get;
pub mod async_put;
pub mod async_rm;

pub use errors::Error;
pub use index::Entry;
