use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::{thread, time};

use sha2::{Digest, Sha256};

fn write_zero_padded_hex(mut n: u64, buffer: &mut [u8]) {
    let hex_chars = b"0123456789abcdef";
    for i in (0..buffer.len()).rev() {
        let digit = (n % 16) as usize;
        buffer[i] = hex_chars[digit];
        n /= 16;
        if n == 0 {
            break;
        }
    }
}

fn find_matching_hash(message: &str, prefix_length: u32) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    let num_threads = num_cpus::get() as u64;
    let found = Arc::new(AtomicBool::new(false));
    let prefix_length = prefix_length as usize;

    for thread_id in 0..num_threads {
        let tx = tx.clone();
        let message = message.to_owned();
        let found = Arc::clone(&found);

        thread::spawn(move || {
            let target = 16u64.pow(prefix_length as u32);
            let start = thread_id * (target / num_threads);
            let end = if thread_id == num_threads - 1 {
                target
            } else {
                (thread_id + 1) * (target / num_threads)
            };

            let message_part = format!("{}", message).into_bytes();
            let message_len = message_part.len();
            let mut buffer = message_part.clone();
            buffer.resize(message_len + prefix_length, b'0');
            let mut hasher = Sha256::new();

            for counter in start..end {
                if found.load(Ordering::Acquire) {
                    return;
                }

                {
                    let counter_part = &mut buffer[message_len..];
                    write_zero_padded_hex(counter, counter_part);
                }

                hasher.update(&buffer);
                let hash = hasher.finalize_reset();
                let hash_bytes = hash.as_slice();

                let byte_count = (prefix_length + 1) / 2;
                if byte_count > hash_bytes.len() {
                    continue;
                }
                let mut hash_prefix = 0u64;
                for i in 0..byte_count {
                    hash_prefix = (hash_prefix << 8) | (hash_bytes[i] as u64);
                }
                if prefix_length % 2 != 0 {
                    hash_prefix >>= 4;
                }

                if hash_prefix == counter {
                    let result = String::from_utf8(buffer.clone()).unwrap();
                    tx.send(Some(result)).unwrap();
                    found.store(true, Ordering::Release);
                    return;
                }
            }

            tx.send(None).unwrap();
        });
    }

    drop(tx);

    for received in rx {
        if let Some(matching_message) = received {
            return Some(matching_message);
        }
    }

    None
}

fn main() {
    println!("Enter the message:");
    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("Failed to read message");
    s = s.trim().to_string();

    println!("Enter the prefix length:");
    let mut prefix_input = String::new();
    std::io::stdin()
        .read_line(&mut prefix_input)
        .expect("Failed to read prefix length");
    let prefix_length = prefix_input
        .trim()
        .parse::<u32>()
        .expect("Invalid prefix length");

    let start = time::Instant::now();
    match find_matching_hash(&s, prefix_length) {
        Some(matching_message) => println!("Matching message: {}", matching_message),
        None => println!("No matching message found."),
    }
    println!("Time elapsed: {:.2?}", start.elapsed());
}
