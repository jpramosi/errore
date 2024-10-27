#!/bin/bash

function println {
    echo -e "\033[1;33m$1\033[0m"
}

if [[ -z "$CI" ]]; then
    println "This script should only run in a CI environment."
    exit 1
fi

function add_architecture {
    local arch="$1"
    rustup target add "$arch" --toolchain "$CHANNEL"
}

add_architecture aarch64-unknown-linux-gnu
add_architecture armv7-unknown-linux-gnueabi
add_architecture armv7-unknown-linux-gnueabihf
add_architecture riscv32imac-unknown-none-elf
add_architecture thumbv7em-none-eabihf
add_architecture wasm32-unknown-unknown
cargo +$CHANNEL generate-lockfile
cargo +$CHANNEL check --locked
