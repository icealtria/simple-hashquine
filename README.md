# Simple Hashquine
Generate message contains the first few digits of its own sha-256.

# Usage
```sh
cargo run --release <message> <prefix_length>
```
example
```sh
cargo run --release "The first six of sha-256 of this text are" 6
```
output:
```
The first six of sha-256 of this text are b9c3fd
```
