use anyhow::Result;
use clap::{Parser, Subcommand};
use regex::Regex;
use rustc_serialize::hex::FromHex;
use std::{
    cmp::Ordering,
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use utils::{
    db::{retrieve_records, retrieve_sep_word, retrieve_separators},
    decrypted, encrypted, hashed_name, insert_record, open_db, s_compare,
};
use walkdir::WalkDir;

pub mod utils;

#[derive(Debug, Parser)]
#[command(author,about="File and Directory Names",long_about=None)]
pub struct Args {
    ///file path
    #[arg(short = 'f', long, default_value = ".")]
    pub file_path: String,

    ///in place
    #[arg(short = 'i', long, default_value = "false")]
    in_place: bool,

    ///max depth
    #[arg(short = 'd', long, default_value = "1")]
    pub max_depth: usize,

    ///file type
    #[arg(short = 't', long, default_value = "f")]
    pub filetype: String,

    ///not ignore hidden file
    #[arg(short = 'I', long, default_value = "false")]
    not_ignore_hidden: bool,

    ///reverse change
    #[arg(short = 'r', long, default_value = "false")]
    pub reverse: bool,

    ///align origin and edited
    #[arg(short = 'a', long, default_value = "false")]
    align: bool,

    ///print version
    #[arg(short = 'V', long)]
    pub version: bool,

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

#[derive(Debug, Clone)]
pub struct DirBase {
    pub dir: String,
    pub base: String,
}

#[derive(Clone)]
pub struct Separator {
    id: i32,
    pub value: String,
}

impl Default for Separator {
    fn default() -> Self {
        Self {
            id: 0,
            value: "_".to_owned(),
        }
    }
}
pub struct ToSepWord {
    id: i32,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Record {
    id: i32,
    hashed_current_name: String,
    encrypted_pre_name: String,
    count: i32,
}

impl Record {
    pub fn new(origin: &str, target: &str) -> Self {
        let hashed = hashed_name(target);
        let encrypted = encrypted(origin, target).unwrap();
        Self {
            id: 0,
            hashed_current_name: hashed,
            encrypted_pre_name: encrypted,
            count: 0,
        }
    }

    pub fn pre_name(self, current: &str) -> Result<String> {
        decrypted(&self.encrypted_pre_name, current)
    }
}

pub fn regular_files(directory: &Path, depth: usize) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in WalkDir::new(directory)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            paths.push(entry.into_path());
        }
    }

    Ok(paths)
}

pub fn directories(directory: &Path, depth: usize) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in WalkDir::new(directory)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            paths.push(entry.into_path());
        }
    }

    paths.sort_by(|a, b| {
        let a_str = a.as_os_str();
        let b_str = b.as_os_str();
        match a_str.cmp(b_str) {
            Ordering::Equal => a_str.len().cmp(&b_str.len()),
            other => other.reverse(),
        }
    });

    Ok(paths)
}

///dir_base function create DirBase struct from abs_path
pub fn dir_base(abs_path: &Path) -> Option<DirBase> {
    if let Some(base_name) = abs_path.file_name() {
        if let Some(dir_path) = abs_path.parent() {
            return Some(DirBase {
                dir: dir_path.to_str().unwrap().to_owned(),
                base: base_name.to_str().unwrap().to_owned(),
            });
        }
    }

    None
}

fn is_hidden_unix(path: &Path) -> bool {
    path.file_name().unwrap().to_str().unwrap().starts_with('.')
}

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
#[cfg(windows)]
use winapi::um::fileapi::FILE_ATTRIBUTE_HIDDEN;

#[cfg(windows)]
fn is_hidden_windows(path: &Path) -> bool {
    match path.metadata() {
        Ok(metadata) => metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0,
        Err(_) => false, // If there's an error, assume it's not hidden
    }
}

pub fn is_hidden(path: &Path) -> bool {
    #[cfg(unix)]
    {
        is_hidden_unix(path)
    }
    #[cfg(windows)]
    {
        is_hidden_windows(path)
    }
}

pub fn remove_continuous(source: &str, word: &str) -> String {
    let re = Regex::new(&format!(r"(?i){}{}+", word, word)).unwrap();
    re.replace_all(source, word).to_string()
}

