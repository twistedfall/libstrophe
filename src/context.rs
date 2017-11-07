use std::{ops, ptr, time};
use std::default::Default;
use std::os::raw;
use std::sync::Arc;

use super::{sys, Logger};

/// Proxy to underlying `xmpp_ctx_t` struct.
///
/// Most of the methods in this struct mimic the methods of the underlying library. So please see
/// libstrophe outdated docs for [context] and [event loop] plus [ctx.c] and [event.c] sources.
/// Only where it's not the case or there is some additional logic involved then you can see the
/// method description.
///
/// This struct implements:
///   * `Drop` ([xmpp_ctx_free]).
///   * `Eq` by comparing internal pointers
///   * `Send`
///
/// [context]: http://strophe.im/libstrophe/doc/0.8-snapshot/group___context.html
/// [event loop]: http://strophe.im/libstrophe/doc/0.8-snapshot/group___event_loop.html
/// [ctx.c]: https://github.com/strophe/libstrophe/blob/0.9.1/src/ctx.c#L16
/// [event.c]: https://github.com/strophe/libstrophe/blob/0.9.1/src/event.c#L16
/// [xmpp_ctx_free]: https://github.com/strophe/libstrophe/blob/0.9.1/src/ctx.c#L422
#[derive(Debug, Hash)]
pub struct Context<'lg> {
	inner: *mut sys::xmpp_ctx_t,
	owned: bool,
	_logger: Option<Logger<'lg>>,
}

impl<'lg> Context<'lg> {
	/// [xmpp_ctx_new](https://github.com/strophe/libstrophe/blob/0.9.1/src/ctx.c#L375)
	pub fn new(logger: Logger<'lg>) -> Context<'lg> {
		super::init();
		unsafe {
			Context::with_inner(
				sys::xmpp_ctx_new(ptr::null(), logger.as_inner()),
				true,
				Some(logger)
			)
		}
	}

	/// Shortcut to return a new context with default logger.
	///
	/// Equivalent to passing default logger to `Context` constructor. The result is also wrapped in
	/// `Arc` to allow multiple ownership.
	pub fn new_with_default_logger() -> Arc<Context<'static>> {
		Arc::new(Context::new(Logger::default()))
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_ctx_t, owned: bool, logger: Option<Logger<'lg>>) -> Context<'lg> {
		if inner.is_null() {
			panic!("Cannot allocate memory for Context")
		}
		Context { inner, owned, _logger: logger }
	}

	pub unsafe fn from_inner_ref(inner: *const sys::xmpp_ctx_t) -> Context<'lg> {
		Context::from_inner_ref_mut(inner as *mut _)
	}

	pub unsafe fn from_inner_ref_mut(inner: *mut sys::xmpp_ctx_t) -> Context<'lg> {
		Context::with_inner(inner, false, None)
	}

	pub fn as_inner(&self) -> *const sys::xmpp_ctx_t { self.inner }

	/// [xmpp_run_once](https://github.com/strophe/libstrophe/blob/0.9.1/src/event.c#L62)
	pub fn run_once(&self, timeout: time::Duration) {
		unsafe {
			sys::xmpp_run_once(self.inner, super::duration_as_ms(timeout))
		}
	}

	/// [xmpp_run](https://github.com/strophe/libstrophe/blob/0.9.1/src/event.c#L322)
	pub fn run(&self) {
		unsafe {
			sys::xmpp_run(self.inner)
		}
	}

	/// [xmpp_stop](https://github.com/strophe/libstrophe/blob/0.9.1/src/event.c#L345)
	pub fn stop(&self) {
		unsafe {
			sys::xmpp_stop(self.inner)
		}
	}

	/// [xmpp_free](https://github.com/strophe/libstrophe/blob/0.9.1/src/ctx.c#L207)
	pub unsafe fn free<T>(&self, p: *mut T) {
		sys::xmpp_free(self.inner, p as *mut raw::c_void)
	}
}

impl<'lg> PartialEq for Context<'lg> {
	fn eq(&self, other: &Context) -> bool {
		self.inner == other.inner
	}
}

impl<'lg> Eq for Context<'lg> {}

impl<'lg> Drop for Context<'lg> {
	/// [xmpp_ctx_free](https://github.com/strophe/libstrophe/blob/0.9.1/src/ctx.c#L422)
	fn drop(&mut self) {
		unsafe {
			if self.owned {
				sys::xmpp_ctx_free(self.inner);
			}
		}
	}
}

unsafe impl<'lg> Send for Context<'lg> {}

/// Wrapper for constant ref to `Arc<Context>`, implements deref to `Arc<Context>`.
///
/// You can obtain such object by calling `context()` method of `Connection`.
#[derive(Debug, Hash, PartialEq)]
pub struct ContextRef<'lg>(Arc<Context<'lg>>);

impl<'lg> ops::Deref for ContextRef<'lg> {
	type Target = Context<'lg>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'lg> Into<ContextRef<'lg>> for Arc<Context<'lg>> {
	fn into(self) -> ContextRef<'lg> {
		ContextRef(self)
	}
}
