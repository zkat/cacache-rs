// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

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

pub mod async_get;
pub mod async_put;
pub mod async_rm;

pub use errors::Error;
pub use index::Entry;
