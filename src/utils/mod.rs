pub mod scmp;
pub use scmp::s_compare;

pub mod enc_dec;
pub use enc_dec::{decrypted, encrypted, hashed_name};

pub mod db;
pub use db::{
    create_records_table, create_sep_words_table, create_separators_table, delete_records,
    delete_sep_word, delete_separator, insert_record, insert_sep_word, insert_separator, open_db,
    update_records, update_sep_word, update_separator,
};
