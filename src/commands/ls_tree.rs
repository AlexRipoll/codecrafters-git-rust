use std::io;
use std::io::prelude::*;

use crate::object::read_object;
use crate::object::TreeEntry;

pub fn ls_tree(hash: String) -> io::Result<()> {
    // Lock stdout for writing
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let obj_buf = read_object(hash)?;

    // Split the decompressed data and check the header
    let (_header, data) = obj_buf.split_at(
        obj_buf.iter().position(|&x| x == 0u8).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid tree object format")
        })? + 1,
    );

    // Iterate through tree entries
    let mut pos = 0;
    while pos < data.len() {
        let null_pos = data
            .iter()
            .skip(pos)
            .position(|&x| x == 0u8)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing null byte in tree entry",
                )
            })?;

        // Get the full entry and process it (the hash is 20 bytes long)
        let entry = TreeEntry::from_bytes(&data[pos..pos + null_pos + 21])?;

        pos += null_pos + 21;

        // Write the entry name to stdout
        stdout.write_all(&entry.name.as_bytes())?;
        writeln!(stdout)?;
    }

    Ok(())
}
