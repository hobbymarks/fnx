use anyhow::{anyhow, Result};
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

fn key_bytes(key_plain: &str, ks: KeySize) -> Vec<u8> {
    match ks {
        KeySize::KeySize128 => {
            let mut out = [0_u8; 32];
            let mut sha = Sha256::new();
            sha.input_str(key_plain);
            sha.result(&mut out);

            out[..16].to_vec()
        }
        KeySize::KeySize192 => {
            let mut out = [0_u8; 32];
            let mut sha = Sha256::new();
            sha.input_str(key_plain);
            sha.result(&mut out);

            out[..24].to_vec()
        }
        KeySize::KeySize256 => {
            let mut out = [0_u8; 32];
            let mut sha = Sha256::new();
            sha.input_str(key_plain);
            sha.result(&mut out);

            out[..].to_vec()
        }
    }
}

pub fn encrypted(plain: &str, key_plain: &str) -> Result<String> {
    let key_size = KeySize::KeySize128;

    let mut encryptor = aes::cbc_encryptor(
        key_size,
        &key_bytes(key_plain, key_size),
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
            .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;

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

pub fn decrypted(encrypted: &str, key_plain: &str) -> Result<String> {
    let key_size = KeySize::KeySize128;

    let mut decryptor = aes::cbc_decryptor(
        key_size,
        &key_bytes(key_plain, key_size),
        &IV,
        blockmodes::PkcsPadding,
    );

    let mut final_result = Vec::<u8>::new();
    let enc = encrypted.from_hex()?;
    let mut read_buffer = buffer::RefReadBuffer::new(&enc);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;

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

#[cfg(test)]
mod tests {
    use rustc_serialize::hex::FromHex;

    use super::{decrypted, encrypted, hashed_name};

    #[test]
    fn test_hashed_name() {
        let s_v = [
            (
                "HelloWorld",
                "872e4e50ce9990d8b041330c47c9ddd11bec6b503ae9386a99da8584e9bb12c4",
            ),
            (
                "HelloWorld!",
                "729e344a01e52c822bdfdec61e28d6eda02658d2e7d2b80a9b9029f41e212dde",
            ),
        ];
        for (k, v) in s_v {
            let rlt = hashed_name(k);
            assert_eq!(rlt, v);
        }
    }

    #[test]
    fn test_enc_dec() {
        let plain = "HelloWorld";
        let k = "key";
        let enc = encrypted(plain, k).unwrap();
        let dec = decrypted(&enc, k).unwrap();
        let dec = dec.from_hex().unwrap();

        assert_eq!(plain, String::from_utf8(dec).unwrap());
    }
}
