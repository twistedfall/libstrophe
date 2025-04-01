#!/bin/bash

set -xeu

cargo fmt --check
cargo clippy -v --workspace -- -D warnings
