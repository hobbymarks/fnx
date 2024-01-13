use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

use fdn::{dir_base, directories, fdn_f, is_hidden, regular_files, utils::s_compare};

#[derive(Debug, Parser)]
#[command(author,about="File and Directory Names",long_about=None)]
struct Args {
    ///file path
    #[arg(short = 'f', long, default_value = ".")]
    file_path: String,

    ///in place
    #[arg(short = 'i', long, default_value = "false")]
    in_place: bool,

    ///max depth
    #[arg(short = 'd', long, default_value = "1")]
    max_depth: usize,

    ///file type
    #[arg(short = 't', long, default_value = "f")]
    filetype: String,

    ///ignore hidden file
    #[arg(short = 'I', long, default_value = "false")]
    not_ignore_hidden: bool,

    ///align origin and edited
    #[arg(short = 'a', long, default_value = "false")]
    align: bool,

    ///print version
    #[arg(short = 'V', long)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ///Config pattern
    Config {
        ///List all configurations
        #[arg(short = 'l', long, default_value = "false")]
        list: bool,

        ///Config Separators,Terms ...
        #[arg(short = 'c', long)]
        config: String,

        ///Delete configurations
        #[arg(short = 'd', long)]
        delete: String,
    },
}

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
    let max_depth = args.max_depth;
    let filetype = args.filetype;
    let align_flag = args.align;
    let not_ignore_hidden = args.not_ignore_hidden;

    if input_path.is_dir() {
        if filetype == "f" {
            let files = regular_files(input_path, max_depth);
            for f in files.unwrap() {
                if !not_ignore_hidden && is_hidden(&f) {
                    continue;
                }
                if let Some(d_b) = dir_base(&f) {
                    let rlt = fdn_f(&d_b, args.in_place).unwrap();

                    let (o_r, e_r) = match align_flag {
                        true => s_compare(&d_b.base, &rlt, "a"),
                        false => s_compare(&d_b.base, &rlt, ""),
                    };
                    if !o_r.eq(&e_r) {
                        if args.in_place {
                            println!("   {}\n==>{}", o_r, e_r);
                        } else {
                            println!("   {}\n-->{}", o_r, e_r);
                        }
                    }
                }
            }
        } else if filetype == "d" {
            let files = directories(input_path, max_depth);
            for f in files.unwrap() {
                println!("iD:{:?}", dir_base(&f));
            }
        }
    } else if input_path.is_file() {
        println!("File:{:?}", dir_base(input_path));
    } else {
        println!("Other:{:?}", dir_base(input_path));
    }

    Ok(())
}
