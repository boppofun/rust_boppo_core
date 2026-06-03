check:
    cargo check

release-checks:
    cargo build
    cargo build --features wasm
    cargo clippy --features wasm
    cargo test --features wasm
    cargo doc --features wasm
