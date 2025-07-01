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
//! [Context]. It contains platform-specific bits like logging and memory allocation. Plus an event
//! loop used to keep things going. This crate wraps logging with the facilities provided by [`log`]
//! crate (provided the default `rust-log` feature is enabled). Memory allocation is also handled by
//! Rust native means. When a [Connection] is created it will temporarily consume the [Context].
//! After all the setup is done, call one of the `connect_*()` methods to retrieve the [Context]
//! back. In this manner a single [Context] can be used for multiple [Connection]s consequently.
//! When you're done with setting up [Connection]s for the [Context], use `run()` or `run_once()`
//! methods to start the event loop rolling.
//!
//!
//! # Safety
//!
//! This crate tries to be as safe as possible. Yet it's not always possible to guarantee that when
//! wrapping a C library. The following assumptions are made which might not necessarily be true and
//! thus might introduce unsafety:
//!
//!  * [Context] event loop methods are borrowing `self` immutably considering it immutable (or
//!    more specifically having interior mutability)
//!
//! The main objects in this crate are marked as `Send` and it should be indeed be safe to send them
//! between threads. Yet, no major investigation of the library source code has been performed to
//! ensure that this is true.
//!
//!
//! # Initialization and shutdown
//!
//! You don't need to call the initialization function, it's done automatically when creating a
//! [Context]. Yet you might want to call the [shutdown()] function when your application
//! terminates. Be aware though that the initialization can be called only once in the program
//! lifetime so you won't be able to use the library properly after you called [shutdown()].
//!
//!
//! # Callbacks
//!
//! The crate has the ability to store callbacks taking ownership of them so you can pass closures
//! and not care about storing them externally. There are some things to note about it though. It's
//! not always possible to know whether the underlying library accepted the callback. The crate will
//! keep the closure internally in either case, though it may not ever be called by the library. You
//! can still remove the callback with the corresponding `*handler_delete()` or `*handler_clear()`
//! method.
//!
//! Due to the way the C libstrophe library is implemented and how Rust optimizes monomorphization,
//! your callbacks must actually be compiled to different function with separate addresses when you
//! pass them to the same handler setup method. So if you want to pass 2 callbacks `hander_add`
//! ensure that their code is unique and rust didn't merge them into a single function behind the
//! scenes. You can test whether 2 callbacks are same or not with the `Connection::*handlers_same()`
//! family of functions. If it returns true then you will only be able to pass one of them to the
//! corresponding handler function, the other will be silently ignored.
//!
//! Due to the fact that the crate uses `userdata` to pass the actual user callback, it's not possible
//! to use `userdata` inside the callbacks for your own data. So if you need to have a state between
//! callback invocations you must use closures.
//!
//! Because the main objects are marked as `Send` and we store callbacks inside them, all callbacks
//! must also be `Send`.
//!
//!
//! # Examples
//! ```
//! let connection_handler = |ctx: &libstrophe::Context,
//!                           _conn: &mut libstrophe::Connection,
//!                           _evt: libstrophe::ConnectionEvent| {
//!    ctx.stop();
//! };
//!
//! let ctx = libstrophe::Context::new_with_default_logger();
//! let mut conn = libstrophe::Connection::new(ctx);
//! conn.set_jid("example@127.0.0.1");
//! conn.set_pass("password");
//! let mut ctx = conn.connect_client(None, None, connection_handler).unwrap();
//! ctx.run();
//! libstrophe::shutdown();
//! ```
//!
//! For more complete examples see this crate `examples` directory and [libstrophe examples].
//!
//!
//! # Crate features
//!
//! The following features are provided:
//!
//!   * `rust-log` - enabled by default, makes the crate integrate into Rust logging facilities
//!   * `libstrophe-0_9_3` - enabled by default, enables functions specific to libstrophe-0.9.3
//!   * `libstrophe-0_10_0` - enabled by default, enables functions specific to libstrophe-0.10
//!   * `libstrophe-0_11_0` - enabled by default, enables functions specific to libstrophe-0.11
//!   * `libstrophe-0_12_0` - enabled by default, enables functions specific to libstrophe-0.12
//!   * `libstrophe-0_13` - enabled by default, enables functions specific to libstrophe-0.13
//!   * `libstrophe-0_14` - enabled by default, enables functions specific to libstrophe-0.14
//!   * `buildtime_bindgen` - forces regeneration of the bindings instead of relying on the
//!     pre-generated sources
//!
//! [libstrophe]: https://strophe.im/libstrophe/
//! [`log`]: https://crates.io/crates/log
//! [docs]: https://strophe.im/libstrophe/doc/0.13.0/
//! [libstrophe examples]: https://github.com/strophe/libstrophe/tree/0.12.2/examples

use core::ffi::{c_long, c_void};
use std::sync::Once;

