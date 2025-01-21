use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn find_matching_hash(message: &str, prefix_length: u32) -> Option<String> {
    let prefix_len = prefix_length as usize;
    let target = 16u64.pow(prefix_length);
    let message_part = format!("{} ", message);
    let mut buffer = Vec::with_capacity(message_part.len() + prefix_len);
    buffer.extend_from_slice(message_part.as_bytes());
    buffer.resize(buffer.capacity(), 0);
    let mut hasher = Sha256::new();

    for counter in 0..target {
        // Isolate mutable borrow in a nested scope
        {
            let counter_part = &mut buffer[message_part.len()..];
            write_zero_padded_hex(counter, counter_part);
        }

        hasher.update(&buffer);
        let hash = hasher.finalize_reset();

        let counter_part = &buffer[message_part.len()..]; // Immutable borrow
        if check_match(counter_part, &hash, prefix_len) {
            return Some(String::from_utf8(buffer.clone()).unwrap());
        }
    }

    None
}

fn write_zero_padded_hex(n: u64, buffer: &mut [u8]) {
    // Removed unnecessary mut
    let len = buffer.len();
    for i in 0..len {
        let nibble = (n >> (4 * (len - 1 - i))) & 0xF;
        buffer[i] = if nibble < 10 {
            b'0' + nibble as u8
        } else {
            b'a' + (nibble - 10) as u8
        };
    }
}

fn check_match(counter_part: &[u8], hash: &[u8], prefix_len: usize) -> bool {
    for j in 0..prefix_len {
        let c = counter_part[j];
        let digit = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => 10 + (c - b'a'),
            _ => return false,
        };

        let byte_idx = j / 2;
        if byte_idx >= 32 {
            return false;
        }

        let nibble = if j % 2 == 0 {
            hash[byte_idx] >> 4
        } else {
            hash[byte_idx] & 0x0F
        };

        if nibble != digit {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_hash() {
        assert_eq!(
            find_matching_hash("Hello, world!", 6),
            Some("Hello, world! 182a5e".to_string())
        );
    }
}
