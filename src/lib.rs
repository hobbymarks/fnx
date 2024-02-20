use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};
use regex::Regex;
use rusqlite::Connection;
use rustc_serialize::hex::FromHex;
use std::{
    cmp::Ordering,
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use utils::{
    db::{insert_term_word, retrieve_records, retrieve_separators, retrieve_to_sep_words},
    decrypted, delete_records, delete_term_word, delete_to_sep_word, encrypted, hashed_name,
    insert_record, insert_to_sep_word, open_db, s_compare,
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

    ///file type,'f' for regular file and 'd' for directory
    #[arg(short = 't', long, default_value = "f")]
    pub filetype: String,

    ///not ignore hidden file
    #[arg(short = 'I', long, default_value = "false")]
    not_ignore_hidden: bool,

    ///exclude file or directory
    #[arg(short = 'X', long, default_values_t = Vec::<String>::new(), action = ArgAction::Append)]
    pub exclude_path: Vec<String>,

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
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    ///Config pattern
    Config {
        ///List all configurations
        #[arg(short = 'l', long, default_value = "false")]
        list: bool,

        ///Config Separators,Terms ...
        #[arg(short = 'c', long)]
        add: Option<String>,

        ///Delete configurations
        #[arg(short = 'd', long)]
        delete: Option<String>,
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

pub struct TermWord {
    id: i32,
    pub key: String,
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
            count: 1,
        }
    }

    // pub fn pre_name(self, current: &str) -> Result<String> {
    //     decrypted(&self.encrypted_pre_name, current)
    // }
}

pub fn regular_files(
    directory: &Path,
    depth: usize,
    excludes: Vec<&Path>,
) -> Result<Vec<PathBuf>> {
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

    for e in excludes {
        paths.retain(|f| {
            !f.as_path()
                .canonicalize()
                .unwrap()
                .starts_with(e.canonicalize().unwrap())
        });
    }

    Ok(paths)
}

pub fn directories(directory: &Path, depth: usize, excludes: Vec<&Path>) -> Result<Vec<PathBuf>> {
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

    for e in excludes {
        paths.retain(|f| {
            !f.as_path()
                .canonicalize()
                .unwrap()
                .starts_with(e.canonicalize().unwrap())
        });
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
fn dir_base(abs_path: &Path) -> Option<DirBase> {
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
    match path.file_name() {
        Some(n) => match n.to_str() {
            Some(s) => s.starts_with('.'),
            None => false,
        },
        None => false,
    }
}

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
#[cfg(windows)]
use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

use crate::utils::retrieve_term_words;

#[cfg(windows)]
fn is_hidden_windows(path: &Path) -> bool {
    match path.metadata() {
        Ok(metadata) => metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0,
        Err(_) => false, // If there's an error, assume it's not hidden
    }
}

fn is_hidden(path: &Path) -> bool {
    #[cfg(unix)]
    {
        is_hidden_unix(path)
    }
    #[cfg(windows)]
    {
        is_hidden_windows(path)
    }
}

fn remove_continuous(source: &str, word: &str) -> String {
    let re = Regex::new(&format!(r"(?i){}{}+", word, word)).unwrap();
    re.replace_all(source, word).to_string()
}

fn remove_prefix_sep_suffix_sep<'a>(s: &'a str, sep: &'a str) -> &'a str {
    let s = s.strip_prefix(sep).unwrap_or(s);
    s.strip_suffix(&sep).unwrap_or(s)
}

fn fdn_f(dir_base: &DirBase, in_place: bool) -> Result<String> {
    let conn = open_db(None).unwrap();

    let sep = retrieve_separators(&conn)?;
    let sep = {
        if !sep.is_empty() {
            sep[0].clone().value
        } else {
            Separator::default().value
        }
    };

    let mut base_name = dir_base.base.to_owned();

    //split to stem and extension
    let mut f_stem = Path::new(&base_name)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let f_ext = Path::new(&base_name)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    //replace to sep words
    let to_sep_words = retrieve_to_sep_words(&conn)?;
    let replacements_map: HashMap<_, _> = to_sep_words
        .iter()
        .map(|e| (e.value.clone(), sep.clone()))
        .collect();
    let mut old_f_stem = f_stem.clone();
    loop {
        for (k, v) in &replacements_map {
            f_stem = f_stem.replace(k, v);
        }
        if old_f_stem.eq(&f_stem) {
            break;
        } else {
            old_f_stem = f_stem.clone();
        }
    }

    //term words
    let term_words = retrieve_term_words(&conn)?;
    let replacements_map: HashMap<_, _> = term_words
        .iter()
        .map(|e| (e.key.clone(), e.value.clone()))
        .collect();
    let mut old_f_stem = f_stem.clone();
    loop {
        for (k, v) in &replacements_map {
            f_stem = f_stem.replace(k, v);
        }
        if old_f_stem.eq(&f_stem) {
            break;
        } else {
            old_f_stem = f_stem.clone();
        }
    }

    //remove continuous
    f_stem = remove_continuous(&f_stem, &sep);

    //remove prefix and suffix sep
    f_stem = remove_prefix_sep_suffix_sep(&f_stem, &sep).to_owned();

    base_name = match f_ext.is_empty() {
        true => f_stem.to_owned(),
        false => format!("{}.{}", f_stem, f_ext),
    };

    //take effect
    if base_name != dir_base.base && in_place {
        let s_path = Path::new(&dir_base.dir).join(dir_base.base.clone());
        let t_path = Path::new(&dir_base.dir).join(base_name.clone());
        fs::rename(s_path, t_path)?;
        let rd = Record::new(&dir_base.clone().base, &base_name);
        insert_record(&conn, rd)?;
    }

    Ok(base_name)
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

fn fdn_rf(dir_base: &DirBase, in_place: bool) -> Result<Option<String>> {
    let conn = open_db(None).unwrap();

    let base_name = &dir_base.base;
    let rds = retrieve_records(&conn).unwrap();

    let map: HashMap<_, _> = rds
        .iter()
        .map(|rd| (rd.clone().hashed_current_name, rd.clone()))
        .collect();
    let rd = map.get(&hashed_name(base_name));

    match rd {
        Some(rd) => match decrypted(&rd.encrypted_pre_name, base_name) {
            Ok(v) => {
                let rt = v.from_hex().unwrap();
                let base_name = String::from_utf8(rt).unwrap();
                //take effect
                if in_place {
                    let s_path = Path::new(&dir_base.dir).join(dir_base.base.clone());
                    let t_path = Path::new(&dir_base.dir).join(base_name.clone());
                    fs::rename(s_path, t_path)?;
                    if rd.count == 1 {
                        delete_records(&conn, rd.id)?;
                    }
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

fn list_to_sep_words(conn: &Connection) -> Result<()> {
    let mut rlts = retrieve_to_sep_words(conn)?;
    println!("ID\tValue");
    rlts.sort_by_key(|tsw| tsw.id);
    for tsw in rlts {
        println!("{}\t{}", tsw.id, tsw.value);
    }
    Ok(())
}
fn list_term_words(conn: &Connection) -> Result<()> {
    let mut rlts = retrieve_term_words(conn)?;
    println!("ID\tKey\tValue");
    rlts.sort_by_key(|tw| tw.id);
    for tw in rlts {
        println!("{}\t{}\t{}", tw.id, tw.key, tw.value);
    }
    Ok(())
}
pub fn config_list() -> Result<()> {
    let conn = open_db(None)?;
    list_to_sep_words(&conn)?;
    list_term_words(&conn)?;

    Ok(())
}

pub fn config_add(word: &str) -> Result<()> {
    let conn = open_db(None)?;
    match word.split_once(':') {
        Some((key, value)) => {
            insert_term_word(&conn, key, value)?;
            list_term_words(&conn)?;
        }
        None => {
            insert_to_sep_word(&conn, word)?;
            list_to_sep_words(&conn)?;
        }
    }

    Ok(())
}

pub fn config_delete(word: &str) -> Result<()> {
    let conn = open_db(None)?;
    match word.split_once(':') {
        Some((key, value)) => {
            let rlts = retrieve_term_words(&conn)?;
            let the_word = rlts.iter().find(|&w| w.key == key && w.value == value);
            if let Some(w) = the_word {
                delete_term_word(&conn, w.id)?;
                list_term_words(&conn)?;
            }
        }
        None => {
            let rlts = retrieve_to_sep_words(&conn)?;
            let the_word = rlts.iter().find(|&w| w.value == word);
            if let Some(w) = the_word {
                delete_to_sep_word(&conn, w.id)?;
                list_to_sep_words(&conn)?;
            }
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
