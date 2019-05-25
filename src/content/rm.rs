use crate::content::path;
use ssri::Integrity;
use std::io;
use std::fs;
use std::path::Path;

pub fn rm(cache: &Path, sri: &Integrity) -> io::Result<()> {
    fs::remove_file(path::content_path(&cache, &sri))
}
