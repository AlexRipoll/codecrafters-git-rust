use std::fs;
use std::io;
use std::path::Path;

use crate::object::compute_sha1;
use crate::object::write_blob;
use crate::object::write_object;
use crate::object::TreeEntry;

pub fn write_tree(path: &Path) -> io::Result<Vec<u8>> {
    let mut tree_entries: Vec<TreeEntry> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;

        if entry.file_name() == ".git" {
            continue;
        }

        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();

        if entry.path().is_dir() {
            let mode = "40000".to_string();
            let hash = write_tree(&entry.path())?;

            tree_entries.push(TreeEntry {
                mode: "40000".to_string(),
                name,
                sha: hash,
            })
        } else if entry.path().is_file() {
            // create blob
            let hash = write_blob(&entry.path())?;

            tree_entries.push(TreeEntry {
                mode: "100644".to_string(),
                name,
                sha: hash,
            })
        }
    }

    // Sort tree entries lexicographically by name
    tree_entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut entries: Vec<u8> = Vec::new();
    for entry in tree_entries {
        let entry_format = format!("{} {}\0", entry.mode, entry.name);
        entries.extend_from_slice(entry_format.as_bytes());
        entries.extend_from_slice(&entry.sha);
    }

    let mut tree_buf: Vec<u8> = Vec::new();
    let header = format!("tree {}\0", entries.len());
    tree_buf.extend_from_slice(header.as_bytes());
    tree_buf.extend_from_slice(&entries);

    let hash = compute_sha1(&tree_buf);
    write_object(&hash, &tree_buf)?;

    Ok(hash)
}
