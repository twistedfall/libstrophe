use std::{fmt, hash, os::raw};
use super::{
	as_udata,
	LogLevel,
	sys,
	udata_as,
};
use super::ffi_types::FFI;

type LogHandler<'cb> = dyn FnMut(LogLevel, &str, &str) + 'cb;

/// Wrapper around the underlying `xmpp_log_t` struct.
///
/// The best option to get a logger is to call [`Logger::default()`]. It will return you a logger that
/// is tied into Rust logging facility provided by [`log`] crate. This functionality is available when
/// compiling with the default `rust-log` feature.
///
/// This struct implements:
///
///   * `Eq` by comparing internal pointers
///   * `Send`
///
/// [`Logger::default()`]: struct.Logger.html#method.default
/// [`log`]: https://crates.io/crates/log
pub struct Logger<'cb> {
	inner: *mut sys::xmpp_log_t,
	owned: bool,
	handler: Box<LogHandler<'cb>>,
}

impl<'cb> Logger<'cb> {
	/// Create a new custom logger.
	///
	/// The callback argument will be called every time a log message needs to be printed.
	pub fn new<CB>(handler: CB) -> Logger<'cb>
		where
			CB: FnMut(LogLevel, &str, &str) + 'cb,
	{
		let handler = Box::new(handler);
		Logger::with_inner(Box::into_raw(Box::new(sys::xmpp_log_t {
			handler: Some(Self::log_handler_cb::<CB>),
			userdata: as_udata(&*handler) as *mut _,
		})), handler, true)
	}

	#[inline]
	fn with_inner(inner: *mut sys::xmpp_log_t, handler: Box<LogHandler<'cb>>, owned: bool) -> Logger<'cb> {
		if inner.is_null() {
			panic!("Cannot allocate memory for Logger")
		}
		Logger { inner, owned, handler }
	}

	/// [xmpp_get_default_logger](http://strophe.im/libstrophe/doc/0.9.2/group___context.html#ga33abde406c7a057006b109cf1b23c8f8)
	///
	/// This method returns default `libstrophe` logger that just outputs log lines to stderr. Use it
	/// if you compile without `rust-log` feature and want a quick debug log output.
	pub fn new_internal(log_level: LogLevel) -> Logger<'static> {
		Logger::with_inner(
			unsafe { sys::xmpp_get_default_logger(log_level) },
			Box::new(|_, _, _| {}),
			false,
		)
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

impl Default for Logger<'static> {
	/// Return a new logger that logs to standard Rust logging facilities.
	///
	/// Logging facilities are provided by [`log`] crate. Only available when compiling with `rust-log`
	/// feature.
	///
	/// [`log`]: https://crates.io/crates/log
	#[cfg(feature = "rust-log")]
	fn default() -> Self {
		Logger::new(|log_level, area, message| {
			match log_level {
				LogLevel::XMPP_LEVEL_DEBUG => debug!("{}: {}", area, message),
				LogLevel::XMPP_LEVEL_INFO => info!("{}: {}", area, message),
				LogLevel::XMPP_LEVEL_WARN => warn!("{}: {}", area, message),
				LogLevel::XMPP_LEVEL_ERROR => error!("{}: {}", area, message),
			}
		})
	}

	/// Create a new default logger by calling [`new_internal()`] with debug log level.
	///
	/// Used when the crate is compiled without `rust-log` feature.
	///
	/// [`new_internal()`]: struct.Logger.html#method.new_internal
	#[cfg(not(feature = "rust-log"))]
	fn default() -> Self {
		Logger::new_internal(LogLevel::XMPP_LEVEL_DEBUG)
	}
}

impl PartialEq for Logger<'_> {
	fn eq(&self, other: &Logger) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Logger<'_> {}

impl fmt::Debug for Logger<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.debug_struct("Logger")
			.field("inner", &self.inner)
			.field("owned", &self.owned)
			.finish()
	}
}

impl hash::Hash for Logger<'_> {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
		self.owned.hash(state);
	}
}

impl Drop for Logger<'_> {
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				Box::from_raw(self.inner);
			}
		}
	}
}

unsafe impl Send for Logger<'_> {}
