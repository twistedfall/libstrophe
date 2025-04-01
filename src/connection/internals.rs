#[cfg(feature = "libstrophe-0_11_0")]
use core::any::TypeId;
use core::cell::RefCell;
#[cfg(feature = "libstrophe-0_14")]
use core::slice;
use core::{fmt, ptr};
#[cfg(feature = "libstrophe-0_12_0")]
use std::ffi::CString;
use std::ffi::c_void;
#[cfg(feature = "libstrophe-0_11_0")]
use std::os::raw::{c_char, c_int};
use std::rc::{Rc, Weak};

#[cfg(feature = "libstrophe-0_11_0")]
pub use libstrophe_0_11::*;
#[cfg(feature = "libstrophe-0_12_0")]
pub use libstrophe_0_12::*;
#[cfg(feature = "libstrophe-0_14")]
pub use libstrophe_0_14::*;

#[cfg(feature = "libstrophe-0_14")]
use crate::SerializedSmStateRef;
use crate::{Connection, ConnectionEvent, Context, Stanza, as_void_ptr, void_ptr_as};

#[cfg(feature = "libstrophe-0_11_0")]
mod libstrophe_0_11 {
	use core::any::TypeId;
	use std::collections::HashMap;
	use std::sync::RwLock;

	use once_cell::sync::Lazy;

	use crate::TlsCert;

	pub type CertFailCallback = dyn Fn(&TlsCert, &str) -> CertFailResult + Send + Sync;
	/// Certificate failure and socket option handlers do not receive userdata argument so we use a `TypeId` keyed static to
	/// store their corresponding closure.
	pub static CERT_FAIL_HANDLERS: Lazy<RwLock<HashMap<TypeId, Box<CertFailCallback>>>> = Lazy::new(Default::default);

	#[derive(Debug)]
	#[repr(i32)]
	pub enum CertFailResult {
		TerminateConnection = 0,
		EstablishConnection = 1,
	}
}

#[cfg(feature = "libstrophe-0_12_0")]
mod libstrophe_0_12 {
	use core::any::TypeId;
	use core::ffi::c_void;
	use std::collections::HashMap;
	use std::sync::RwLock;

	use once_cell::sync::Lazy;

	use super::FatHandler;
	use crate::Connection;

	pub type SockoptCallback = dyn Fn(*mut c_void) -> SockoptResult + Send + Sync;
	pub static SOCKOPT_HANDLERS: Lazy<RwLock<HashMap<TypeId, Box<SockoptCallback>>>> = Lazy::new(Default::default);

	#[derive(Debug)]
	#[repr(i32)]
	pub enum SockoptResult {
		Ok = 0,
		Error = -1,
	}

	pub type PasswordCallback<'cb, 'cx> = dyn Fn(&Connection<'cb, 'cx>, usize) -> Option<String> + Send + 'cb;
	pub type PasswordFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, PasswordCallback<'cb, 'cx>, ()>;
}

#[cfg(feature = "libstrophe-0_14")]
mod libstrophe_0_14 {
	use super::FatHandler;
	use crate::Connection;
	use crate::sm_state::SerializedSmStateRef;

	pub type SmStateCallback<'cb, 'cx> = dyn FnMut(&mut Connection<'cb, 'cx>, SerializedSmStateRef) + Send + 'cb;
	pub type SmStateFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, SmStateCallback<'cb, 'cx>, ()>;
}

#[derive(Debug)]
#[repr(i32)]
pub enum HandlerResult {
	RemoveHandler = 0,
	KeepHandler = 1,
}

pub type ConnectionCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb;
pub type ConnectionFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, ConnectionCallback<'cb, 'cx>, ()>;

pub type TimedCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb;
pub type TimedFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, TimedCallback<'cb, 'cx>, ()>;

pub type StanzaCallback<'cb, 'cx> =
	dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb;
pub type StanzaFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, StanzaCallback<'cb, 'cx>, Option<String>>;

pub struct ConnectionHandlers<'cb, 'cx> {
	pub connection: Option<BoxedHandler<'cb, 'cx, ConnectionCallback<'cb, 'cx>, ()>>,
	pub timed: BoxedHandlers<'cb, 'cx, TimedCallback<'cb, 'cx>, ()>,
	pub stanza: BoxedHandlers<'cb, 'cx, StanzaCallback<'cb, 'cx>, Option<String>>,
	#[cfg(feature = "libstrophe-0_11_0")]
	pub cert_fail_handler_id: Option<TypeId>,
	#[cfg(feature = "libstrophe-0_12_0")]
	pub sockopt_handler_id: Option<TypeId>,
	#[cfg(feature = "libstrophe-0_12_0")]
	pub password: Option<BoxedHandler<'cb, 'cx, PasswordCallback<'cb, 'cx>, ()>>,
	#[cfg(feature = "libstrophe-0_14")]
	pub sm_state: Option<BoxedHandler<'cb, 'cx, SmStateCallback<'cb, 'cx>, ()>>,
}

