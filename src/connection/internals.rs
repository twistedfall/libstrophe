use std::{
	cell::RefCell,
	fmt,
	rc::Weak,
};

use crate::{Connection, ConnectionEvent, Context, Stanza};

pub type ConnectionCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb;
pub type ConnectionFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, ConnectionCallback<'cb, 'cx>, ()>;

pub type Handlers<H> = Vec<Box<H>>;

pub type TimedCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> bool + Send + 'cb;
pub type TimedFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, TimedCallback<'cb, 'cx>, ()>;

pub type StanzaCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> bool + Send + 'cb;
pub type StanzaFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, StanzaCallback<'cb, 'cx>, Option<String>>;

pub struct FatHandlers<'cb, 'cx> {
	pub connection: Option<ConnectionFatHandler<'cb, 'cx>>,
	pub timed: Handlers<TimedFatHandler<'cb, 'cx>>,
	pub stanza: Handlers<StanzaFatHandler<'cb, 'cx>>,
}

impl fmt::Debug for FatHandlers<'_, '_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut s = f.debug_struct("FatHandlers");
		s.field("connection", &if self.connection.is_some() { "set" } else { "unset" });
		s.field("timed", &format!("{} handlers", self.timed.len()));
		s.field("stanza", &format!("{} handlers", self.stanza.len()));
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
		if $conn_ptr as *mut ::core::ffi::c_void == $userdata {
			$crate::void_ptr_as::<$typ>($userdata)($($args),*);
		}
	};
}

