# libstrophe-sys-bindgen

## Documentation

See [full documentation](https://docs.rs/libstrophe-sys-bindgen)

## Usage

Add this to your Cargo.toml:
```
[dependencies]
libstrophe-sys-bindgen = "7.1.0"
```

![Maintenance](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)
[![Build Status](https://github.com/twistedfall/libstrophe/actions/workflows/libstrophe.yml/badge.svg)](https://github.com/twistedfall/libstrophe/actions/workflows/libstrophe.yml)
[![Documentation](https://docs.rs/libstrophe-sys-bindgen/badge.svg)](https://docs.rs/libstrophe-sys-bindgen)

## libstrophe C library bindings

This crate provides bindings to [libstrophe] C library which enables you the creation of XMPP
clients and servers. The bindings were statically generated using [bindgen] so the crate doesn't
have a hard dependency on bindgen. If you still want to regenerate the bindings during building
of the create, enable `buildtime_bindgen` feature.

Usage of this crate creates runtime dependency on libstrophe.so so be sure to install that using
your package manager.

Current bindings were generated from libstrophe version: 0.14.0

The difference from [libstrophe-sys] crate is that this one is automatically generated hence
easier to maintain.

This crate contains only C bindings, for Rust ergonomic interface see [libstrophe][libstrophe_crate] crate.

[libstrophe]: http://strophe.im/libstrophe
[bindgen]: https://crates.io/crates/bindgen
[libstrophe-sys]: https://crates.io/crates/libstrophe-sys
[libstrophe_crate]: https://crates.io/crates/libstrophe

License: LGPL-3.0
