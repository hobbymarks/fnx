use anyhow::{anyhow, Result};
use directories::UserDirs;
use rusqlite::{params, Connection};
use std::{fs, path::PathBuf};

use crate::{Record, Separator, ToSepWord};

const DEFAULT_DB_NAME: &str = "fdn.db";
const SEP_WORD: &str = "_";
const TOBE_SEP_S: [&str; 24] = [
    "：", ":", "，", ",", "！", "!", "？", "?", "（", "(", ")", "【", "[", "】", "]", "~", "》",
    "《", "▯", "“", "”", "\"", " ", "-",
];

//////////separators
//Create separators table
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

//Create from to_sep_words
pub fn insert_to_sep_word(conn: &Connection, word: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO to_sep_words (value) VALUES (?1)",
        params![word],
    )?;
    Ok(())
}

//Retrieve from to_sep_words
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

//Update from to_sep_words
pub fn update_to_sep_word(conn: &Connection, id: i32, new_value: &str) -> Result<()> {
    conn.execute(
        "UPDATE to_sep_words SET value = ?1 WHERE id = ?2",
        params![new_value, id],
    )?;

    Ok(())
}

//Delete from to_sep_words
pub fn delete_to_sep_word(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM to_sep_words WHERE id = ?", params![id])?;

    Ok(())
}

//////////records
//Create records table
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

//Create from records
pub fn insert_record(conn: &Connection, record: Record) -> Result<()> {
    conn.execute("INSERT INTO records (hashed_current_name, encrypted_previous_name, count) VALUES (?1, ?2, ?3)", params![record.hashed_current_name,record.encrypted_pre_name,record.count])?;

    Ok(())
}

//Retrieve from records
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

                //Create records table
                create_records_table(&conn)?;

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

#[cfg(test)]
mod tests {

    use crate::{open_db, remove_continuous, utils::db::DEFAULT_DB_NAME};

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
