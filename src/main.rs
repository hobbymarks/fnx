use anyhow::{Ok, Result};
use clap::Parser;
use std::path::{Path, PathBuf};

use fdn::{directories, fdn_fs_post, fdn_rfs_post, regular_files, Args};

fn main() -> Result<()> {
    let args = Args::parse();

    if args.version {
        println!(
            "fdn\nVersion {}\nBuild {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        );

        return Ok(());
    }

    let input_path = Path::new(&args.file_path);

    if args.filetype == "f" {
        let files = match input_path.is_dir() {
            true => regular_files(input_path, args.max_depth).unwrap(),
            false => vec![PathBuf::from(input_path)],
        };

        if args.reverse {
            fdn_rfs_post(files, args)?;
        } else {
            fdn_fs_post(files, args)?;
        }
    } else if args.filetype == "d" {
        let files = match input_path.is_dir() {
            true => directories(input_path, args.max_depth).unwrap(),
            false => panic!("input path not match filetype"),
        };

        if args.reverse {
            fdn_rfs_post(files, args)?;
        } else {
            fdn_fs_post(files, args)?;
        }
    }

    Ok(())
}
