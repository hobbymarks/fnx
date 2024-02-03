pub mod scmp;
pub use scmp::s_compare;

pub mod enc_dec;
pub use enc_dec::{decrypted, encrypted, hashed_name};

pub mod db;
pub use db::{
    create_records_table, create_separators_table, create_term_words_table,
    create_to_sep_words_table, delete_records, delete_separator, delete_term_word,
    delete_to_sep_word, insert_record, insert_separator, insert_term_word, insert_to_sep_word,
    open_db, retrieve_term_words, update_records, update_separator, update_term_word,
    update_to_sep_word,
};
