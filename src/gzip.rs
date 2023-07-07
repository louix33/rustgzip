use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, BufWriter};
use crate::deflate::deflate;
use std::time::SystemTime;

pub fn compress_to_gzip(src_path: &str, dst_path: &str) -> Result<(), Box<dyn Error>> {
    let mut src_file = File::open(src_path)?;
    let dst_file = File::create(dst_path)?;

    let mut raw_data = Vec::new();
    src_file.read_to_end(&mut raw_data)?;

    let deflated_data = deflate(&raw_data)?;

    let mut writer = BufWriter::new(dst_file);

    writer.write_all(&[0x1f, 0x8b, // fixed values
                       0x08, // compression method: deflate
                       0x08, // flags, FNAME set
                       ])?;

    // write mtime
    let mtime = src_file.metadata()?.modified()?.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as u32;
    writer.write_all(&mtime.to_le_bytes())?;

    // write extra flags, os
    writer.write_all(&[0x00, 0x03])?;

    // write file name
    let filename = std::path::Path::new(src_path).file_name().unwrap().to_str().unwrap();
    writer.write_all(filename.as_bytes())?;
    writer.write_all(&[0x00])?;

    // write deflate data
    writer.write_all(&deflated_data)?;

    // write crc32
    let crc32 = crc32fast::hash(&raw_data);
    writer.write_all(&crc32.to_le_bytes())?;

    // write isize
    writer.write_all(&(raw_data.len() as u32).to_le_bytes())?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip() {
        compress_to_gzip("/home/louis/rust/test.txt", "/home/louis/rust/test.gz").unwrap();
    }
}