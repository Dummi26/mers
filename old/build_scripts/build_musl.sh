#!/usr/bin/env sh
cd ../mers
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/mers ../build_scripts
