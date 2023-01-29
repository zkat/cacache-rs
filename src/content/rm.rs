use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::path;
use crate::errors::{IoErrorExt, Result};

pub fn rm(cache: &Path, sri: &Integrity) -> Result<()> {
    fs::remove_file(path::content_path(cache, sri)).with_context(|| {
        format!(
            "Failed to remove cache file {}",
            path::content_path(cache, sri).display()
        )
    })?;
    Ok(())
}

pub async fn rm_async(cache: &Path, sri: &Integrity) -> Result<()> {
    crate::async_lib::remove_file(path::content_path(cache, sri))
        .await
        .with_context(|| {
            format!(
                "Failed to remove cache file {}",
                path::content_path(cache, sri).display()
            )
        })?;
    Ok(())
}
