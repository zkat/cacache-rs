//! Functions for iterating over the cache.
use std::path::Path;

use crate::index;

pub fn all<P: AsRef<Path>>(cache: P) -> impl Iterator {
    index::ls(cache.as_ref())
}
