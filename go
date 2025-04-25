#!/usr/bin/bash

if ! cargo fetch --offline; then
  echo "!!!!!!!!! Needing internet on first time run"
  set -xe
  #cargo update
  cargo fetch
  set +xe
  echo "!!!! Now run it again, for no internet."
  exit 0
fi
set -xe
cargo build --offline --locked
cargo run --offline --locked
cargo test --offline --locked
