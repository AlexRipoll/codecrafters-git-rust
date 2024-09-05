use clap::Parser;
use clap::Subcommand;
use core::panic;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,
        file: String,
    },
    LsTree {
        #[arg(long = "name-only")]
        name_only: bool,
        tree_sha: String,
    },
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::Init => {
            fs::create_dir(".git")?;
            fs::create_dir(".git/objects")?;
            fs::create_dir(".git/refs")?;
            fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
            println!("Initialized got directory")
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            let subdirectory = &object_hash[0..2];
            let filename = &object_hash[2..];

            let object = fs::read(format!(".git/objects/{}/{}", subdirectory, filename))?;

            let mut decoder = ZlibDecoder::new(&object[..]);
            let mut s = String::new();
            decoder.read_to_string(&mut s)?;
            let pr = s.split_once("\0").unwrap();
            print!("{}", pr.1);
        }
        Commands::HashObject { write, file } => {
            let content = fs::read(file)?;
            let header = format!("blob {}\0", content.len());

            let mut object = header.into_bytes();
            object.extend_from_slice(&content);

            let hash_hex = compute_sha1(&object);
            println!("{}", hash_hex);

            let subdirectory = format!("{}", &hash_hex[..2]);
            let filename = format!("{}", &hash_hex[2..]);

            let compressed = compress_object(&object)?;

            let subdirectory_path = format!(".git/objects/{}", subdirectory);
            if !Path::new(&subdirectory_path).exists() {
                fs::create_dir(&subdirectory_path)?;
            }

            fs::write(
                format!(".git/objects/{}/{}", subdirectory, filename),
                compressed,
            )?;
        }
        Commands::LsTree {
            name_only,
            tree_sha,
        } => {
            // Read and decompress the tree object
            let object = fs::read(format!(
                ".git/objects/{}/{}",
                &tree_sha[..2],
                &tree_sha[2..]
            ))?;
            let mut decoder = ZlibDecoder::new(&object[..]);
            let mut s = Vec::new();
            decoder.read_to_end(&mut s)?;

            // Lock stdout for writing
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            // Split the decompressed data and check the header
            let (header, data) = s.split_at(
                s.iter().position(|&x| x == 0u8).ok_or_else(|| {
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
                stdout.write_all(&entry.name)?;
                writeln!(stdout)?;
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct TreeEntry {
    mode: String,
    name: Vec<u8>,
    sha: Vec<u8>,
}

impl TreeEntry {
    fn from_bytes(data: &[u8]) -> io::Result<TreeEntry> {
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
            name: data[space_pos + 1..null_pos].to_vec(),
            sha: data[null_pos + 1..].to_vec(),
        })
    }
}

fn compute_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let hash = hasher.finalize();

    format!("{:x}", hash)
}

fn compress_object(object: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&object)?;

    encoder.finish()
}
