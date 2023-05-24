#!/bin/bash

set -xeu

cargo fmt --check

cargo clippy -v --features=creds-test -- -D warnings
cargo clippy -v --release --features=creds-test -- -D warnings

cargo clippy -v --no-default-features --features=creds-test,libstrophe-0_9_3 -- -D warnings
cargo clippy -v --no-default-features --features=creds-test,libstrophe-0_10_0 -- -D warnings
cargo clippy -v --no-default-features --features=creds-test,libstrophe-0_11_0 -- -D warnings
cargo clippy -v --no-default-features --features=creds-test,libstrophe-0_12_0 -- -D warnings
