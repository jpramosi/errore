#!/bin/bash

function println {
    echo -e "\033[1;33m$1\033[0m"
}

if [[ -z "$CI" ]]; then
    println "This script should only run in a CI environment."
    exit 1
fi

function build_no_std {
    local arch="$1"
    local flags="$2"
    local features="$3"

    if [ "${features}" ]; then
        features="--features $features"
    fi

    println "Try to build no-std target for architecture '$arch'"
    export RUSTFLAGS="-C panic=abort $flags"
    cargo +$CHANNEL build --target "$arch" --manifest-path tests/no-std/Cargo.toml --no-default-features $features
    export RUSTFLAGS=""
}

println "Try to build for host architecture '$RUNNER_ARCH'"
cargo +$CHANNEL build --all
println "Try to build example-optional with 'thiserror' feature"
cargo +$CHANNEL build --package example-optional --features thiserror
println "Try to build example-optional with 'errore' feature"
cargo +$CHANNEL build --package example-optional --features errore

# ctor is not supported
build_no_std "aarch64-unknown-linux-gnu" "-C link-arg=-nostartfiles -C target-feature=-outline-atomics -C linker=aarch64-linux-gnu-gcc"
build_no_std "riscv32imac-unknown-none-elf"
build_no_std "thumbv7em-none-eabihf" "-C link-arg=-nostartfiles -C linker=arm-linux-gnueabihf-gcc"
build_no_std "wasm32-unknown-unknown"

# not working in CI environment:
# https://github.com/crosstool-ng/crosstool-ng/issues/1548
# build_no_std "armv7-unknown-linux-gnueabi" "-C link-arg=-mapcs -C link-arg=-mabi=aapcs-linux -C link-arg=-nostartfiles -C linker=arm-linux-gnueabihf-gcc"
# build_no_std "armv7-unknown-linux-gnueabihf" "-C link-arg=-nostartfiles -C linker=arm-linux-gnueabihf-gcc"
