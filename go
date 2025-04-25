#!/usr/bin/bash

set -xe
cargo build
cargo run
cargo test
