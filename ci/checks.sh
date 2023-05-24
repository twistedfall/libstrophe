#!/bin/bash

set -xeu

cargo fmt --check

clippy -v --workspace --all-targets --all-features -- -D warnings
