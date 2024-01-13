use anyhow::Result;
use crypto::{
    aes::{self, KeySize},
    blockmodes,
    buffer::{self, BufferResult, ReadBuffer, WriteBuffer},
    digest::Digest,
    sha2::Sha256,
};
use rustc_serialize::hex::{FromHex, ToHex};
const IV: [u8; 16] = [0; 16];

pub fn hashed_name(s: &str) -> String {
    let mut sha = Sha256::new();
    sha.input_str(s);
    sha.result_str()
}

pub fn encrypted(plain: &str, key: &str) -> Result<String> {
    let mut encryptor = aes::cbc_encryptor(
        KeySize::KeySize256,
        key.as_bytes(),
        &IV,
        blockmodes::PkcsPadding,
    );

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(plain.as_bytes());
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .unwrap();

        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result.to_hex())
}

pub fn decrypted(encrypted: &str, key: &str) -> Result<String> {
    let mut decryptor = aes::cbc_decryptor(
        KeySize::KeySize256,
        key.as_bytes(),
        &IV,
        blockmodes::PkcsPadding,
    );

    let mut final_result = Vec::<u8>::new();
    let enc = encrypted.from_hex().unwrap();
    let mut read_buffer = buffer::RefReadBuffer::new(&enc);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true)
            .unwrap();
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result.to_hex())
}
