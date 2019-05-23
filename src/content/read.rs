use crate::content::path::content_path;
use ssri::Integrity;
use std::path::Path;
use std::fs::{self, File};
use std::io;

pub fn open(cache: &Path, sri: &Integrity) -> io::Result<File> {
    File::open(content_path(&cache, &sri))
}

pub fn read(cache: &Path, sri: &Integrity) -> io::Result<Vec<u8>> {
    fs::read(content_path(&cache, &sri))
}

pub fn read_to_string(cache: &Path, sri: &Integrity) -> io::Result<String> {
    fs::read_to_string(content_path(&cache, &sri))
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> io::Result<u64> {
    fs::copy(content_path(&cache, &sri), to)
}
