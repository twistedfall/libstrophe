# libstrophe

[![Build Status](https://github.com/twistedfall/libstrophe/actions/workflows/libstrophe.yml/badge.svg)](https://github.com/twistedfall/libstrophe/actions/workflows/libstrophe.yml)
[![Documentation](https://docs.rs/libstrophe/badge.svg)](https://docs.rs/libstrophe)
[![Crates.io](https://img.shields.io/crates/v/libstrophe)](https://crates.io/crates/libstrophe)
![Maintenance](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)

[Support the project](https://github.com/sponsors/twistedfall) | [Documentation](https://docs.rs/libstrophe)


## Usage

Run:
```shell
cargo add libstrophe
```
Or add to your Cargo.toml:
```toml
[dependencies]
libstrophe = "0.20.3"
```

## libstrophe - ergonomic wrapper for Rust

This library provides high level ergonomic bindings for [libstrophe], an XMPP client library.


## Documentation

The documentation for this library covers only Rust specific bits and refers to [original
library documentation][docs] in most other cases.


## Workflow

The general workflow is quite similar to what you get with the C library. The topmost object is
[Context]. It contains platform-specific bits like logging and memory allocation. Plus an event
loop used to keep things going. This crate wraps logging with the facilities provided by [`log`]
crate (provided the default `rust-log` feature is enabled). Memory allocation is also handled by
Rust native means. When a [Connection] is created it will temporarily consume the [Context].
After all the setup is done, call one of the `connect_*()` methods to retrieve the [Context]
back. In this manner a single [Context] can be used for multiple [Connection]s consequently.
When you're done with setting up [Connection]s for the [Context], use `run()` or `run_once()`
methods to start the event loop rolling.


## Safety

This crate tries to be as safe as possible. Yet it's not always possible to guarantee that when
wrapping a C library. The following assumptions are made which might not necessarily be true and
thus might introduce unsafety:

 * [Context] event loop methods are borrowing `self` immutably considering it immutable (or
   more specifically having interior mutability)

The main objects in this crate are marked as `Send` and it should be indeed be safe to send them
between threads. Yet, no major investigation of the library source code has been performed to
ensure that this is true.


## Initialization and shutdown

You don't need to call the initialization function, it's done automatically when creating a
[Context]. Yet you might want to call the [shutdown()] function when your application
terminates. Be aware though that the initialization can be called only once in the program
lifetime so you won't be able to use the library properly after you called [shutdown()].


## Callbacks

The crate has the ability to store callbacks taking ownership of them so you can pass closures
and not care about storing them externally. There are some things to note about it though. It's
not always possible to know whether the underlying library accepted the callback. The crate will
keep the closure internally in either case, though it may not ever be called by the library. You
can still remove the callback with the corresponding `*handler_delete()` or `*handler_clear()`
method.

Due to the way the C libstrophe library is implemented and how Rust optimizes monomorphization,
your callbacks must actually be compiled to different function with separate addresses when you
pass them to the same handler setup method. So if you want to pass 2 callbacks `hander_add`
ensure that their code is unique and rust didn't merge them into a single function behind the
scenes. You can test whether 2 callbacks are same or not with the `Connection::*handlers_same()`
family of functions. If it returns true then you will only be able to pass one of them to the
corresponding handler function, the other will be silently ignored.

Due to the fact that the crate uses `userdata` to pass the actual user callback, it's not possible
to use `userdata` inside the callbacks for your own data. So if you need to have a state between
callback invocations you must use closures.

Because the main objects are marked as `Send` and we store callbacks inside them, all callbacks
must also be `Send`.


## Examples
```rust
let connection_handler = |ctx: &libstrophe::Context,
                          _conn: &mut libstrophe::Connection,
                          _evt: libstrophe::ConnectionEvent| {
   ctx.stop();
};

let ctx = libstrophe::Context::new_with_default_logger();
let mut conn = libstrophe::Connection::new(ctx);
conn.set_jid("example@127.0.0.1");
conn.set_pass("password");
let mut ctx = conn.connect_client(None, None, connection_handler).unwrap();
ctx.run();
libstrophe::shutdown();
```

For more complete examples see this crate `examples` directory and [libstrophe examples].


## Crate features

The following features are provided:

  * `rust-log` - enabled by default, makes the crate integrate into Rust logging facilities
  * `libstrophe-0_9_3` - enabled by default, enables functions specific to libstrophe-0.9.3
  * `libstrophe-0_10_0` - enabled by default, enables functions specific to libstrophe-0.10
  * `libstrophe-0_11_0` - enabled by default, enables functions specific to libstrophe-0.11
  * `libstrophe-0_12_0` - enabled by default, enables functions specific to libstrophe-0.12
  * `libstrophe-0_13` - enabled by default, enables functions specific to libstrophe-0.13
  * `libstrophe-0_14` - enabled by default, enables functions specific to libstrophe-0.14
  * `buildtime_bindgen` - forces regeneration of the bindings instead of relying on the
    pre-generated sources

[libstrophe]: https://strophe.im/libstrophe/
[`log`]: https://crates.io/crates/log
[docs]: https://strophe.im/libstrophe/doc/0.13.0/
[libstrophe examples]: https://github.com/strophe/libstrophe/tree/0.12.2/examples

## License

LGPL-3.0
