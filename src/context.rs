use std::{
	alloc,
	default::Default,
	mem,
	os::raw,
	ptr,
	ptr::NonNull,
	time::Duration,
};

use crate::{
	Connection,
	duration_as_ms,
	FFI,
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
		let memory = Box::new(sys::xmpp_mem_t {
			alloc: Some(Self::custom_alloc),
			free: Some(Self::custom_free),
			realloc: Some(Self::custom_realloc),
			userdata: ptr::null_mut(),
		});
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

	#[inline(always)]
	fn calculate_layout(size: usize) -> alloc::Layout {
		// we leave additional sizeof(usize) bytes in front for the actual allocation size, it's needed later for deallocation
		alloc::Layout::from_size_align(size + mem::size_of_val(&size), mem::align_of_val(&size)).expect("Cannot create layout")
	}

	#[inline(always)]
	fn write_real_alloc(p: *mut u8, size: usize) -> *mut raw::c_void {
		#![allow(clippy::cast_ptr_alignment)]
		// it's ok to cast it as *mut usize because we align to usize during allocation and p points to the beginning of that buffer
		let out = p as *mut usize;
		unsafe {
			out.write(size);
			out.add(1) as _
		}
	}

	#[inline(always)]
	fn read_real_alloc(p: *mut raw::c_void) -> (*mut u8, alloc::Layout) {
		let memory: *mut usize = unsafe { (p as *mut usize).sub(1) };
		let size = unsafe { memory.read() };
		(memory as _, alloc::Layout::from_size_align(size, mem::align_of_val(&size)).expect("Cannot create layout"))
	}

	extern "C" fn custom_alloc(size: usize, _userdata: *mut raw::c_void) -> *mut raw::c_void {
		let layout = Self::calculate_layout(size);
		Self::write_real_alloc(unsafe { alloc::alloc(layout) }, layout.size())
	}

	extern "C" fn custom_free(p: *mut raw::c_void, _userdata: *mut raw::c_void) {
		let (p, layout) = Self::read_real_alloc(p);
		unsafe { alloc::dealloc(p, layout) };
	}

	extern "C" fn custom_realloc(p: *mut raw::c_void, size: usize, _userdata: *mut raw::c_void) -> *mut raw::c_void {
		let new_layout = Self::calculate_layout(size);
		let (p, layout) = Self::read_real_alloc(p);
		Self::write_real_alloc(unsafe { alloc::realloc(p, layout, new_layout.size()) }, new_layout.size())
	}

	/// [xmpp_set_timeout](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#gab03acfbb7c9aa92f60fedb8f6ca43114)
	///
	/// Default timeout is 1000ms
	#[cfg(feature = "libstrophe-0_9_2")]
	pub fn set_timeout(&mut self, timeout: Duration) {
		unsafe {
			sys::xmpp_ctx_set_timeout(self.inner.as_mut(), duration_as_ms(timeout))
		}
	}

	/// [xmpp_run_once](http://strophe.im/libstrophe/doc/0.9.2/group___event_loop.html#ga02816aa5ce34d97fe5bbde5f9c6956ce)
	pub fn run_once(&self, timeout: Duration) {
		unsafe {
			sys::xmpp_run_once(self.inner.as_ptr(), duration_as_ms(timeout))
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

	/// [xmpp_free](https://github.com/strophe/libstrophe/blob/0.9.2/src/ctx.c#L214)
	pub unsafe fn free<T>(&self, p: *mut T) {
		sys::xmpp_free(self.inner.as_ptr(), p as _)
	}

	/// [xmpp_jid_new](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L21)
	pub fn jid_new(&self, node: Option<&str>, domain: impl AsRef<str>, resource: Option<&str>) -> Option<String> {
		let node = FFI(node).send();
		let domain = FFI(domain.as_ref()).send();
		let resource = FFI(resource).send();
		unsafe {
			FFI(sys::xmpp_jid_new(self.inner.as_ptr(), node.as_ptr(), domain.as_ptr(), resource.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_bare](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L65)
	pub fn jid_bare(&self, jid: impl AsRef<str>) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_bare(self.inner.as_ptr(), jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_node](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L87)
	pub fn jid_node(&self, jid: impl AsRef<str>) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_node(self.inner.as_ptr(), jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_domain](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L112)
	pub fn jid_domain(&self, jid: impl AsRef<str>) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_domain(self.inner.as_ptr(), jid.as_ptr())).receive_with_free(|x| self.free(x))
		}
	}

	/// [xmpp_jid_resource](https://github.com/strophe/libstrophe/blob/0.9.2/src/jid.c#L143)
	pub fn jid_resource(&self, jid: impl AsRef<str>) -> Option<String> {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_jid_resource(self.inner.as_ptr(), jid.as_ptr())).receive_with_free(|x| self.free(x))
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
				self.connections.truncate(0);
				sys::xmpp_ctx_free(self.inner.as_mut());
			}
		}
	}
}

unsafe impl Send for Context<'_, '_> {}
