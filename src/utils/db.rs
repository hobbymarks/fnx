use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use directories::UserDirs;
use rusqlite::{params, Connection};

use crate::{Record, Separator, TermWord, ToSepWord};

const DEFAULT_DB_NAME: &str = "fdn.db";
const SEP_WORD: &str = "_";
const TOBE_SEP_S: [&str; 24] = [
    "：", ":", "，", ",", "！", "!", "？", "?", "（", "(", ")", "【", "[", "】", "]", "~", "》",
    "《", "▯", "“", "”", "\"", " ", "-",
];

//////////separators
///Create separators table via database connection
pub fn create_separators_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS separators (
                    id      INTEGER PRIMARY KEY,
                    value   TEXT NOT NULL UNIQUE,
                    created TIMESTAMP DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
                )",
        (),
    )?;

    Ok(())
}

///Create from separators via database connection
pub fn insert_separator(conn: &Connection, sep: &str) -> Result<()> {
    conn.execute("INSERT INTO separators (value) VALUES (?1)", params![sep])?;
    Ok(())
}

///Retrieve from separators via database connection
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

///Update from separators via database connection
pub fn update_separator(conn: &Connection, id: i32, new_value: &str) -> Result<()> {
    conn.execute(
        "UPDATE separators SET value = ?1 WHERE id = ?2",
        params![new_value, id],
    )?;

    Ok(())
}

///Delete from separators via database connection
pub fn delete_separator(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM separators WHERE id = ?", params![id])?;

    Ok(())
}

//////////to_sep_words
///Create to_sep_words table via database connection
pub fn create_to_sep_words_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS to_sep_words (
                    id      INTEGER PRIMARY KEY,
                    value   TEXT NOT NULL UNIQUE,
                    created TIMESTAMP DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
                )",
        (),
    )?;

    Ok(())
}

///Create from to_sep_words via database connection
pub fn insert_to_sep_word(conn: &Connection, word: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO to_sep_words (value) VALUES (?1)",
        params![word],
    )?;
    Ok(())
}

///Retrieve from to_sep_words via database connection
pub fn retrieve_to_sep_words(conn: &Connection) -> Result<Vec<ToSepWord>> {
    let mut stmt = conn.prepare("SELECT id,value FROM to_sep_words")?;
    let rows = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut results = Vec::new();
    for row_rlt in rows {
        let (id, value) = row_rlt?;
        results.push(ToSepWord { id, value });
    }

    Ok(results)
}

///Update from to_sep_words via database connection
pub fn update_to_sep_word(conn: &Connection, id: i32, new_value: &str) -> Result<()> {
    conn.execute(
        "UPDATE to_sep_words SET value = ?1 WHERE id = ?2",
        params![new_value, id],
    )?;

    Ok(())
}

///Delete from to_sep_words via database connection
pub fn delete_to_sep_word(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM to_sep_words WHERE id = ?", params![id])?;

    Ok(())
}

//////////term_words
///Create term_words table via database connection
pub fn create_term_words_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS term_words (
                    id      INTEGER PRIMARY KEY,
                    key     TEXT NOT NULL UNIQUE,
                    value   TEXT,
                    created TIMESTAMP DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
                )",
        (),
    )?;

    Ok(())
}

///Insert into term_words via database connection
pub fn insert_term_word(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO term_words (key,value) VALUES (?1,?2)",
        params![key, value],
    )?;
    Ok(())
}

///Retrieve from term_words via database connection
pub fn retrieve_term_words(conn: &Connection) -> Result<Vec<TermWord>> {
    let mut stmt = conn.prepare("SELECT id,key,value FROM term_words")?;
    let rows = stmt.query_map(params![], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

    let mut results = Vec::new();
    for row_rlt in rows {
        let (id, key, value) = row_rlt?;
        results.push(TermWord { id, key, value });
    }

    Ok(results)
}

///Update from term_words via database connection
pub fn update_term_word(conn: &Connection, id: i32, new_value: &str) -> Result<()> {
    conn.execute(
        "UPDATE term_words SET value = ?1 WHERE id = ?2",
        params![new_value, id],
    )?;

    Ok(())
}

///Delete from term_words via database connection
pub fn delete_term_word(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM term_words WHERE id = ?", params![id])?;

    Ok(())
}

//////////records
///Create records table via database connection
pub fn create_records_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
                    id                      INTEGER PRIMARY KEY,
                    hashed_current_name     TEXT NOT NULL,
                    encrypted_previous_name TEXT NOT NULL,
                    count                   INTEGER,
                    created     TIMESTAMP DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
                )",
        (),
    )?;

    Ok(())
}

