#!/bin/sh
set -e
cargo --version
rustc --version
cargo build -p rafiki-sae
cargo run --release -p rafiki-sae --example simulate
echo "ENV-CHECK PASS"
