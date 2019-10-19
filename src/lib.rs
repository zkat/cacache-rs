//! cacache is a Rust library for managing local key and content address
//! caches. It's really fast, really good at concurrency, and it will never
//! give you corrupted data, even if cache files get corrupted or manipulated.
//!
//! ## Examples
//!
//! Un-suffixed APIs are all async, using
//! [`async-std`](https://crates.io/crates/async-std). They let you put data
//! in and get it back out -- asynchronously!
//!
//! ```no_run
//! use async_attributes;
//!
//! #[async_attributes::main]
//! async fn main() -> Result<(), cacache::Error> {
//!   // Data goes in...
//!   cacache::put::data("./my-cache", "key", b"hello").await?;
//!
//!   // ...data comes out!
//!   let data = cacache::get::data("./my-cache", "key").await?;
//!   assert_eq!(data, b"hello");
//!
//!   Ok(())
//! }
//! ```
//!
//! ### Lookup by hash
//!
//! What makes `cacache` content addressable, though, is its ability to fetch
//! data by its "content address", which in our case is a ["subresource
//! integrity" hash](https://crates.io/crates/ssri), which `cacache::put`
//! conveniently returns for us. Fetching data by hash is significantly faster
//! than doing key lookups:
//!
//! ```no_run
//! use async_attributes;
//!
//! #[async_attributes::main]
//! async fn main() -> Result<(), cacache::Error> {
//!   // Data goes in...
//!   let sri = cacache::put::data("./my-cache", "key", b"hello").await?;
//!
//!   // ...data gets looked up by `sri` ("Subresource Integrity").
//!   let data = cacache::get::data_hash("./my-cache", &sri).await?;
//!   assert_eq!(data, b"hello");
//!
//!   Ok(())
//! }
//! ```
//!
//! ### Large file support
//!
//! `cacache` supports large file reads, in both async and sync mode, through
//! an API reminiscent of `std::fs::OpenOptions`:
//!
//! ```no_run
//! use async_attributes;
//! use async_std::prelude::*;
//!
//! #[async_attributes::main]
//! async fn main() -> Result<(), cacache::Error> {
//!   let mut fd = cacache::put::PutOpts::new().open("./my-cache", "key").await?;
//!   for _ in 0..10 {
//!     fd.write_all(b"very large data").await?;
//!   }
//!   // Data is only persisted to the cache after you do `fd.commit()`!
//!   let sri = fd.commit().await?;
//!   println!("integrity: {}", &sri);
//!
//!   let mut fd = cacache::get::open("./my-cache", "key").await?;
//!   let mut buf = String::new();
//!   fd.read_to_string(&mut buf).await?;
//!
//!   // Make sure to call `.check()` when you're done! It makes sure that what
//!   // you just read is actually valid. `cacache` always verifies the data
//!   // you get out is what it's supposed to be. The check is very cheap!
//!   fd.check()?;
//!
//!   Ok(())
//! }
//! ```
//!
//! ### Sync API
//!
//! There are also sync APIs available if you don't want to use async/await:
//!
//! ```no_run
//! fn main() -> Result<(), cacache::Error> {
//!   cacache::put::data_sync("./my-cache", "key", b"my-data").unwrap();
//!   let data = cacache::get::data_sync("./my-cache", "key").unwrap();
//!   assert_eq!(data, b"my-data");
//!   Ok(())
//! }
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

pub use errors::Error;
pub use index::Entry;
