// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for asynchronously removing things from the cache.
//!
//! Asynchronous operations are able to trade off some linear performance in
//! exchange for potentially much higher performance on heavily-concurrent
//! loads.
use std::path::Path;

use async_std::fs as afs;
use ssri::Integrity;

use crate::content::rm;
use crate::errors::Error;
use crate::index;

/// Removes an individual index entry. The associated content will be left
/// intact.
pub async fn entry<P: AsRef<Path>>(cache: P, key: &str) -> Result<(), Error> {
    index::delete_async(cache.as_ref(), &key).await
}

/// Removes an individual content entry. Any index entries pointing to this
/// content will become invalidated.
pub async fn content<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<(), Error> {
    rm::rm_async(cache.as_ref(), &sri).await
}

/// Removes entire contents of the cache, including temporary files, the entry
/// index, and all content data.
pub async fn all<P: AsRef<Path>>(cache: P) -> Result<(), Error> {
    for entry in cache.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            afs::remove_dir_all(entry.path()).await?;
        }
    }
    Ok(())
}
