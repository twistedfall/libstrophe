# libstrophe

## Documentation

See [full documentation](https://docs.rs/libstrophe)

## Usage

Add this to your Cargo.toml:
```
[dependencies]
libstrophe = "0.14.0"
```

[![Build Status](https://travis-ci.org/twistedfall/libstrophe.svg?branch=master)](https://travis-ci.org/twistedfall/libstrophe)

## libstrophe - ergonomic wrapper for Rust

This library provides high level ergonomic bindings for [libstrophe], an XMPP client library.


## Documentation

The documentation for this library covers only Rust specific bits and refers to [original
library documentation][docs] in most other cases.


## Workflow

The general workflow is quite similar to what you get with the C library. The topmost object is
[`Context`]. It contains platform-specific bits like logging and memory allocation. Plus an event
loop used to keep things going. This crate wraps logging with the facilities provided by [`log`]
crate (provided the default `rust-log` feature is enabled). Memory allocation is also handled by
Rust native means. When a [`Connection`] is created it will temporarily consume the [`Context`].
After all of the setup is done, call one of the `connect_*()` methods to retrieve the [`Context`]
back. In this manner a single [`Context`] can be used for multiple [`Connection`]s consequently.
When you're done with setting up [`Connection`]s for the [`Context`], use `run()` or `run_once()`
methods to start the event loop rolling.


## Safety

This create tries to be as safe as possible. Yet it's not always possible to guarantee that when
wrapping a C library. The following assumptions are made which might not necessary be true and
thus might introduce unsafety:

 * [`Context`] event loop methods are borrowing `self` immutably considering it immutable (or
   more specifically having interior mutability)

The main objects in this crate are marked as `Send` and it should be indeed be safe to send them
between threads. Yet, no major investigation of the library source code has been performed to
ensure that this is true.


## Initialization and shutdown

You don't need to call the initialization function, it's done automatically when creating a
[`Context`]. Yet you might want to call the [`shutdown()`] function when your application
terminates. Be aware though that the initialization can be called only once in the program
lifetime so you won't be able to use the library properly after you called [`shutdown()`].


## Callbacks

The crate has the ability to store callbacks taking ownership of them so you can pass closures
and not care about storing them externally. There are some things to note about it though. Please
note though that it's not always possible to know whether the underlying library accepted the
callback or not. The crate will keep the closure internally in either case, though it may not ever
be called by the library. You can still remove the callback with the corresponding `*handler_delete()`
or `*handler_clear()` method.

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
let ctx = conn.connect_client(None, None, connection_handler).unwrap();
ctx.run();
libstrophe::shutdown();
```

For more complete examples see this crate `src/examples` directory and [libstrophe examples].


## Crate features

The following features are provided:

  * `rust-log` - enabled by default, makes the create integrate into Rust logging facilities
  * `libstrophe-0_9_3` - enabled by default, enables functionality specific to libstrophe-0.9.3

[libstrophe]: http://strophe.im/libstrophe/
[`log`]: https://crates.io/crates/log
[docs]: http://strophe.im/libstrophe/doc/0.9.2/
[libstrophe examples]: https://github.com/strophe/libstrophe/tree/0.9.2/examples
[`Context`]: https://docs.rs/libstrophe/*/libstrophe/struct.Context.html
[`Connection`]: https://docs.rs/libstrophe/*/libstrophe/struct.Connection.html
[`shutdown()`]: https://docs.rs/libstrophe/*/libstrophe/fn.shutdown.html

License: LGPL-3.0
