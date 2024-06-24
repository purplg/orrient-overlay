use std::{fs::File, io::Read, path::Path};

#[derive(Debug, PartialEq)]
pub struct Trail {
    pub version: i32,
    pub map_id: i32,
    pub coordinates: Vec<Coord>,
}

#[derive(Debug, PartialEq)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    EOF,
}

/// Convenience function to try to read a Trail file.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Trail, Error> {
    read(File::open(path).map_err(Error::Io)?)
}

/// The main reader function. This will try to parse any reader of
/// u8's into a [`Trail`] struct.
pub fn read<R: Read>(mut input: R) -> Result<Trail, Error> {
    // The first 4 bytes are the trail version number which is a i32.
    let mut version_buf = [0u8; 4];
    input.read_exact(&mut version_buf).map_err(Error::Io)?;
    let version = i32::from_le_bytes(version_buf);

    // The second 4 bytes are the map id which is a i32.
    let mut map_id_buf = [0u8; 4];
    input.read_exact(&mut map_id_buf).map_err(Error::Io)?;
    let map_id = i32::from_le_bytes(map_id_buf);

    // The rest of the file are tuples of 3 f32 values. An x, y, and
    // z.
    let mut coordinates: Vec<Coord> = vec![];
    loop {
        let mut buf = [0u8; 12];
        let read = input.read(&mut buf).map_err(Error::Io)?;
        if read == 0 {
            // If no bytes are read, then we've reached the end of the
            // file and we can break from the loop.
            break;
        }
        if read < buf.len() {
            // If we read some but not enough bytes, the file is corrupt.
            return Err(Error::EOF);
        }
        let x = f32::from_le_bytes(buf[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(buf[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(buf[8..12].try_into().unwrap());
        coordinates.push(Coord { x, y, z });
    }

    Ok(Trail {
        version,
        map_id,
        coordinates,
    })
}
