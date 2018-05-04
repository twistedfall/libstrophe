use std::{ops, ptr, time};
use std::default::Default;
use std::os::raw;
use std::sync::Arc;
use super::{FFI, Logger, sys};

/// Proxy to underlying `xmpp_ctx_t` struct.
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
#[derive(Debug, Hash)]
pub struct Context<'lg> {
	inner: *mut sys::xmpp_ctx_t,
	owned: bool,
	_logger: Option<Logger<'lg>>,
}

impl<'lg> Context<'lg> {
	/// [xmpp_ctx_new](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#gaeb32490f33760a7ffc0f86a0565b43b2)
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

	/// [xmpp_set_timeout](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#gab03acfbb7c9aa92f60fedb8f6ca43114)
	#[cfg(feature = "libstrophe-0_9_2")]
	pub fn set_timeout(&self, timeout: time::Duration) {
		unsafe  {
			sys::xmpp_ctx_set_timeout(self.inner, super::duration_as_ms(timeout))
		}
	}

	/// [xmpp_run_once](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga02816aa5ce34d97fe5bbde5f9c6956ce)
	pub fn run_once(&self, timeout: time::Duration) {
		unsafe {
			sys::xmpp_run_once(self.inner, super::duration_as_ms(timeout))
		}
	}

	/// [xmpp_run](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga14ca97546803cf27c772fa8d2eabfffd)
	pub fn run(&self) {
		unsafe {
			sys::xmpp_run(self.inner)
		}
	}

	/// [xmpp_stop](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga44689e9b7782cec520ed60196e8c15c2)
	pub fn stop(&self) {
		unsafe {
			sys::xmpp_stop(self.inner)
		}
	}

	/// [xmpp_free](https://github.com/strophe/libstrophe/blob/0.9.2/src/ctx.c#L214)
	pub unsafe fn free<T>(&self, p: *mut T) {
		sys::xmpp_free(self.inner, p as *mut raw::c_void)
	}

	/// [xmpp_jid_new](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L21)
	pub fn jid_new<RefStr: AsRef<str>>(&self, node: Option<&str>, domain: RefStr, resource: Option<&str>) -> Option<String>
	{
		let node = FFI(node).send();
		let domain = FFI(domain.as_ref()).send();
		let resource = FFI(resource).send();
		unsafe {
			FFI(sys::xmpp_jid_new(self.inner, node.as_ptr(), domain.as_ptr(), resource.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_bare](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L65)
	pub fn jid_bare<RefStr: AsRef<str>>(&self, jid: RefStr) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_bare(self.inner, jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_node](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L87)
	pub fn jid_node<RefStr: AsRef<str>>(&self, jid: RefStr) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_node(self.inner, jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_domain](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L112)
	pub fn jid_domain<RefStr: AsRef<str>>(&self, jid: RefStr) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_domain(self.inner, jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_resource](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L143)
	pub fn jid_resource<RefStr: AsRef<str>>(&self, jid: RefStr) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_resource(self.inner, jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}
}

impl<'lg> PartialEq for Context<'lg> {
	fn eq(&self, other: &Context) -> bool {
		self.inner == other.inner
	}
}

impl<'lg> Eq for Context<'lg> {}

impl<'lg> Drop for Context<'lg> {
	/// [xmpp_ctx_free](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#ga3ae5f04bc23ab2e7b55760759e21d623)
	fn drop(&mut self) {
		unsafe {
			if self.owned {
				sys::xmpp_ctx_free(self.inner);
			}
		}
	}
}

unsafe impl<'lg> Send for Context<'lg> {}

/// Wrapper for constant ref to `Arc<Context>`, implements `Deref` to `Arc<Context>`.
///
/// You can obtain such object by calling e.g. [`context()`] method of [`Connection`].
///
/// [`context()`]: struct.Connection.html#method.context
/// [`Connection`]: struct.Connection.html
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
