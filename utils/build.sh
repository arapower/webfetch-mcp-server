#!/bin/sh

docker build -t rust-dev .
docker run --rm -v "$(pwd)":/workspace rust-dev cargo build --release

# ./target/release/webfetch
# ./target/release/webfetch-api
