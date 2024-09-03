use clap::Parser;
use clap::Subcommand;
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

            let mut d = ZlibDecoder::new(&object[..]);
            let mut s = String::new();
            d.read_to_string(&mut s)?;
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
    }

    Ok(())
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
