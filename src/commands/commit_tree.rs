use std::fmt::Write;

use std::io;
use std::time::UNIX_EPOCH;

use crate::object::compute_sha1;
use crate::object::write_object;

pub fn commit_tree(hash: String, message: String, parent_hash: Option<String>) -> io::Result<()> {
    let mut content = String::new();
    write_line(&mut content, &format!("tree {hash}"))?;
    if let Some(parent_hash) = parent_hash {
        write_line(&mut content, &format!("parent {parent_hash}"))?;
    }
    let sys_time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Time error: {:?}", e)))?
        .as_secs();
    write_line(
        &mut content,
        &format!(
            "author {} <{}> {} +0200",
            "John Doe", "johndow@gmail.com", sys_time
        ),
    )?;
    write_line(
        &mut content,
        &format!(
            "author {} <{}> {} +0200",
            "John Doe", "johndow@gmail.com", sys_time
        ),
    )?;
    write_line(&mut content, "")?;
    write_line(&mut content, &message)?;

    let header = format!("commit {}\0", content.len());

    let object = format!("{}{}", header, content);

    let hash = compute_sha1(&object.clone().into_bytes());

    write_object(&hash, &object.into_bytes())?;
    println!("{}", hex::encode(&hash));

    Ok(())
}

// Helper function to handle the writeln! errors and map them to io::Result
fn write_line(content: &mut String, line: &str) -> io::Result<()> {
    writeln!(content, "{}", line)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Write line error: {:?}", e)))
}
