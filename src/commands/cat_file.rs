use std::io;

use crate::object::read_object;

pub fn cat_file(hash: String) -> io::Result<String> {
    let obj_buf = read_object(hash)?;

    let header_content_split: Vec<&[u8]> = obj_buf.split(|&byte| byte == 0).collect();
    let content = header_content_split[1].to_vec();

    Ok(String::from_utf8(content).unwrap())
}
