use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::{env, sync::mpsc};

use sha2::{Digest, Sha256};

fn find_matching_hash(message: &str, prefix_length: u32) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    let num_threads = num_cpus::get() as u64;
    let found = Arc::new(AtomicBool::new(false));

    for thread_id in 0..num_threads {
        let tx = tx.clone();
        let message = message.to_owned();
        let found = Arc::clone(&found);

        thread::spawn(move || {
            let target = 16u64.pow(prefix_length);

            // Calculate the range of counters for this thread
            let start = thread_id * (target / num_threads);
            let end = if thread_id == num_threads - 1 {
                target
            } else {
                (thread_id + 1) * (target / num_threads)
            };

            let mut hasher = Sha256::new();
            let mut buffer = Vec::with_capacity(message.len() + prefix_length as usize);

            for counter in start..end {
                if found.load(Ordering::Relaxed) {
                    return;
                }

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
                    tx.send(Some(String::from_utf8(buffer).unwrap())).unwrap();
                    found.store(true, Ordering::Relaxed);
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
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <message> <prefix_length>", &args[0]);
        std::process::exit(1);
    }
    
    let s = &args[1];
    let prefix_length = args[2]
        .parse::<u32>()
        .expect("Invalid prefix length.");

    match find_matching_hash(s, prefix_length) {
        Some(matching_message) => println!("Matching message: {}", matching_message),
        None => println!("No matching message found."),
    }
}