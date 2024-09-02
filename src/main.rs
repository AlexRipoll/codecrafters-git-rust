use core::panic;
use flate2::bufread::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        "cat-file" => match args[2].as_str() {
            "-p" => {
                let object_hash = &args[3];
                let subdirectory = &object_hash[0..2];
                let filename = &object_hash[2..];

                let object =
                    fs::read(format!(".git/objects/{}/{}", subdirectory, filename)).unwrap();

                let mut d = ZlibDecoder::new(&object[..]);
                let mut s = String::new();
                d.read_to_string(&mut s).unwrap();
                let pr = s.split_once("\0").unwrap();
                print!("{}", pr.1);
            }
            _ => panic!("unknown flag"),
        },
        _ => println!("unknown command: {}", args[1]),
    }
}
