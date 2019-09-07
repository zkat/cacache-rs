// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for iterating over the cache.
use std::path::Path;

use crate::index;

/// Returns an iterator that lists all cache index entries.
pub fn all<P: AsRef<Path>>(cache: P) -> impl Iterator {
    index::ls(cache.as_ref())
}