impl fmt::Debug for ConnectionHandlers<'_, '_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut s = f.debug_struct("FatHandlers");
		s.field(
			"connection",
			&if self.connection.is_some() {
				"set"
			} else {
				"unset"
			},
		);
		s.field("timed", &format!("{} handlers", self.timed.count()));
		s.field("stanza", &format!("{} handlers", self.stanza.count()));
		#[cfg(feature = "libstrophe-0_11_0")]
		s.field(
			"cert_fail_handler_id",
			&if self.connection.is_some() {
				"set"
			} else {
				"unset"
			},
		);
		#[cfg(feature = "libstrophe-0_12_0")]
		s.field(
			"sockopt_handler_id",
			&if self.connection.is_some() {
				"set"
			} else {
				"unset"
			},
		);
		#[cfg(feature = "libstrophe-0_12_0")]
		s.field(
			"password",
			&if self.password.is_some() {
				"handler set"
			} else {
				"no handler"
			},
		);
		#[cfg(feature = "libstrophe-0_14")]
		s.field(
			"sm_state",
			&if self.sm_state.is_some() {
				"handler set"
			} else {
				"no handler"
			},
		);
		s.finish()
	}
}

pub struct FatHandler<'cb, 'cx, CB: ?Sized, Extra> {
	/// Weak reference back to the `Connection`'s `handlers` to be able to create a `Connection` object in the extern "C" callback
	pub handlers: Weak<RefCell<ConnectionHandlers<'cb, 'cx>>>,
	/// The actual Rust handler that will call will be forwarded to
	pub handler: Box<CB>,
	/// The address of the extern "C" callback
	pub cb_addr: CbAddr,
	/// Extra internal data associated with the handler
	pub extra: Extra,
}

impl<CB: ?Sized, Extra> FatHandler<'_, '_, CB, Extra> {
	pub fn as_userdata(&mut self) -> *mut c_void {
		as_void_ptr(self)
	}

	pub fn cb_addr(&self) -> CbAddr {
		self.cb_addr
	}

	pub unsafe fn from_userdata<'h>(userdata: *mut c_void) -> &'h mut Self {
		unsafe { void_ptr_as(userdata) }
	}
}

pub enum HandlerCb {}
pub type CbAddr = *const HandlerCb;

pub struct BoxedHandler<'cb, 'cx, CB: ?Sized, Extra> {
	handler: Box<FatHandler<'cb, 'cx, CB, Extra>>,
}

impl<'cb, 'cx, CB: ?Sized, Extra> BoxedHandler<'cb, 'cx, CB, Extra> {
	pub fn new(handler: FatHandler<'cb, 'cx, CB, Extra>) -> Self {
		Self {
			handler: Box::new(handler),
		}
	}

	pub fn make(handlers: &Rc<RefCell<ConnectionHandlers<'cb, 'cx>>>, handler: Box<CB>, cb_addr: CbAddr, extra: Extra) -> Self {
		Self::new(FatHandler {
			handlers: Rc::downgrade(handlers),
			handler,
			cb_addr,
			extra,
		})
	}

	fn cb_addr(&self) -> CbAddr {
		self.handler.cb_addr
	}

	fn as_ref(&self) -> &FatHandler<'cb, 'cx, CB, Extra> {
		self.handler.as_ref()
	}

	fn as_mut(&mut self) -> &mut FatHandler<'cb, 'cx, CB, Extra> {
		self.handler.as_mut()
	}
}

pub trait MaybeBoxedHandler<'cb, 'cx, CB: ?Sized, Extra> {
	fn set_handler(
		&mut self,
		handler: BoxedHandler<'cb, 'cx, CB, Extra>,
	) -> (
		Option<BoxedHandler<'cb, 'cx, CB, Extra>>,
		&mut FatHandler<'cb, 'cx, CB, Extra>,
	);
}

impl<'cb, 'cx, CB: ?Sized, Extra> MaybeBoxedHandler<'cb, 'cx, CB, Extra> for Option<BoxedHandler<'cb, 'cx, CB, Extra>> {
	fn set_handler(
		&mut self,
		handler: BoxedHandler<'cb, 'cx, CB, Extra>,
	) -> (
		Option<BoxedHandler<'cb, 'cx, CB, Extra>>,
		&mut FatHandler<'cb, 'cx, CB, Extra>,
	) {
		let old_handler = self.take();
		let self_ref = self.insert(handler).handler.as_mut();
		(old_handler, self_ref)
	}
}

pub struct BoxedHandlers<'cb, 'cx, CB: ?Sized, Extra> {
	handlers: Vec<BoxedHandler<'cb, 'cx, CB, Extra>>,
}

impl<'cb, 'cx, CB: ?Sized, Extra> BoxedHandlers<'cb, 'cx, CB, Extra> {
	pub fn new(capacity: usize) -> Self {
		Self {
			handlers: Vec::with_capacity(capacity),
		}
	}

	pub fn count(&self) -> usize {
		self.handlers.len()
	}

	pub fn retain(&mut self, mut cb: impl FnMut(&FatHandler<'cb, 'cx, CB, Extra>) -> bool) {
		self.handlers.retain(|boxed| cb(boxed.as_ref()));
		self.handlers.shrink_to_fit();
	}

