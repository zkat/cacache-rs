use crate::content::path::content_path;
use ssri::Integrity;
use std::path::Path;
use std::fs::File;

pub fn open(cache: &Path, sri: &Integrity) -> std::io::Result<File> {
    File::open(content_path(&cache, &sri))
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> std::io::Result<u64> {
    std::fs::copy(content_path(&cache, &sri), to)
}
