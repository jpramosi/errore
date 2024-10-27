#!/bin/bash

function println {
    echo -e "\033[1;33m$1\033[0m"
}

if [[ -z "$CI" ]]; then
    println "This script should only run in a CI environment."
    exit 1
fi

function on_exit {
    export MIRIFLAGS=""
}

trap on_exit EXIT
export MIRIFLAGS="-Zrandomize-layout -Zmiri-disable-isolation -Zmiri-permissive-provenance"

cargo +$CHANNEL miri setup
println "Try to run miri with default features"
cargo +$CHANNEL miri test
println "Try to run miri with no default features"
cargo +$CHANNEL miri test --no-default-features
# 'ctor' is automatically disabled if miri is used

# cargo +$CHANNEL miri test --package errore --test test_trace_mixed -- test_trace_duplicate_location
