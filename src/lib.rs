//! # libstrophe - ergonomic wrapper for Rust
//!
//! This library provides high level ergonomic bindings for [libstrophe], an XMPP client library.
//!
//!
//! # Documentation
//!
//! The documentation for this library covers only Rust specific bits and refers to [original
//! library documentation][docs] in most other cases.
//!
//!
//! # Workflow
//!
//! The general workflow is quite similar to what you get with the C library. The topmost object is
//! [`Context`]. It contains platform-specific bits like logging and memory allocation. Plus an event
//! loop used to keep things going. This crate wraps logging with the facilities provide by [`log`]
//! crate (provided the default `rust-log` feature is enabled). Memory allocation is not yet handled
//! by Rust native means (waiting for allocator API to stabilize). A [`Connection`] is created with a
//! specific [`Context`]. A single [`Context`] can be used for multiple [`Connection`]s because they
//! accept `Arc<Context>` to allow them to share it without requiring you to keep original [`Context`]
//! and handle out references.
//!
//!
//! # Safety
//!
//! This create tries to be as safe as possible. Yet it's not always possible to guarantee that when
//! wrapping a C library. The following assumptions are made which might not necessary be true and
//! thus might introduce unsafety:
//!
//!   * [`Context`] is considered immutable (or more specifically having interior mutability) so its
//!     methods only borrow it immutably
//!
//! The main objects in this crate are marked as `Send` and it should be indeed be safe to send them
//! between threads. Yet, no major investigation of the library source code has been performed to
//! ensure that this is true.
//!
//!
//! # Initialization and shutdown
//!
//! You don't need to call the initialization function, it's done automatically when creating a
//! [`Context`]. Yet you might want to call the [`shutdown()`] function when your application
//! terminates. Be aware though that the initialization can be called only once in the program
//! lifetime so you won't be able to use the library properly after you called [`shutdown()`].
//!
//!
//! # Callbacks
//!
//! Due to the nature of the crate it cannot take ownership of the callbacks that are passed to it.
//! So all callbacks must be owned and stored outside of the library. Also another consequence is
//! that there is no possibility of using `userdata` inside the callbacks so if you need to have
//! a state between callback invocations you must use closures. This is because the crate makes use
//! of `userdata` to pass the actual user callback.
//!
//! With closures you might run into lifetime expressivity problems. Basically adding a callback
//! from another callback cannot be expressed as safe in terms of current Rust. So if you plan on
//! doing so please use corresponding `*_add_unsafe()` handler method. Please also see
//! `src/examples/bot_closure_unsafe.rs` for the example of this.
//!
//!
//! # Examples
//! ```
//! let connection_handler = |ctx: &libstrophe::Context,
//!                           _conn: &mut libstrophe::Connection,
//!                           _evt: libstrophe::ConnectionEvent,
//!                           _error: i32,
//!                           _stream_error: Option<&libstrophe::error::StreamError>| {
//!    ctx.stop();
//! };
//!
//! let ctx = libstrophe::Context::new_with_default_logger();
//! let mut conn = libstrophe::Connection::new(ctx);
//! conn.set_jid("example@127.0.0.1");
//! conn.set_pass("password");
//! let ctx = conn.connect_client(None, None, connection_handler).unwrap();
//! ctx.run();
//! libstrophe::shutdown();
//! ```
//!
//! For more complete examples see this crate `src/examples` directory and [libstrophe examples].
//!
//!
//! # Crate features
//!
//! The following features are provided:
//!
//!   * `rust-log` - enabled by default, makes the create integrate into Rust logging facilities
//!   * `libstrophe-0_9_2` - enabled by default, enables functionality specific to libstrophe-0.9.2
//!   * `fail-tests` - development feature, enables some additional tests that must fail unless
//!                    safety contracts are broken
//!
//! [libstrophe]: http://strophe.im/libstrophe/
//! [`log`]: https://crates.io/crates/log
//! [docs]: http://strophe.im/libstrophe/doc/0.9.2/
//! [libstrophe examples]: https://github.com/strophe/libstrophe/tree/0.9.2/examples
//! [`Context`]: https://docs.rs/libstrophe/*/libstrophe/struct.Context.html
//! [`Connection`]: https://docs.rs/libstrophe/*/libstrophe/struct.Connection.html
//! [`shutdown()`]: https://docs.rs/libstrophe/*/libstrophe/fn.shutdown.html

#[macro_use]
extern crate failure_derive;

use std::{sync, time};
use std::os::raw;

use bitflags::bitflags;

pub use sys::{
	xmpp_conn_event_t as ConnectionEvent,
	xmpp_log_level_t as LogLevel,
};

pub use self::connection::{Connection, HandlerId, IdHandlerId, TimedHandlerId};
pub use self::context::Context;
use self::ffi_types::FFI;
pub use self::logger::Logger;
pub use self::stanza::{Stanza, StanzaMutRef, StanzaRef};

mod ffi_types;
mod connection;
mod context;
pub mod error;
mod logger;
mod stanza;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod examples;

bitflags! {
	pub struct ConnectionFlags: raw::c_long {
		const DISABLE_TLS = sys::XMPP_CONN_FLAG_DISABLE_TLS as raw::c_long;
		const MANDATORY_TLS = sys::XMPP_CONN_FLAG_MANDATORY_TLS as raw::c_long;
		const LEGACY_SSL = sys::XMPP_CONN_FLAG_LEGACY_SSL as raw::c_long;
		#[cfg(feature = "libstrophe-0_9_2")]
		const TRUST_TLS = sys::XMPP_CONN_FLAG_TRUST_TLS as raw::c_long;
	}
}

static INIT: sync::Once = sync::ONCE_INIT;
static DEINIT: sync::Once = sync::ONCE_INIT;

/// Convert `Duration` to milliseconds
#[inline]
fn duration_as_ms(duration: time::Duration) -> raw::c_ulong {
	(duration.as_secs() * 1_000 + u64::from(duration.subsec_millis())) as raw::c_ulong
}

/// Convert type to *void for passing as `userdata`
fn as_void_ptr<T>(cb: &T) -> *mut raw::c_void {
	cb as *const _ as _
}

/// Convert *void from `userdata` to appropriate type
unsafe fn void_ptr_as<'cb, T>(ptr: *const raw::c_void) -> &'cb mut T {
	(ptr as *mut T).as_mut().expect("userdata must be non-null")
}

/// Ensure that underlying C library is initialized
///
/// Must be called from every possible crate usage entry point.
fn init() {
	INIT.call_once(|| {
		unsafe {
			sys::xmpp_initialize();
		}
	});
}

fn deinit() {
	DEINIT.call_once(|| {
		unsafe {
			sys::xmpp_shutdown()
		}
	});
}

/// [xmpp_version_check](http://strophe.im/libstrophe/doc/0.9.2/group___init.html#ga6cc7afca422acce51e0e7f52424f1db3)
pub fn version_check(major: i32, minor: i32) -> bool {
	unsafe {
		FFI(sys::xmpp_version_check(major, minor)).receive_bool()
	}
}

/// [xmpp_shutdown](http://strophe.im/libstrophe/doc/0.9.2/group___init.html#ga06e07524aee531de1ceb825541307963)
///
/// Call this function when your application terminates, but be aware that you can't use the library
/// after you called `shutdown()` and there is now way to reinitialize it again.
///
/// This function is thread safe, it's safe to call it several times and it's safe to call it before
/// doing any initialization.
pub fn shutdown() {
	init();
	deinit();
}
