use clap::Parser;
use clap::Subcommand;
use std::env;
use std::fs;
use std::io;

mod commands;
mod object;

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
            let blob = commands::cat_file::cat_file(object_hash.to_string())?;
            print!("{}", blob);
        }
        Commands::HashObject { write: _, file } => {
            commands::hash_object::hash_object(file.to_string())?;
        }
        Commands::LsTree {
            name_only: _,
            tree_sha,
        } => {
            commands::ls_tree::ls_tree(tree_sha.to_string())?;
        }
        Commands::WriteTree => {
            let hash = commands::write_tree::write_tree(&env::current_dir().unwrap())?;
            println!("{}", hex::encode(&hash));
        }
    }

    Ok(())
}
