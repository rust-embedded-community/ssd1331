#!/bin/sh

set -e

if [ -z $TARGET ]; then
    echo "TARGET environment variable required but not set"

    exit 1
fi

cargo build --target $TARGET --all-features --release

cargo test --lib --target x86_64-unknown-linux-gnu
cargo test --doc --target x86_64-unknown-linux-gnu

if [ -z $DISABLE_EXAMPLES ]; then
	cargo build --target $TARGET --all-features --examples
fi

cargo doc --all-features

linkchecker target/$TARGET/doc/ssd1331/index.html
