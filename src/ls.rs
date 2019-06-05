//! Functions for iterating over the cache.
use std::path::Path;

use crate::index;

pub fn all(cache: &Path) -> impl Iterator {
    index::ls(cache)
}
