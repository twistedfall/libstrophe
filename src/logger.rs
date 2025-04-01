use core::ffi::{c_char, c_void};
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ptr::NonNull;

#[cfg(feature = "log")]
use log::{debug, error, info, warn};

use crate::{FFI, LogLevel, as_void_ptr, void_ptr_as};

type LogHandler<'cb> = dyn Fn(LogLevel, &str, &str) + Send + 'cb;

/// Wrapper around the underlying [sys::xmpp_log_t] struct.
///
/// The best option to get a logger is to call [Logger::default()]. It will return you a logger that
/// is tied into Rust logging facility provided by [log] crate. This functionality is available when
/// compiling with the default `rust-log` feature.
///
/// This struct implements:
///
///   * `Eq` by comparing internal pointers
///   * `Hash` by hashing internal pointer
///   * `Send`
pub struct Logger<'cb> {
	inner: NonNull<sys::xmpp_log_t>,
	owned: bool,
	handler: Box<LogHandler<'cb>>,
}

impl<'cb> Logger<'cb> {
	/// Create a new custom logger.
	///
	/// The callback argument will be called every time a log message needs to be printed.
	pub fn new<CB>(handler: CB) -> Self
	where
		CB: Fn(LogLevel, &str, &str) + Send + 'cb,
	{
		let mut handler = Box::new(handler);
		Logger::with_inner(
			Box::into_raw(Box::new(sys::xmpp_log_t {
				handler: Some(Self::log_handler_cb::<CB>),
				userdata: as_void_ptr(handler.as_mut()),
			})),
			handler,
			true,
		)
	}

	#[inline]
	fn with_inner(inner: *mut sys::xmpp_log_t, handler: Box<LogHandler<'cb>>, owned: bool) -> Self {
		Logger {
			inner: NonNull::new(inner).expect("Cannot allocate memory for Logger"),
			owned,
			handler,
		}
	}

	/// [xmpp_get_default_logger](https://strophe.im/libstrophe/doc/0.13.0/group___context.html#ga40caddfbd7d786f8ef1390866880edb9)
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

	/// This method returns null logger that doesn't output any information.
	pub fn new_null() -> Logger<'static> {
		Logger::new(|_, _, _| {})
	}

	unsafe extern "C" fn log_handler_cb<CB>(
		userdata: *mut c_void,
		level: sys::xmpp_log_level_t,
		area: *const c_char,
		msg: *const c_char,
	) where
		CB: FnMut(LogLevel, &str, &str) + Send + 'cb,
	{
		let area = unsafe { FFI(area).receive() }.unwrap();
		let msg = unsafe { FFI(msg).receive() }.unwrap();
		unsafe { void_ptr_as::<CB>(userdata)(level, area, msg) }
	}

	pub(crate) fn as_ptr(&self) -> *const sys::xmpp_log_t {
		self.inner.as_ptr()
	}

	pub fn log(&self, level: LogLevel, area: &str, msg: &str) {
		(self.handler)(level, area, msg);
	}
}

impl Default for Logger<'static> {
	/// Return a new logger that logs to standard Rust logging facilities.
	///
	/// Logging facilities are provided by [`log`] crate. Only available when compiling with `rust-log`
	/// feature.
	///
	/// [`log`]: https://crates.io/crates/log
	#[cfg(feature = "log")]
	fn default() -> Self {
		Logger::new(|log_level, area, message| match log_level {
			LogLevel::XMPP_LEVEL_DEBUG => debug!("{area}: {message}"),
			LogLevel::XMPP_LEVEL_INFO => info!("{area}: {message}"),
			LogLevel::XMPP_LEVEL_WARN => warn!("{area}: {message}"),
			LogLevel::XMPP_LEVEL_ERROR => error!("{area}: {message}"),
		})
	}

	/// Create a new default logger by calling [`new_internal()`] with debug log level.
	///
	/// Used when the crate is compiled without `rust-log` feature.
	///
	/// [`new_internal()`]: struct.Logger.html#method.new_internal
	#[cfg(not(feature = "log"))]
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

impl Hash for Logger<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
	}
}

impl Drop for Logger<'_> {
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				drop(Box::from_raw(self.inner.as_mut()));
			}
		}
	}
}

unsafe impl Send for Logger<'_> {}

#[test]
fn callbacks() {
	fn logger_eq<L, R>(_left: L, _right: R) -> bool
	where
		L: FnMut(LogLevel, &str, &str) + Send + Sync,
		R: FnMut(LogLevel, &str, &str) + Send + Sync,
	{
		std::ptr::eq(
			Logger::log_handler_cb::<L> as *const (),
			Logger::log_handler_cb::<R> as *const (),
		)
	}

	let a = |_: LogLevel, _: &str, _: &str| {
		println!("1");
	};
	let b = |_: LogLevel, _: &str, _: &str| {
		println!("2");
	};

	assert!(logger_eq(a, a));
	assert!(!logger_eq(a, b));
}
