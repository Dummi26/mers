#!/usr/bin/env sh
cd ../mers
cargo build --release
cp target/release/mers ../build_scripts
