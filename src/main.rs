use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

use fdn::{
    config_add, config_delete, config_list, directories, fdn_fs_post, fdn_rfs_post, regular_files,
    Args, Commands,
};

fn main() -> Result<()> {
    let args = Args::parse();

    //process version
    if args.version {
        println!(
            "fdn\nVersion {}\nBuild {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        );

        return Ok(());
    }

    //process subcommands
    if let Some(ref subcmd) = args.command {
        match subcmd {
            Commands::Config {
                list: ls,
                add: cfg,
                delete: dlt,
            } => {
                if let Some(word) = cfg {
                    config_add(word)?;

                    return Ok(());
                }
                if let Some(word) = dlt {
                    config_delete(word)?;

                    return Ok(());
                }
                if *ls {
                    config_list()?;

                    return Ok(());
                }
            }
        }
    }

    //process fdn
    let input_path = match args.file {
        Some(ref v) => Path::new(v),
        None => Path::new(&args.file_path),
    };
    let e_arg = args.exclude_path.clone();
    let exs = e_arg.iter().map(Path::new).collect();

    if args.filetype == "f" {
        let files = match input_path.is_dir() {
            true => regular_files(input_path, args.max_depth, exs).unwrap(),
            false => vec![PathBuf::from(input_path)],
        };

        if args.reverse {
            fdn_rfs_post(files, args)?;
        } else {
            fdn_fs_post(files, args)?;
        }
    } else if args.filetype == "d" {
        let files = match input_path.is_dir() {
            true => directories(input_path, args.max_depth, exs).unwrap(),
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
