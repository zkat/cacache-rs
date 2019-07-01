//! Functions for iterating over the cache.
use std::path::Path;

use crate::index;

/// Returns an iterator that lists all cache index entries.
pub fn all<P: AsRef<Path>>(cache: P) -> impl Iterator {
    index::ls(cache.as_ref())
}
