#!/usr/bin/env bash

set -e

bash build_platform.sh linux

# Copy the git-p binary for linux
mkdir -p doc/linux
cp platforms/linux/target/x86_64-unknown-linux-musl/release/git-p doc/linux/

# Generate the docs
bash build_doc.sh
