# libstrophe

## Documentation

See [full documentation](https://docs.rs/libstrophe)

## Usage

Add this to your Cargo.toml:
```
[dependencies]
libstrophe = "0.8"
```

## libstrophe - ergonomic wrapper for Rust

This library provides high level ergonomic bindings for [libstrophe], an XMPP client library.


## Documentation

The documentation for this library covers only Rust specific bits and refers to [original
library documentation][docs] in most other cases.


## Workflow

The general workflow is quite similar to what you get with the C library. The topmost object is
[`Context`]. It contains platform-specific bits like logging and memory allocation. Plus an event
loop used to keep things going. This crate wraps logging with the facilities provide by [`log`]
crate (provided the default `rust-log` feature is enabled). Memory allocation is not yet handled
by Rust native means (waiting for allocator API to stabilize). A [`Connection`] is created with a
specific [`Context`]. A single [`Context`] can be used for multiple [`Connection`]s because they
accept `Arc<Context>` to allow them to share it without requiring you to keep original [`Context`]
and handle out references.


## Safety

This create tries to be as safe as possible. Yet it's not always possible to guarantee that when
wrapping a C library. The following assumptions are made which might not necessary be true and
thus might introduce unsafety:

  * [`Context`] is considered immutable (or more specifically having interior mutability) so its
    methods only borrow it immutably

The main objects in this crate are marked as `Send` and it should be indeed be safe to send them
between threads. Yet, no major investigation of the library source code has been performed to
ensure that this is true.


## Initialization and shutdown

You don't need to call the initialization function, it's done automatically when creating a
[`Context`]. Yet you might want to call the [`shutdown()`] function when your application
terminates. Be aware though that the initialization can be called only once in the program
lifetime so you won't be able to use the library properly after you called [`shutdown()`].


## Callbacks

Due to the nature of the crate it cannot take ownership of the callbacks that are passed to it.
So all callbacks must be owned and stored outside of the library. Also another consequence is
that there is no possibility of using `userdata` inside the callbacks so if you need to have
a state between callback invocations you must use closures. This is because the crate makes use
of `userdata` to pass the actual user callback.

With closures you might run into lifetime expressivity problems. Basically adding a callback
from another callback cannot be expressed as safe in terms of current Rust. So if you plan on
doing so please use corresponding `*_add_unsafe()` handler method. Please also see
`src/examples/bot_closure_unsafe.rs` for the example of this.


## Examples
```rust
let connection_handler = |conn: &mut libstrophe::Connection,
                          _evt: libstrophe::ConnectionEvent,
                          _error: i32,
                          _stream_error: Option<&libstrophe::error::StreamError>| {
   conn.context().stop();
};

let ctx = libstrophe::Context::new_with_default_logger();
let mut conn = libstrophe::Connection::new(ctx);
conn.set_jid("example@127.0.0.1");
conn.set_pass("password");
conn.connect_client(None, None, &connection_handler).unwrap();
// ctx.run();
libstrophe::shutdown();
```

For more complete examples see this crate `src/examples` directory and [libstrophe examples].


## Crate features

The following features are provided:

  * `rust-log` - enabled by default, makes the create integrate into Rust logging facilities
  * `fail-tests` - development feature, enables some additional tests that must fail unless
                   safety contracts are broken

[libstrophe]: http://strophe.im/libstrophe/
[`log`]: https://crates.io/crates/log
[docs]: http://strophe.im/libstrophe/doc/0.8-snapshot/
[libstrophe examples]: https://github.com/strophe/libstrophe/tree/0.9.1/examples
[`Context`]: https://docs.rs/libstrophe/*/libstrophe/struct.Context.html
[`Connection`]: https://docs.rs/libstrophe/*/libstrophe/struct.Connection.html
[`shutdown()`]: https://docs.rs/libstrophe/*/libstrophe/fn.shutdown.html

License: LGPL-3.0
