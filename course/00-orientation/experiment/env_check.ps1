@echo off
cargo --version || exit /b 1
rustc --version || exit /b 1
cargo build -p rafiki-sae || exit /b 1
cargo run --release -p rafiki-sae --example simulate || exit /b 1
echo ENV-CHECK PASS