	pub fn store(&mut self, handler: BoxedHandler<'cb, 'cx, CB, Extra>) -> Option<&mut FatHandler<'cb, 'cx, CB, Extra>> {
		if !self
			.handlers
			.iter()
			.any(|existing| ptr::eq(handler.cb_addr(), existing.cb_addr()))
		{
			self.handlers.push(handler);
			Some(self.handlers.last_mut().expect("Impossible, just pushed it").as_mut())
		} else {
			None
		}
	}

	pub fn validate(&self, cb_addr: CbAddr) -> Option<&FatHandler<'cb, 'cx, CB, Extra>> {
		self
			.handlers
			.iter()
			.find_map(|boxed| ptr::eq(cb_addr, boxed.cb_addr()).then_some(boxed.as_ref()))
	}

	pub fn drop_handler(&mut self, cb_addr: CbAddr) -> usize {
		let mut removed_count = 0;
		self.handlers.retain(|handler| {
			let keep_it = !ptr::eq(cb_addr, handler.cb_addr());
			if !keep_it {
				removed_count += 1;
			}
			keep_it
		});
		removed_count
	}
}

/// In the release mode Rust/LLVM tries to meld functions that have identical bodies together,
/// but the crate code requires that monomorphized callback functions passed to C remain unique.
/// Those are `connection_handler_cb`, `timed_handler_cb`, `handler_cb`. They are not making
/// any use of the type argument in their bodies thus there will be only one function address for
/// each callback function and libstrophe rejects callback with the same address. This macro
/// imitates the use of the typed argument so that the code is actually different and those
/// functions are not melded together.
macro_rules! ensure_unique {
	($typ: ty, $conn_ptr: ident, $userdata: ident, $($args: expr),*) => {
		if $conn_ptr.cast::<::core::ffi::c_void>() == $userdata { // dummy condition that's never true
			unsafe { $crate::void_ptr_as::<$typ>($userdata)($($args),*) };
		}
	};
}

#[cfg(feature = "libstrophe-0_11_0")]
pub unsafe extern "C" fn certfail_handler_cb<CB: 'static>(cert: *const sys::xmpp_tlscert_t, errormsg: *const c_char) -> c_int {
	if let Ok(handlers) = CERT_FAIL_HANDLERS.read() {
		if let Some(handler) = handlers.get(&TypeId::of::<CB>()) {
			let cert = unsafe { crate::TlsCert::from_ref(cert) };
			let error_msg = unsafe { crate::FFI(errormsg).receive() }.unwrap_or("Can't process libstrophe error");
			return handler(&cert, error_msg) as c_int;
		}
	}
	CertFailResult::TerminateConnection as c_int
}

#[cfg(feature = "libstrophe-0_12_0")]
pub unsafe extern "C" fn sockopt_callback<CB: 'static>(_conn: *mut sys::xmpp_conn_t, sock: *mut c_void) -> c_int {
	if let Ok(handlers) = SOCKOPT_HANDLERS.read() {
		if let Some(handler) = handlers.get(&TypeId::of::<CB>()) {
			return handler(sock) as c_int;
		}
	}
	SockoptResult::Error as c_int
}

#[cfg(feature = "libstrophe-0_12_0")]
pub unsafe extern "C" fn password_handler_cb(
	pw: *mut c_char,
	pw_max: usize,
	conn_ptr: *mut sys::xmpp_conn_t,
	userdata: *mut c_void,
) -> c_int {
	let password_handler = unsafe { PasswordFatHandler::from_userdata(userdata) };
	if let Some(fat_handlers) = password_handler.handlers.upgrade() {
		let conn = unsafe { Connection::from_ref_mut(conn_ptr, fat_handlers) };
		// we need to leave place for the null byte that will be written by libstrophe
		let max_password_len = pw_max.saturating_sub(1);
		let result = (password_handler.handler)(&conn, max_password_len);
		if let Some(password) = result {
			if let Ok(password) = CString::new(password) {
				if password.as_bytes().len() <= max_password_len {
					let pass_len = password.as_bytes().len();
					unsafe { ptr::copy_nonoverlapping(password.as_ptr(), pw, pass_len) };
					return c_int::try_from(pass_len).unwrap_or(c_int::MAX);
				}
			}
		}
	}
	-1
}

#[cfg(feature = "libstrophe-0_14")]
pub unsafe extern "C" fn sm_state_handler_cb(
	conn_ptr: *mut sys::xmpp_conn_t,
	userdata: *mut c_void,
	buf: *const std::ffi::c_uchar,
	size: usize,
) {
	let buf = if buf.is_null() {
		&[]
	} else {
		unsafe { slice::from_raw_parts(buf, size) }
	};
	let serialized = SerializedSmStateRef { buf };
	let sm_state_handler = unsafe { SmStateFatHandler::from_userdata(userdata) };
	if let Some(fat_handlers) = sm_state_handler.handlers.upgrade() {
		let mut conn = unsafe { Connection::from_ref_mut(conn_ptr, fat_handlers) };
		(sm_state_handler.handler)(&mut conn, serialized);
	}
}
