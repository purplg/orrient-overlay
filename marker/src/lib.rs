mod marker;

use std::path::Path;

pub use marker::*;

#[derive(Debug)]
pub enum Error {
    FsErr(std::io::Error),
    DeErr(quick_xml::de::DeError),
}

pub fn read(path: &Path) -> Result<OverlayData, Error> {
    let content = std::fs::read_to_string(path).map_err(Error::FsErr)?;
    quick_xml::de::from_str(&content).map_err(Error::DeErr)
}
