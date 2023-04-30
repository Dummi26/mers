#!/usr/bin/env sh
cd ../mers
cargo build --features nushell_plugin --profile nushellplugin
cp target/nushellplugin/mers ../build_scripts/nu_plugin_mers
echo "done. now run 'register nu_plugin_mers' in nu. You should then be able to use mers-nu."
