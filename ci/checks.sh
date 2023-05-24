#!/bin/bash

set -xeu

cargo fmt --check

cargo clippy -v --workspace --all-targets -- -D warnings
