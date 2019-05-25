use crate::content::path;
use ssri::Integrity;
use std::path::Path;
use std::fs::{self, File};
use std::io;

pub fn open(cache: &Path, sri: &Integrity) -> io::Result<File> {
    File::open(path::content_path(&cache, &sri))
}

pub fn read(cache: &Path, sri: &Integrity) -> io::Result<Vec<u8>> {
    fs::read(path::content_path(&cache, &sri))
}

pub fn read_to_string(cache: &Path, sri: &Integrity) -> io::Result<String> {
    fs::read_to_string(path::content_path(&cache, &sri))
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> io::Result<u64> {
    fs::copy(path::content_path(&cache, &sri), to)
}

pub fn has_content(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if path::content_path(&cache, &sri).exists() {
        Some(sri.clone())
    } else {
        None
    }
}
