use std::marker;
use std::os::raw;

use super::{
	sys,
	as_udata,
	udata_as,
	LogLevel,
};
use super::ffi_types::FFI;

/// Wrapper around underlying `xmpp_log_t` struct.
///
/// The best option to get a logger is to call `Logger::default()`. It will return you a logger that
/// is tied into Rust logging facility provided by `log` crate. This functionality is available when
/// compiling with the default `rust-log` feature.
///
/// This struct implements:
///
///   * `Eq` by comparing internal pointers
///   * `Send`.
#[derive(Debug, Hash)]
pub struct Logger<'cb> {
	inner: *mut sys::xmpp_log_t,
	owned: bool,
	_callbacks: marker::PhantomData<&'cb fn()>,

}

impl<'cb> Logger<'cb> {
	/// Create a new custom logger.
	///
	/// The callback argument will be called every time a log message needs to be printed.
	pub fn new<CB>(handler: &'cb CB) -> Logger<'cb>
		where
			CB: FnMut(LogLevel, &str, &str) + 'cb,
	{
		Logger::with_inner(Box::into_raw(Box::new(sys::xmpp_log_t {
			handler: Some(Self::log_handler_cb::<CB>),
			userdata: as_udata(handler) as *mut _,
		})), true)
	}

	#[inline]
	fn with_inner(inner: *mut sys::xmpp_log_t, owned: bool) -> Logger<'cb> {
		if inner.is_null() {
			panic!("Cannot allocate memory for Logger")
		}
		Logger { inner, owned, _callbacks: marker::PhantomData }
	}

	/// [xmpp_get_default_logger](https://github.com/strophe/libstrophe/blob/0.9.1/src/ctx.c#L170)
	///
	/// This method returns default `libstrophe` logger that just outputs log lines to stderr. Use it
	/// if you compile without `rust-log` feature and want a quick debug log output.
	pub fn new_internal(log_level: LogLevel) -> Logger<'static> {
		Logger::with_inner(unsafe { sys::xmpp_get_default_logger(log_level) }, false)
	}

	extern "C" fn log_handler_cb<CB>(userdata: *const raw::c_void, level: sys::xmpp_log_level_t, area: *const raw::c_char, msg: *const raw::c_char)
		where
			CB: FnMut(LogLevel, &str, &str) + 'cb,
	{
		let area = unsafe { FFI(area).receive() }.unwrap();
		let msg = unsafe { FFI(msg).receive() }.unwrap();
		unsafe {
			udata_as::<CB>(userdata)(level, &area, &msg);
		}
	}

	pub fn as_inner(&self) -> *const sys::xmpp_log_t {
		self.inner
	}
}

#[cfg(feature = "rust-log")]
fn log_handler(log_level: LogLevel, area: &str, message: &str) {
	match log_level {
		LogLevel::XMPP_LEVEL_DEBUG => debug!("{}: {}", area, message),
		LogLevel::XMPP_LEVEL_INFO => info!("{}: {}", area, message),
		LogLevel::XMPP_LEVEL_WARN => warn!("{}: {}", area, message),
		LogLevel::XMPP_LEVEL_ERROR => error!("{}: {}", area, message),
	}
}

impl Default for Logger<'static> {
	/// Return a new logger that logs to standard Rust logging facilities.
	///
	/// Logging facilities are provided by `log` crate. Only available when compiling with `rust-log`
	/// feature.
	#[cfg(feature = "rust-log")]
	fn default() -> Self {
		// this trick allows us to get the real pointer to global function and not reference to
		// the temporary local var (as in case with &log_handler)
		let log_handler_ptr = Box::into_raw(Box::new(log_handler));
		let out = Logger::new(unsafe { log_handler_ptr.as_mut().unwrap() });
		out
	}

	/// Create a new default logger by calling `new_internal()` with debug log level.
	///
	/// Used when the crate is compiled without `rust-log` feature.
	#[cfg(not(feature = "rust-log"))]
	fn default() -> Self {
		Logger::new_internal(LogLevel::XMPP_LEVEL_DEBUG)
	}
}

impl<'cb> PartialEq for Logger<'cb> {
	fn eq(&self, other: &Logger) -> bool {
		self.inner == other.inner
	}
}

impl<'cb> Eq for Logger<'cb> {}

impl<'cb> Drop for Logger<'cb> {
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				Box::from_raw(self.inner);
			}
		}
	}
}

unsafe impl<'cb> Send for Logger<'cb> {}
