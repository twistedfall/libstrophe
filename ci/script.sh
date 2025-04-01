#!/bin/bash

set -xeu

# https://stackoverflow.com/questions/4023830/how-to-compare-two-strings-in-dot-separated-version-format-in-bash
verlte() {
	[ "$1" = "`echo -e "$1\n$2" | sort -V | head -n1`" ]
}

if verlte "0.9.3" "$LIBSTROPHE_VERSION"; then
	ARGS="--no-default-features --features=buildtime_bindgen,libstrophe-0_9_3"
	cargo test -v $ARGS -- --test-threads=1
	cargo test -v $ARGS --release -- --test-threads=1
fi

if verlte "0.10.0" "$LIBSTROPHE_VERSION"; then
	ARGS="--no-default-features --features=buildtime_bindgen,libstrophe-0_10_0"
	cargo test -v $ARGS -- --test-threads=1
	cargo test -v $ARGS --release -- --test-threads=1
fi

if verlte "0.11.0" "$LIBSTROPHE_VERSION"; then
	ARGS="--no-default-features --features=buildtime_bindgen,libstrophe-0_11_0"
	cargo test -v $ARGS -- --test-threads=1
	cargo test -v $ARGS --release -- --test-threads=1
fi

if verlte "0.12.0" "$LIBSTROPHE_VERSION"; then
	cargo test -v -- --test-threads=1
	cargo test -v --release -- --test-threads=1
	cargo test -v --features=buildtime_bindgen -- --test-threads=1
	cargo test -v --release --features=buildtime_bindgen -- --test-threads=1
fi
