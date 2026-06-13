check:
    cargo check

release-checks:
    cargo build
    cargo build --all-features
    cargo clippy --all-features
    cargo test --no-default-features
    cargo test --all-features
    cargo doc --features wasm
