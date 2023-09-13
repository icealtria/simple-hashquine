use std::thread;
use std::{env, sync::mpsc};

use sha2::{Digest, Sha256};

fn find_matching_hash(message: &str, prefix_length: u32) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    let num_threads = num_cpus::get();

    for _ in 0..num_threads {
        let tx = tx.clone();
        let message = message.to_owned();
        thread::spawn(move || {
            let mut counter: u64 = 0;
            let target = 16u64.pow(prefix_length);

            while counter < target {
                let format_hash = format!("{:0width$x}", counter, width = prefix_length as usize);
                let msg = format!("{} {}", message, format_hash);

                let mut hasher = Sha256::new();
                hasher.update(msg.as_bytes());
                let hash_value = hasher.finalize();
                let hash_string = hex::encode(hash_value);
                let hash_prefix = &hash_string[..prefix_length as usize];

                if hash_prefix == format_hash {
                    tx.send(Some(msg)).unwrap();
                    return;
                }

                counter += 1;
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
    let args: Vec<String> = env::args().collect();
    let s = &args[1];
    let prefix_length = args[2].parse::<u32>().expect("Invalid prefix length.");

    match find_matching_hash(s, prefix_length) {
        Some(matching_message) => println!("Matching message: {}", matching_message),
        None => println!("No matching message found."),
    }
}
