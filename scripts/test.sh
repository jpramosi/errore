#!/bin/bash

function println {
    echo -e "\033[1;33m$1\033[0m"
}

if [[ -z "$CI" ]]; then
    println "This script should only run in a CI environment."
    exit 1
fi

function on_exit {
    export RUSTFLAGS=""
}

trap on_exit EXIT
export RUSTFLAGS="-Zrandomize-layout"

println "Try to run tests with default features"
cargo +$CHANNEL test
println "Try to run tests with no default features"
cargo +$CHANNEL test --no-default-features
println "Try to run tests with std feature only"
cargo +$CHANNEL test --no-default-features --features errore/std
println "Try to run tests with ctor feature only"
cargo +$CHANNEL test --no-default-features --features errore/ctor
