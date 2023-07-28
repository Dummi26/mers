#!/usr/bin/env sh
./build_musl.sh
cd ../mers
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/mers.exe ../build_scripts
