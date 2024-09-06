use std::fs;
use std::io;

use crate::object::compute_sha1;
use crate::object::write_object;

pub fn hash_object(file: String) -> io::Result<()> {
    let content = fs::read(file)?;
    let header = format!("blob {}\0", content.len());
    let mut object = header.into_bytes();
    object.extend_from_slice(&content);

    let hash = compute_sha1(&object);

    write_object(&hash, &object)?;
    println!("{}", hex::encode(&hash));

    Ok(())
}