pub use alloc_context::AllocContext;
use bitflags::bitflags;
#[cfg(feature = "libstrophe-0_11_0")]
pub use connection::CertFailResult;
#[cfg(feature = "libstrophe-0_12_0")]
pub use connection::SockoptResult;
pub use connection::{Connection, ConnectionEvent, HandlerId, HandlerResult, IdHandlerId, TimedHandlerId};
pub use context::Context;
pub use error::{
	ConnectClientError, ConnectionError, Error, OwnedConnectionError, OwnedStreamError, Result, StreamError, ToTextError,
};
use ffi_types::FFI;
pub use logger::Logger;
use once_cell::sync::Lazy;
#[cfg(feature = "libstrophe-0_12_0")]
pub use sm_state::SMState;
#[cfg(feature = "libstrophe-0_14")]
pub use sm_state::{SerializedSmState, SerializedSmStateRef};
pub use stanza::{Stanza, StanzaMutRef, StanzaRef, XMPP_STANZA_NAME_IN_NS};
#[cfg(feature = "libstrophe-0_11_0")]
pub use sys::xmpp_cert_element_t as CertElement;
#[cfg(feature = "libstrophe-0_9_3")]
pub use sys::xmpp_error_type_t as ErrorType;
pub use sys::xmpp_log_level_t as LogLevel;
#[cfg(feature = "libstrophe-0_12_0")]
pub use sys::xmpp_queue_element_t as QueueElement;
#[cfg(feature = "libstrophe-0_11_0")]
pub use tls_cert::TlsCert;

mod alloc_context;
mod connection;
mod context;
mod error;
mod ffi_types;
pub mod jid;
mod logger;
#[cfg(feature = "libstrophe-0_12_0")]
mod sm_state;
mod stanza;
#[cfg(feature = "libstrophe-0_11_0")]
mod tls_cert;

bitflags! {
	pub struct ConnectionFlags: c_long {
		const DISABLE_TLS = sys::XMPP_CONN_FLAG_DISABLE_TLS;
		const MANDATORY_TLS = sys::XMPP_CONN_FLAG_MANDATORY_TLS;
		const LEGACY_SSL = sys::XMPP_CONN_FLAG_LEGACY_SSL;
		const TRUST_TLS = sys::XMPP_CONN_FLAG_TRUST_TLS;
		#[cfg(feature = "libstrophe-0_9_3")]
		const LEGACY_AUTH = sys::XMPP_CONN_FLAG_LEGACY_AUTH;
		#[cfg(feature = "libstrophe-0_12_0")]
		const DISABLE_SM = sys::XMPP_CONN_FLAG_DISABLE_SM;
		#[cfg(feature = "libstrophe-0_13")]
		const ENABLE_COMPRESSION = sys::XMPP_CONN_FLAG_ENABLE_COMPRESSION;
		#[cfg(feature = "libstrophe-0_13")]
		const COMPRESSION_DONT_RESET = sys::XMPP_CONN_FLAG_COMPRESSION_DONT_RESET;
	}
}

static ALLOC_CONTEXT: Lazy<AllocContext> = Lazy::new(AllocContext::default);

/// Convert type to `void*` for passing as `userdata`
fn as_void_ptr<T>(cb: &mut T) -> *mut c_void {
	(cb as *mut T).cast::<c_void>()
}

/// Convert `void*` from `userdata` to the appropriate type
unsafe fn void_ptr_as<'cb, T>(ptr: *mut c_void) -> &'cb mut T {
	unsafe { ptr.cast::<T>().as_mut() }.expect("userdata must be non-null")
}

/// Ensure that underlying C library is initialized
///
/// Must be called from every possible crate usage entry point.
fn init() {
	static INIT: Once = Once::new();
	INIT.call_once(|| unsafe {
		sys::xmpp_initialize();
	});
}

fn deinit() {
	static DEINIT: Once = Once::new();
	DEINIT.call_once(|| unsafe { sys::xmpp_shutdown() });
}

/// [xmpp_version_check](https://strophe.im/libstrophe/doc/0.13.0/group___init.html#ga6cc7afca422acce51e0e7f52424f1db3)
pub fn version_check(major: i32, minor: i32) -> bool {
	unsafe { FFI(sys::xmpp_version_check(major, minor)).receive_bool() }
}

/// [xmpp_shutdown](https://strophe.im/libstrophe/doc/0.13.0/group___init.html#ga06e07524aee531de1ceb825541307963)
///
/// Call this function when your application terminates, but be aware that you can't use the library
/// after you called `shutdown()` and there is now a way to reinitialize it again.
///
/// This function is thread safe, it's safe to call it several times, and it's safe to call it before
/// doing any initialization.
pub fn shutdown() {
	init();
	deinit();
}
