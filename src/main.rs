use clap::Parser;
use clap::Subcommand;
use flate2::bufread::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;

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
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Commands::CatFile {
            pretty_print,
            object_hash,
        } => {
            let subdirectory = &object_hash[0..2];
            let filename = &object_hash[2..];

            let object = fs::read(format!(".git/objects/{}/{}", subdirectory, filename)).unwrap();

            let mut d = ZlibDecoder::new(&object[..]);
            let mut s = String::new();
            d.read_to_string(&mut s).unwrap();
            let pr = s.split_once("\0").unwrap();
            print!("{}", pr.1);
        }
    }
}
