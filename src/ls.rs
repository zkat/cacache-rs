//! Functions for iterating over the cache.
use std::path::Path;

use crate::index;

/// Returns a synchronous iterator that lists all cache index entries.
pub fn list_sync<P: AsRef<Path>>(cache: P) -> impl Iterator {
    index::ls(cache.as_ref())
}
