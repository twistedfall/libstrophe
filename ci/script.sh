#!/bin/bash

set -vex

cargo test -v --features=creds-test -- --test-threads=1
cargo test -v --release --features=creds-test -- --test-threads=1