pub fn fdn_f(dir_base: &DirBase, in_place: bool) -> Result<String> {
    let conn = open_db(None).unwrap();

    let sep = retrieve_separators(&conn)?;
    let sep = {
        if !sep.is_empty() {
            sep[0].clone().value
        } else {
            Separator::default().value
        }
    };

    let to_sep_words = retrieve_sep_word(&conn)?;

    let replacements_map: HashMap<_, _> = to_sep_words
        .iter()
        .map(|e| (e.value.clone(), sep.clone()))
        .collect();

    let mut base_name = dir_base.base.to_owned();

    //replace to sep words
    for (k, v) in &replacements_map {
        base_name = base_name.replace(k, v);
    }
    //remove continuous
    base_name = remove_continuous(&base_name, &sep);

    //split to stem and extension
    let mut f_stem = Path::new(&base_name).file_stem().unwrap().to_str().unwrap();
    let f_ext = Path::new(&base_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    //remove prefix and suffix sep
    f_stem = remove_prefix_sep_suffix_sep(f_stem, &sep);
    base_name = match f_ext.is_empty() {
        true => f_stem.to_owned(),
        false => format!("{}.{}", f_stem, f_ext),
    };

    //take effect
    if in_place {
        let s_path = Path::new(&dir_base.dir).join(dir_base.base.clone());
        let t_path = Path::new(&dir_base.dir).join(base_name.clone());
        fs::rename(s_path, t_path)?;
        let rd = Record::new(&dir_base.clone().base, &base_name);
        insert_record(&conn, rd)?;
    }

    Ok(base_name)
}

fn remove_prefix_sep_suffix_sep<'a>(s: &'a str, sep: &'a str) -> &'a str {
    let s = s.strip_prefix(sep).unwrap_or(s);
    s.strip_suffix(&sep).unwrap_or(s)
}

pub fn fdn_fs_post(files: Vec<PathBuf>, args: Args) -> Result<()> {
    for f in files {
        if !args.not_ignore_hidden && is_hidden(&f) {
            continue;
        }
        if let Some(d_b) = dir_base(&f) {
            let rlt = fdn_f(&d_b, args.in_place)?;

            let (o_r, e_r) = match args.align {
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

    Ok(())
}

pub fn fdn_rf(dir_base: &DirBase, in_place: bool) -> Result<Option<String>> {
    let conn = open_db(None).unwrap();

    let base_name = &dir_base.base;
    let rds = retrieve_records(&conn).unwrap();

    let map: HashMap<_, _> = rds
        .iter()
        .map(|rd| {
            (
                rd.clone().hashed_current_name,
                rd.clone().encrypted_pre_name,
            )
        })
        .collect();
    let rd = map.get(&hashed_name(base_name));

    match rd {
        Some(v) => match decrypted(v, base_name) {
            Ok(v) => {
                let rt = v.from_hex().unwrap();
                let base_name = String::from_utf8(rt).unwrap();
                //take effect
                if in_place {
                    let s_path = Path::new(&dir_base.dir).join(dir_base.base.clone());
                    let t_path = Path::new(&dir_base.dir).join(base_name.clone());
                    fs::rename(s_path, t_path)?;
                    // let rd = Record::new(&dir_base.clone().base, &base_name);
                    // insert_record(&conn, rd)?;
                }
                Ok(Some(base_name))
            }
            Err(err) => Err(err),
        },
        None => Ok(None),
    }
}

pub fn fdn_rfs_post(files: Vec<PathBuf>, args: Args) -> Result<()> {
    for f in files {
        if !args.not_ignore_hidden && is_hidden(&f) {
            continue;
        }
        if let Some(d_b) = dir_base(&f) {
            if let Ok(Some(rlt)) = fdn_rf(&d_b, args.in_place) {
                let (o_r, e_r) = match args.align {
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
            };
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::remove_prefix_sep_suffix_sep;

    #[test]
    fn test_remove_xfix_sep() {
        let sep = "_";
        let s = "_PDFScholar_";
        assert!(s.starts_with(sep));
        assert!(s.ends_with(sep));
        let t = "PDFScholar";
        assert_eq!(remove_prefix_sep_suffix_sep(s, sep), t);
    }
}
