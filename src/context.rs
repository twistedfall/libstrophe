use std::{
	default::Default,
	os::raw,
	ptr::NonNull,
	time::Duration,
};

use crate::{
	AllocContext,
	Connection,
	FFI,
	Logger,
	LogLevel,
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
/// [context]: https://strophe.im/libstrophe/doc/0.10.0/group___context.html
/// [event loop]: https://strophe.im/libstrophe/doc/0.10.0/group___event_loop.html
/// [ctx.c]: https://github.com/strophe/libstrophe/blob/0.10.0/src/ctx.c
/// [event.c]: https://github.com/strophe/libstrophe/blob/0.10.0/src/event.c
/// [xmpp_ctx_free]: https://strophe.im/libstrophe/doc/0.10.0/group___context.html#ga3ae5f04bc23ab2e7b55760759e21d623
#[derive(Debug)]
pub struct Context<'lg, 'cn> {
	inner: NonNull<sys::xmpp_ctx_t>,
	owned: bool,
	connections: Vec<Connection<'cn, 'lg>>,
	_logger: Option<Logger<'lg>>,
	_memory: Option<Box<sys::xmpp_mem_t>>,
}

impl<'lg, 'cn> Context<'lg, 'cn> {
	/// [xmpp_ctx_new](https://strophe.im/libstrophe/doc/0.10.0/group___context.html#gaeb32490f33760a7ffc0f86a0565b43b2)
	pub fn new(logger: Logger<'lg>) -> Self {
		crate::init();
		let memory = Box::new(AllocContext::get_xmpp_mem_t());
		unsafe {
			Self::with_inner(
				sys::xmpp_ctx_new(memory.as_ref(), logger.as_ptr()),
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
			_memory: memory,
			_logger: logger,
		}
	}

	pub(crate) fn consume_connection(&mut self, conn: Connection<'cn, 'lg>) {
		self.connections.push(conn);
	}

	/// # Safety
	/// inner must be a valid pointer to a previously allocated xmp_ctx_t and you must make sure that
	/// Self doesn't outlive the context behind that pointer
	pub unsafe fn from_ref(inner: *const sys::xmpp_ctx_t) -> Self {
		Self::from_ref_mut(inner as _)
	}

	/// # Safety
	/// inner must be a valid pointer to a previously allocated mutable xmp_ctx_t and you must make
	/// sure that Self doesn't outlive the context behind that pointer
	pub unsafe fn from_ref_mut(inner: *mut sys::xmpp_ctx_t) -> Self {
		Self::with_inner(inner, false, None, None)
	}

	pub(crate) fn as_ptr(&self) -> *mut sys::xmpp_ctx_t { self.inner.as_ptr() }

	/// [xmpp_set_timeout](https://strophe.im/libstrophe/doc/0.10.0/group___context.html#gab03acfbb7c9aa92f60fedb8f6ca43114)
	///
	/// Default timeout is 1000ms
	pub fn set_timeout(&mut self, timeout: Duration) {
		unsafe {
			sys::xmpp_ctx_set_timeout(self.inner.as_mut(), timeout.as_millis() as raw::c_ulong)
		}
	}

	// todo: add global_timed_handler support

	/// [xmpp_run_once](https://strophe.im/libstrophe/doc/0.10.0/group___event_loop.html#ga02816aa5ce34d97fe5bbde5f9c6956ce)
	pub fn run_once(&self, timeout: Duration) {
		unsafe {
			sys::xmpp_run_once(self.inner.as_ptr(), timeout.as_millis() as raw::c_ulong)
		}
	}

	/// [xmpp_run](https://strophe.im/libstrophe/doc/0.10.0/group___event_loop.html#ga14ca97546803cf27c772fa8d2eabfffd)
	pub fn run(&self) {
		unsafe {
			sys::xmpp_run(self.inner.as_ptr())
		}
	}

	/// [xmpp_stop](https://strophe.im/libstrophe/doc/0.10.0/group___event_loop.html#ga44689e9b7782cec520ed60196e8c15c2)
	pub fn stop(&self) {
		unsafe {
			sys::xmpp_stop(self.inner.as_ptr())
		}
	}

	pub fn log(&self, level: LogLevel, area: &str, msg: &str) {
		unsafe { ctx_log(self.inner.as_ptr(), level, area, msg) }
	}
}

impl PartialEq for Context<'_, '_> {
	fn eq(&self, other: &Context) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Context<'_, '_> {}

impl Drop for Context<'_, '_> {
	/// [xmpp_ctx_free](https://strophe.im/libstrophe/doc/0.10.0/group___context.html#ga3ae5f04bc23ab2e7b55760759e21d623)
	fn drop(&mut self) {
		if self.owned {
			self.connections.clear();
			unsafe {
				sys::xmpp_ctx_free(self.inner.as_mut());
			}
		}
	}
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Context<'_, '_> {}

pub(crate) unsafe fn ctx_log(ctx: *const sys::xmpp_ctx_t, level: sys::xmpp_log_level_t, area: &str, msg: &str) {
	#[allow(non_camel_case_types)]
	#[repr(C)]
	struct _xmpp_ctx_t {
		mem: *const sys::xmpp_mem_t,
		log: *const sys::xmpp_log_t,
		// ...
	}
	let inner = (ctx as *mut _xmpp_ctx_t).as_ref().expect("Null pointer for Context");
	if let Some(log) = inner.log.as_ref() {
		if let Some(log_handler) = log.handler {
			let area = FFI(area).send();
			let msg = FFI(msg).send();
			log_handler(log.userdata, level, area.as_ptr(), msg.as_ptr());
		}
	}
}
