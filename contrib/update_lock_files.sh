#!/bin/bash
set -e

echo "Generating Cargo-minimal.lock..."
cargo +nightly update -Z minimal-versions
cp Cargo.lock Cargo-minimal.lock

echo "Generating Cargo-recent.lock..."
cargo update
cp Cargo.lock Cargo-recent.lock

echo "Done."
