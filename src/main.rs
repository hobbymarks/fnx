use std::path::Path;

use clap::Parser;

use fdn::{directories, regular_files};

#[derive(Debug, Parser)]
#[command(author,version,about="File and Directory Names",long_about=None)]
struct Args {
    ///input path
    #[arg(short = 'i', long, default_value = ".")]
    input_path: String,

    ///max depth
    #[arg(short = 'd', long, default_value = "1")]
    max_depth: usize,

    ///file type
    #[arg(short = 't', long, default_value = "f")]
    filetype: String,
}

fn main() {
    let args = Args::parse();

    let input_path = Path::new(&args.input_path);
    let max_depth = args.max_depth;
    let filetype = args.filetype;

    if input_path.is_dir() {
        if filetype == "f" {
            let files = regular_files(input_path, max_depth);
            for f in files.unwrap() {
                println!("{:?}", f);
            }
        } else if filetype == "d" {
            let files = directories(input_path, max_depth);
            for f in files.unwrap() {
                println!("{:?}", f);
            }
        }
    }
}