///Create from records via database connection
pub fn insert_record(conn: &Connection, record: Record) -> Result<()> {
    conn.execute("INSERT INTO records (hashed_current_name, encrypted_previous_name, count) VALUES (?1, ?2, ?3)", params![record.hashed_current_name,record.encrypted_pre_name,record.count])?;

    Ok(())
}

///Retrieve from records via database connection
pub fn retrieve_records(conn: &Connection) -> Result<Vec<Record>> {
    let mut stmt =
        conn.prepare("SELECT id,hashed_current_name,encrypted_previous_name,count FROM records")?;
    let rows = stmt.query_map(params![], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    let mut results = Vec::new();
    for row_rlt in rows {
        let (id, hashed_current_name, encrypted_pre_name, count) = row_rlt?;
        results.push(Record {
            id,
            hashed_current_name,
            encrypted_pre_name,
            count,
        });
    }

    Ok(results)
}

///Update from records via database connection
pub fn update_records(conn: &Connection, id: i32, origin: &str, target: &str) -> Result<()> {
    conn.execute(
        "UPDATE records SET encrypted_previous_name = ?1, hashed_current_name = ?2 WHERE id = ?3",
        params![origin, target, id],
    )?;

    Ok(())
}

///Delete from records via database connection
pub fn delete_records(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM records WHERE id = ?", params![id])?;

    Ok(())
}

//
fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    let sql = format!(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='{}';",
        name
    );
    let mut stmt = conn.prepare(&sql)?;
    let exists = stmt.exists([])?;

    Ok(exists)
}

///Open database and return database connection via Result
pub fn open_db(db_path: Option<&str>) -> Result<Connection> {
    let mut t_c_map: HashMap<String, fn(&Connection) -> Result<()>> = HashMap::new();
    t_c_map.insert(String::from("separators"), create_separators_table);
    t_c_map.insert(String::from("to_sep_words"), create_to_sep_words_table);
    t_c_map.insert(String::from("term_words"), create_term_words_table);
    t_c_map.insert(String::from("records"), create_records_table);

    let db_path = match db_path {
        Some(v) => Path::new(v),
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
            &db_dir.join(DEFAULT_DB_NAME)
        }
    };

    if !db_path.exists() {
        match Connection::open(db_path) {
            core::result::Result::Ok(conn) => {
                //Create separators table and initial it with default value
                create_separators_table(&conn)?;
                let sep = Separator {
                    id: 0,
                    value: SEP_WORD.to_owned(),
                };
                insert_separator(&conn, &sep.value)?;

                //Create to_sep_words table and initial it with default value
                create_to_sep_words_table(&conn)?;
                for w in TOBE_SEP_S {
                    let to_sep_word = ToSepWord {
                        id: 0,
                        value: w.to_owned(),
                    };
                    insert_to_sep_word(&conn, &to_sep_word.value)?;
                }

                //Create term words table
                create_term_words_table(&conn)?;

                //Create records table
                create_records_table(&conn)?;

                Ok(conn)
            }
            Err(err) => Err(anyhow!(format!("{}", err))),
        }
    } else {
        match Connection::open(db_path) {
            core::result::Result::Ok(conn) => {
                for (tb, c) in t_c_map {
                    match table_exists(&conn, &tb) {
                        Ok(true) => {}
                        Ok(false) => c(&conn)?,
                        Err(err) => return Err(anyhow!(format!("{}", err))),
                    }
                }
                Ok(conn)
            }
            Err(err) => Err(anyhow!(format!("{}", err))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{open_db, utils::db::DEFAULT_DB_NAME};
    use std::fs;

    #[test]
    fn test_initial_db() {
        let exist = fs::metadata(DEFAULT_DB_NAME).is_ok();
        let rlt = open_db(Some(DEFAULT_DB_NAME));
        assert!(rlt.is_ok());
        assert!(fs::metadata(DEFAULT_DB_NAME).is_ok());
        if !exist {
            assert!(fs::remove_file(DEFAULT_DB_NAME).is_ok());
        }
    }
}
