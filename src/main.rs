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
    LsTree {
        #[arg(long = "name-only")]
        name_only: bool,
        tree_sha: String,
    },
    WriteTree,
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
            pretty_print: _,
            object_hash,
        } => {
            let subdirectory = &object_hash[0..2];
            let filename = &object_hash[2..];

            let object = fs::read(format!(".git/objects/{}/{}", subdirectory, filename))?;

            let mut decoder = ZlibDecoder::new(&object[..]);
            let mut s = String::new();
            decoder.read_to_string(&mut s)?;
            let pr = s.split_once('\0').unwrap();
            print!("{}", pr.1);
        }
        Commands::HashObject { write: _, file } => {
            let content = fs::read(file)?;
            let header = format!("blob {}\0", content.len());
            let mut object = header.into_bytes();
            object.extend_from_slice(&content);

            let hash = compute_sha1(&object);

            write_object(&hash, &object)?;
            println!("{}", hex::encode(&hash));
        }
        Commands::LsTree {
            name_only: _,
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
            let (_header, data) = s.split_at(
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
                stdout.write_all(&entry.name.as_bytes())?;
                writeln!(stdout)?;
            }
        }
        Commands::WriteTree => {
            let hash = write_tree(&env::current_dir().unwrap())?;
            println!("{}", hex::encode(&hash));
        }
    }

    Ok(())
}

#[derive(Debug)]
struct TreeEntry {
    mode: String,
    name: String,
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
            name: String::from_utf8(data[space_pos + 1..null_pos].to_vec()).unwrap(),
            sha: data[null_pos + 1..].to_vec(),
        })
    }
}

fn write_object(hash: &[u8], content: &[u8]) -> io::Result<()> {
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

fn write_blob(file_path: &Path) -> io::Result<Vec<u8>> {
    let content = fs::read(file_path)?;
    let header = format!("blob {}\0", content.len());

    let mut blob = header.into_bytes();
    blob.extend_from_slice(&content);

    let hash = compute_sha1(&blob);

    write_object(&hash, &blob)?;

    Ok(hash)
}

fn write_tree(path: &Path) -> io::Result<Vec<u8>> {
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

fn compute_sha1(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn compress_object(object: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(object)?;

    encoder.finish()
}
