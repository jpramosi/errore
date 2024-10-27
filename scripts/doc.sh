#!/bin/bash

function println {
    echo -e "\033[1;33m$1\033[0m"
}

# if in CI environment, try to install minijinja-cli if not found
if [[ "$CI" ]]; then
    if ! command -v "minijinja-cli" 2>&1 >/dev/null
    then
        println "Try to install minijinja-cli"
        curl -sSfL https://github.com/mitsuhiko/minijinja/releases/latest/download/minijinja-cli-installer.sh | sh
        sudo cp /root/.local/bin/minijinja-cli /usr/local/bin/
        minijinja-cli --version
    fi
fi

rm -r docs/build/* &> /dev/null
TEMPLATES="docs/templates"
minijinja-cli "$TEMPLATES/main.md.jinja" --output "docs/build/main.md"
minijinja-cli "$TEMPLATES/README.md.jinja" --output "README.md"
minijinja-cli "$TEMPLATES/src/lib.md.jinja" --output "src/lib.md"

rm -r target/doc &> /dev/null
cargo +nightly doc --no-deps
