use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub sha: Vec<u8>,
}

impl TreeEntry {
    pub fn from_bytes(data: &[u8]) -> io::Result<TreeEntry> {
        let space_pos = data.iter().position(|&x| x == b' ').ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing space in tree entry",
        ))?;
        let null_pos = data.iter().position(|&x| x == 0u8).ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing null byte in tree entry",
        ))?;
        let mode = String::from_utf8(data[0..space_pos].to_vec())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in mode"))?;

        Ok(Self {
            mode,
            name: String::from_utf8(data[space_pos + 1..null_pos].to_vec()).unwrap(),
            sha: data[null_pos + 1..].to_vec(),
        })
    }
}

pub fn read_object(hash: String) -> io::Result<Vec<u8>> {
    let obj = fs::read(Path::new(".git/objects/").join(format!("{}/{}", &hash[..2], &hash[2..])))?;

    let mut decoder = ZlibDecoder::new(&obj[..]);
    let mut decompressed_obj = Vec::new();
    decoder.read_to_end(&mut decompressed_obj)?;

    Ok(decompressed_obj)
}

pub fn write_object(hash: &[u8], content: &[u8]) -> io::Result<()> {
    let hash_hex = hex::encode(&hash);
    let subdirectory = &hash_hex[..2].to_string();
    let filename = &hash_hex[2..].to_string();

    let subdirectory_path = Path::new(".git/objects/").join(subdirectory);

    if !&subdirectory_path.exists() {
        fs::create_dir(&subdirectory_path)?;
    }

    let compressed = compress_object(&content)?;

    fs::write(subdirectory_path.join(filename), compressed)?;

    Ok(())
}

pub fn write_blob(file_path: &Path) -> io::Result<Vec<u8>> {
    let content = fs::read(file_path)?;
    let header = format!("blob {}\0", content.len());

    let mut blob = header.into_bytes();
    blob.extend_from_slice(&content);

    let hash = compute_sha1(&blob);

    write_object(&hash, &blob)?;

    Ok(hash)
}

pub fn compute_sha1(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn compress_object(object: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(object)?;

    encoder.finish()
}
