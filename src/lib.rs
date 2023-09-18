use std::io::Write;
use wasm_bindgen::prelude::wasm_bindgen;

use sha2::{Digest, Sha256};

#[wasm_bindgen]
pub fn find_matching_hash(message: &str, prefix_length: u32) -> Option<String> {
    let target = 16u64.pow(prefix_length);
    let mut hasher = Sha256::new();
    let mut buffer = Vec::with_capacity(message.len() + prefix_length as usize);

    for counter in 0..target {
        buffer.clear();
        write!(
            &mut buffer,
            "{} {:0width$x}",
            message,
            counter,
            width = prefix_length as usize
        )
        .unwrap();

        hasher.update(&buffer);
        let hash_string = hex::encode(hasher.finalize_reset());
        let hash_prefix = &hash_string[..prefix_length as usize];

        if &buffer[message.len() + 1..] == hash_prefix.as_bytes() {
            return Some(String::from_utf8(buffer).unwrap());
        }
    }

    None
}
