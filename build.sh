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

cargo build --target $TARGET --all-features --release

cargo test --lib --target x86_64-unknown-linux-gnu

# Always test docs against thumbv7m target as the complete readme example needs to compile against it
cargo test --doc --target thumbv7m-none-eabi

if [ -z $DISABLE_EXAMPLES ]; then
	cargo build --target $TARGET --all-features --examples
fi

cargo doc --all-features --target $TARGET

linkchecker target/$TARGET/doc/ssd1331/index.html
