use anyhow::{anyhow, Result};
use bevy::math::Vec3;
use std::{fs::File, io::Read, path::Path};

/// The raw trail data read directly from a file.
#[derive(Debug)]
pub struct TrailData {
    pub version: u32,
    pub map_id: u32,
    pub path: Vec<Vec3>,
}

/// Convenience function to try to read a Trail file.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<TrailData> {
    read(File::open(path)?)
}

/// The main reader function. This will try to parse any reader of
/// u8's into a [`Trail`] struct.
pub fn read<R: Read>(mut input: R) -> Result<TrailData> {
    // The first 4 bytes are the trail version number which is a u32.
    let mut version_buf = [0u8; 4];
    input.read_exact(&mut version_buf)?;
    let version = u32::from_le_bytes(version_buf);

    // The second 4 bytes are the map id which is a u32.
    let mut map_id_buf = [0u8; 4];
    input.read_exact(&mut map_id_buf)?;
    let map_id = u32::from_le_bytes(map_id_buf);

    // The rest of the file are tuples of 3 f32 values. An x, y, and
    // z.
    let mut path: Vec<Vec3> = vec![];
    let mut buf = [0u8; 12];
    loop {
        let read = input.read(&mut buf)?;
        if read == 0 {
            // If no bytes are read, then we've reached the end of the
            // file and we can break from the loop.
            return Ok(TrailData {
                version,
                map_id,
                path,
            });
        }
        if read < buf.len() {
            // If we read some but not enough bytes, the file is corrupt.
            return Err(anyhow!(
                "Only read {read} bytes when expected {len}",
                len = buf.len()
            ));
        }
        let x = f32::from_le_bytes(buf[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(buf[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(buf[8..12].try_into().unwrap());
        path.push(Vec3 { x, y, z });
    }
}
