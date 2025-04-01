//! # libstrophe C library bindings
//!
//! This crate provides bindings to [libstrophe] C library which enables you the creation of XMPP
//! clients and servers. The bindings were statically generated using [bindgen] so the crate doesn't
//! have a hard dependency on bindgen. If you still want to regenerate the bindings during building
//! of the crate, enable `buildtime_bindgen` feature.
//!
//! Usage of this crate creates runtime dependency on libstrophe.so so be sure to install that using
//! your package manager.
//!
//! Current bindings were generated from libstrophe version: 0.12.0
//!
//! The difference from [libstrophe-sys] crate is that this one is automatically generated hence
//! easier to maintain.
//!
//! This crate contains only C bindings, for Rust ergonomic interface see [libstrophe][libstrophe_crate] crate.
//!
//! [libstrophe]: http://strophe.im/libstrophe
//! [bindgen]: https://crates.io/crates/bindgen
//! [libstrophe-sys]: https://crates.io/crates/libstrophe-sys
//! [libstrophe_crate]: https://crates.io/crates/libstrophe

#[allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code, unused_imports)]
mod ffi;

pub use crate::ffi::*;
