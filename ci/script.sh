#!/bin/bash

set -xeu

cargo test -v --features=creds-test -- --test-threads=1
cargo test -v --release --features=creds-test -- --test-threads=1

cargo test -v --no-default-features --features=creds-test,libstrophe-0_9_3 -- --test-threads=1
cargo test -v --no-default-features --features=creds-test,libstrophe-0_10_0 -- --test-threads=1
cargo test -v --no-default-features --features=creds-test,libstrophe-0_11_0 -- --test-threads=1
cargo test -v --no-default-features --features=creds-test,libstrophe-0_12_0 -- --test-threads=1
