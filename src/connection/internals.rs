#[cfg(any(feature = "libstrophe-0_11_0", feature = "libstrophe-0_12_0"))]
use std::any::TypeId;
use std::cell::RefCell;
#[cfg(feature = "libstrophe-0_12_0")]
use std::ffi::c_void;
#[cfg(any(feature = "libstrophe-0_11_0", feature = "libstrophe-0_12_0"))]
use std::ffi::{c_char, c_int};
use std::fmt;
use std::rc::Weak;

#[cfg(feature = "libstrophe-0_11_0")]
pub use libstrophe_0_11::*;
#[cfg(feature = "libstrophe-0_12_0")]
pub use libstrophe_0_12::*;

use crate::{Connection, ConnectionEvent, Context, Stanza};

#[cfg(feature = "libstrophe-0_11_0")]
mod libstrophe_0_11 {
	use std::any::TypeId;
	use std::collections::HashMap;
	use std::sync::RwLock;

	use once_cell::sync::Lazy;

	use crate::TlsCert;

	pub type CertFailCallback = dyn Fn(&TlsCert, &str) -> CertFailResult + Send + Sync;
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
	use std::any::TypeId;
	use std::collections::HashMap;
	use std::ffi::c_void;
	use std::sync::RwLock;

	use once_cell::sync::Lazy;

	use crate::connection::internals::FatHandler;
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

#[derive(Debug)]
#[repr(i32)]
pub enum HandlerResult {
	RemoveHandler = 0,
	KeepHandler = 1,
}

pub type ConnectionCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb;
pub type ConnectionFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, ConnectionCallback<'cb, 'cx>, ()>;

pub type Handlers<H> = Vec<Box<H>>;

pub type TimedCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb;
pub type TimedFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, TimedCallback<'cb, 'cx>, ()>;

pub type StanzaCallback<'cb, 'cx> =
	dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb;
pub type StanzaFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, StanzaCallback<'cb, 'cx>, Option<String>>;

pub struct FatHandlers<'cb, 'cx> {
	pub connection: Option<ConnectionFatHandler<'cb, 'cx>>,
	pub timed: Handlers<TimedFatHandler<'cb, 'cx>>,
	pub stanza: Handlers<StanzaFatHandler<'cb, 'cx>>,
	#[cfg(feature = "libstrophe-0_11_0")]
	pub cert_fail_handler_id: Option<TypeId>,
	#[cfg(feature = "libstrophe-0_12_0")]
	pub sockopt_handler_id: Option<TypeId>,
	#[cfg(feature = "libstrophe-0_12_0")]
	pub password: Handlers<PasswordFatHandler<'cb, 'cx>>,
}

impl fmt::Debug for FatHandlers<'_, '_> {
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
		s.field("timed", &format!("{} handlers", self.timed.len()));
		s.field("stanza", &format!("{} handlers", self.stanza.len()));
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
		s.field("password", &format!("{} handlers", self.password.len()));
		s.finish()
	}
}

pub struct FatHandler<'cb, 'cx, CB: ?Sized, T> {
	pub fat_handlers: Weak<RefCell<FatHandlers<'cb, 'cx>>>,
	pub handler: Box<CB>,
	pub cb_addr: *const (),
	pub extra: T,
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
		if $conn_ptr as *mut ::core::ffi::c_void == $userdata { // dummy condition that's never true
			$crate::void_ptr_as::<$typ>($userdata)($($args),*);
		}
	};
}

#[cfg(feature = "libstrophe-0_11_0")]
pub unsafe extern "C" fn certfail_handler_cb<CB: 'static>(cert: *const sys::xmpp_tlscert_t, errormsg: *const c_char) -> c_int {
	if let Ok(handlers) = CERT_FAIL_HANDLERS.read() {
		if let Some(handler) = handlers.get(&TypeId::of::<CB>()) {
			let cert = crate::TlsCert::from_ref(cert);
			let error_msg = crate::FFI(errormsg).receive().unwrap_or("Can't process libstrophe error");
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
