#!/bin/sh

# Exit early on error
set -e

# Print commands as they're run
set -x

if [ -z $TARGET ]; then
    echo "TARGET environment variable required but not set"

    exit 1
fi

cargo fmt --all -- --check

# todo not building all features
cargo build --target $TARGET --release

cargo test --lib --target x86_64-unknown-linux-gnu
cargo test --doc --target x86_64-unknown-linux-gnu

if [ -z $DISABLE_EXAMPLES ]; then
    # todo list of example directories in metadata so other
    (cd ssd1331-examples/stm32f1-examples && cargo build --examples)
    (cd ssd1331-examples/embassy-nrf && cargo build --examples)
fi

# Remove stale docs - the linkchecker might miss links to old files if they're not removed
cargo clean --doc
cargo clean --doc --target $TARGET

# todo not building all features
cargo doc --target $TARGET

linkchecker target/$TARGET/doc/ssd1331/index.html
