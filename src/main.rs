use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use clap::Parser;
use tracing::warn;

use fdn::{
    config_add, config_delete, config_list, directories, fdn_fs_post, fdn_rfs_post, regular_files,
    Args, Commands,
};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

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
            Commands::Mv { inputs } => {
                match inputs.len().cmp(&2) {
                    Ordering::Less => {
                        return Err(anyhow!("At least two arguments are required:{:?}", inputs));
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {
                        warn!("Only the first two arguments are valid.")
                    }
                }

                let sfs = vec![PathBuf::from(inputs[0].clone())];
                if sfs.iter().all(|f| f.is_dir() || f.is_file()) {
                    let tns = vec![inputs[1].clone()];
                    fdn_fs_post(sfs, tns, args)?;
                } else {
                    return Err(anyhow!("The paths not exist:{:?}", sfs));
                }

                return Ok(());
            }
        }
    }

    //process fdn with no subcommands
    let input_paths = match args.files {
        Some(ref vs) => vs.iter().map(Path::new).collect(),
        None => vec![Path::new(&args.file_path)],
    };

    let exs: Vec<_> = args.exclude_path.iter().map(Path::new).collect();

    input_paths.iter().for_each(|f_path| {
        let args = args.clone();

        if args.filetype == "f" {
            let files = match f_path.is_dir() {
                true => regular_files(f_path, args.max_depth, exs.clone()).unwrap(),
                false => vec![PathBuf::from(f_path)],
            };

            if (args.reverse) || (args.reverse_chainly) {
                let _ = fdn_rfs_post(files, args);
            } else {
                let _ = fdn_fs_post(files, Vec::new(), args);
            }
        } else if args.filetype == "d" {
            let files = match f_path.is_dir() {
                true => directories(f_path, args.max_depth, exs.clone()).unwrap(),
                false => panic!("input path not match filetype"),
            };

            if (args.reverse) || (args.reverse_chainly) {
                let _ = fdn_rfs_post(files, args);
            } else {
                let _ = fdn_fs_post(files, Vec::new(), args);
            }
        }
    });

    Ok(())
}
