use std::{
	default::Default,
	os::raw,
	ptr::NonNull,
	time::Duration,
};

use crate::{
	AllocContext,
	Connection,
	Logger,
};

/// Proxy to the underlying `xmpp_ctx_t` struct.
///
/// Most of the methods in this struct mimic the methods of the underlying library. So please see
/// libstrophe docs for [context] and [event loop] plus [ctx.c] and [event.c] sources.
/// Only where it's not the case or there is some additional logic involved then you can see the
/// method description.
///
/// This struct implements:
///
///   * `Drop` ([xmpp_ctx_free]).
///   * `Eq` by comparing internal pointers
///   * `Send`
///
/// [context]: http://strophe.im/libstrophe/doc/0.9.2/group___context.html
/// [event loop]: http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html
/// [ctx.c]: https://github.com/strophe/libstrophe/blob/0.9.2/src/ctx.c
/// [event.c]: https://github.com/strophe/libstrophe/blob/0.9.2/src/event.c
/// [xmpp_ctx_free]: http://strophe.im/libstrophe/doc/0.9.2/group___context.html#ga3ae5f04bc23ab2e7b55760759e21d623
#[derive(Debug)]
pub struct Context<'lg, 'cn> {
	inner: NonNull<sys::xmpp_ctx_t>,
	owned: bool,
	connections: Vec<Connection<'cn, 'lg>>,
	logger: Option<Logger<'lg>>,
	memory: Option<Box<sys::xmpp_mem_t>>,
}

impl<'lg, 'cn> Context<'lg, 'cn> {
	/// [xmpp_ctx_new](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#gaeb32490f33760a7ffc0f86a0565b43b2)
	pub fn new(logger: Logger<'lg>) -> Self {
		crate::init();
		let memory = AllocContext::get_xmpp_mem_t();
		unsafe {
			Self::with_inner(
				sys::xmpp_ctx_new(memory.as_ref(), logger.as_inner()),
				true,
				Some(memory),
				Some(logger),
			)
		}
	}

	/// Shortcut to return a new context with default logger.
	///
	/// Equivalent to passing default logger to `Context` constructor.
	pub fn new_with_default_logger() -> Context<'static, 'cn> {
		Context::new(Logger::default())
	}

	/// Shortcut to return a new context with null logger.
	///
	/// Equivalent to passing null logger to `Context` constructor.
	pub fn new_with_null_logger() -> Context<'static, 'cn> {
		Context::new(Logger::new_null())
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_ctx_t, owned: bool, memory: Option<Box<sys::xmpp_mem_t>>, logger: Option<Logger<'lg>>) -> Self {
		if owned && (memory.is_none() || logger.is_none()) {
			panic!("Memory and logger must be supplied for owned Context instances");
		}
		Self {
			inner: NonNull::new(inner).expect("Cannot allocate memory for Context"),
			owned,
			connections: Vec::with_capacity(0),
			memory,
			logger,
		}
	}

	pub(crate) fn consume_connection(&mut self, conn: Connection<'cn, 'lg>) {
		self.connections.push(conn);
	}

	pub unsafe fn from_inner_ref(inner: *const sys::xmpp_ctx_t) -> Self {
		Self::from_inner_ref_mut(inner as _)
	}

	pub unsafe fn from_inner_ref_mut(inner: *mut sys::xmpp_ctx_t) -> Self {
		Self::with_inner(inner, false, None, None)
	}

	pub fn as_inner(&self) -> *mut sys::xmpp_ctx_t { self.inner.as_ptr() }

	/// [xmpp_set_timeout](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#gab03acfbb7c9aa92f60fedb8f6ca43114)
	///
	/// Default timeout is 1000ms
	#[cfg(feature = "libstrophe-0_9_2")]
	pub fn set_timeout(&mut self, timeout: Duration) {
		unsafe {
			sys::xmpp_ctx_set_timeout(self.inner.as_mut(), timeout.as_millis() as raw::c_ulong)
		}
	}

	/// [xmpp_run_once](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga02816aa5ce34d97fe5bbde5f9c6956ce)
	pub fn run_once(&self, timeout: Duration) {
		unsafe {
			sys::xmpp_run_once(self.inner.as_ptr(), timeout.as_millis() as raw::c_ulong)
		}
	}

	/// [xmpp_run](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga14ca97546803cf27c772fa8d2eabfffd)
	pub fn run(&self) {
		unsafe {
			sys::xmpp_run(self.inner.as_ptr())
		}
	}

	/// [xmpp_stop](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga44689e9b7782cec520ed60196e8c15c2)
	pub fn stop(&self) {
		unsafe {
			sys::xmpp_stop(self.inner.as_ptr())
		}
	}
}

impl PartialEq for Context<'_, '_> {
	fn eq(&self, other: &Context) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Context<'_, '_> {}

impl Drop for Context<'_, '_> {
	/// [xmpp_ctx_free](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#ga3ae5f04bc23ab2e7b55760759e21d623)
	fn drop(&mut self) {
		unsafe {
			if self.owned {
				self.connections.clear();
				sys::xmpp_ctx_free(self.inner.as_mut());
			}
		}
	}
}

unsafe impl Send for Context<'_, '_> {}
