use anyhow::{anyhow, Result};
use directories::UserDirs;
use regex::Regex;
use rusqlite::{params, Connection};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub mod utils;

const DEFAULT_DB_NAME: &str = "fdn.db";
const SEP_WORD: &str = "_";
const TOBE_SEP_S: [&str; 24] = [
    "：", ":", "，", ",", "！", "!", "？", "?", "（", "(", ")", "【", "[", "】", "]", "~", "》",
    "《", "▯", "“", "”", "\"", " ", "-",
];

#[derive(Debug)]
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

pub struct Record {
    id: i32,
    encrypted_previous_name: String,
    hashed_current_name: String,
    count: i32,
}

impl Record {
    pub fn new(self, origin: String, target: String) -> Self {
        Self {
            id: 0,
            encrypted_previous_name: origin,
            hashed_current_name: target,
            count: 0,
        }
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

//////////separators
//Create separators table
pub fn create_separators(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS separators (
                    id    INTEGER PRIMARY KEY,
                    value TEXT NOT NULL
                )",
        (),
    )?;

    Ok(())
}

//Create from separators
pub fn insert_separator(conn: &Connection, sep: &str) -> Result<()> {
    conn.execute("INSERT INTO separators (value) VALUES (?1)", params![sep])?;
    Ok(())
}

//Retrieve from separators
pub fn retrieve_separators(conn: &Connection) -> Result<Vec<Separator>> {
    let mut stmt = conn.prepare("SELECT id,value FROM separators")?;
    let rows = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut results = Vec::new();
    for row_rlt in rows {
        let (id, value) = row_rlt?;
        results.push(Separator { id, value });
    }

    Ok(results)
}

//Update from separators
pub fn update_separator(conn: &Connection, id: i32, new_value: &str) -> Result<()> {
    conn.execute(
        "UPDATE separators SET value = ?1 WHERE id = ?2",
        params![new_value, id],
    )?;

    Ok(())
}

//Delete from separators
pub fn delete_separator(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM separators WHERE id = ?", params![id])?;

    Ok(())
}

//////////to_sep_words
//Create to_sep_words table
pub fn create_sep_words(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS to_sep_words (
                    id    INTEGER PRIMARY KEY,
                    value TEXT NOT NULL
                )",
        (),
    )?;

    Ok(())
}

//Create from to_sep_words
pub fn insert_sep_word(conn: &Connection, word: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO to_sep_words (value) VALUES (?1)",
        params![word],
    )?;
    Ok(())
}

//Retrieve from to_sep_words
pub fn retrieve_sep_word(conn: &Connection) -> Result<Vec<ToSepWord>> {
    let mut stmt = conn.prepare("SELECT id,value FROM to_sep_words")?;
    let rows = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut results = Vec::new();
    for row_rlt in rows {
        let (id, value) = row_rlt?;
        results.push(ToSepWord { id, value });
    }

    Ok(results)
}

//Update from to_sep_words
pub fn update_sep_word(conn: &Connection, id: i32, new_value: &str) -> Result<()> {
    conn.execute(
        "UPDATE to_sep_words SET value = ?1 WHERE id = ?2",
        params![new_value, id],
    )?;

    Ok(())
}

//Delete from to_sep_words
pub fn delete_sep_word(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM to_sep_words WHERE id = ?", params![id])?;

    Ok(())
}

//////////records
//Create records table
pub fn create_records(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
                    id    INTEGER PRIMARY KEY,
                    encrypted_previous_name TEXT NOT NULL,
                    hashed_current_name TEXT NOT NULL,
                    count INTEGER
                )",
        (),
    )?;

    Ok(())
}

//Create from records
pub fn insert_record(conn: &Connection, origin: &str, target: &str, count: i32) -> Result<()> {
    conn.execute("INSERT INTO records (encrypted_previous_name, hashed_current_name, count) VALUES (?1, ?2, ?3)", params![origin,target,count])?;

    Ok(())
}

//Retrieve from records
pub fn retrieve_records(conn: &Connection) -> Result<Vec<Record>> {
    let mut stmt =
        conn.prepare("SELECT id,encrypted_previous_name,hashed_current_name,count FROM records")?;
    let rows = stmt.query_map(params![], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    let mut results = Vec::new();
    for row_rlt in rows {
        let (id, encrypted_previous_name, hashed_current_name, count) = row_rlt?;
        results.push(Record {
            id,
            encrypted_previous_name,
            hashed_current_name,
            count,
        });
    }

    Ok(results)
}

//Update from records
pub fn update_records(conn: &Connection, id: i32, origin: &str, target: &str) -> Result<()> {
    conn.execute(
        "UPDATE records SET encrypted_previous_name = ?1, hashed_current_name = ?2 WHERE id = ?3",
        params![origin, target, id],
    )?;

    Ok(())
}

//Delete from records
pub fn delete_records(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM records WHERE id = ?", params![id])?;

    Ok(())
}

pub fn open_db(db_path: Option<&str>) -> Result<Connection> {
    let db_path = match db_path {
        Some(v) => PathBuf::from(v),
        None => {
            let db_dir = match UserDirs::new() {
                Some(v) => {
                    let path = v.home_dir().to_path_buf().join(".fdn");
                    if !path.exists() {
                        match fs::create_dir_all(path.clone()) {
                            Ok(()) => path,
                            Err(err) => panic!("{}", err),
                        }
                    } else {
                        path
                    }
                }
                None => PathBuf::from("."),
            };
            db_dir.join(DEFAULT_DB_NAME)
        }
    };

    if !db_path.exists() {
        match Connection::open(db_path) {
            core::result::Result::Ok(conn) => {
                //Create separators table and initial it with default value
                create_separators(&conn)?;
                let sep = Separator {
                    id: 0,
                    value: SEP_WORD.to_owned(),
                };
                insert_separator(&conn, &sep.value)?;

                //Create to_sep_words table and initial it with default value
                create_sep_words(&conn)?;
                for w in TOBE_SEP_S {
                    let to_sep_word = ToSepWord {
                        id: 0,
                        value: w.to_owned(),
                    };
                    insert_sep_word(&conn, &to_sep_word.value)?;
                }

                //Create records table
                create_records(&conn)?;

                Ok(conn)
            }
            Err(err) => Err(anyhow!(format!("{}", err))),
        }
    } else {
        match Connection::open(db_path) {
            core::result::Result::Ok(conn) => Ok(conn),
            Err(err) => Err(anyhow!(format!("{}", err))),
        }
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
        .unwrap();

    //remove prefix suffix sep
    f_stem = f_stem
        .strip_prefix(&sep)
        .unwrap_or(f_stem)
        .strip_suffix(&sep)
        .unwrap_or(f_stem);
    base_name = format!("{}.{}", f_stem, f_ext);

    //take effect
    if in_place {
        let s_path = Path::new(&dir_base.dir).join(dir_base.base.clone());
        let t_path = Path::new(&dir_base.dir).join(base_name.clone());
        fs::rename(s_path, t_path)?;
        insert_record(&conn, &dir_base.base, &base_name, 1)?;
    }

    Ok(base_name)
}

#[cfg(test)]
mod tests {

    use crate::{open_db, remove_continuous, DEFAULT_DB_NAME};

    #[test]
    fn test_initial_db() {
        let rlt = open_db(Some(DEFAULT_DB_NAME));
        assert!(rlt.is_ok())
    }

    #[test]
    fn test_remove_continuous() {
        let src = "A_B__C___D_.txt";
        let sep = "_";
        let tgt = "A_B_C_D_.txt";
        assert_eq!(remove_continuous(src, sep), tgt);
    }
}
