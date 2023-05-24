#[cfg(feature = "libstrophe-0_11_0")]
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
#[cfg(feature = "libstrophe-0_12_0")]
use std::ffi::CString;
#[cfg(feature = "libstrophe-0_12_0")]
use std::os::raw::c_char;
use std::os::raw::{c_int, c_ulong};
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Duration;
use std::{fmt, mem, ptr, result, str};

#[cfg(feature = "libstrophe-0_11_0")]
pub use internals::CertFailResult;
pub use internals::HandlerResult;
#[cfg(feature = "libstrophe-0_12_0")]
pub use internals::SockoptResult;
#[cfg(feature = "libstrophe-0_11_0")]
use internals::CERT_FAIL_HANDLERS;
use internals::{ConnectionFatHandler, FatHandler, FatHandlers, Handlers, StanzaFatHandler, TimedFatHandler};
#[cfg(feature = "libstrophe-0_12_0")]
use internals::{PasswordFatHandler, SOCKOPT_HANDLERS};

use crate::error::IntoResult;
use crate::ffi_types::Nullable;
#[cfg(feature = "libstrophe-0_11_0")]
pub use crate::TlsCert;
use crate::{
	as_void_ptr, void_ptr_as, ConnectClientError, ConnectionError, ConnectionFlags, Context, Error, Result, Stanza, StreamError,
	FFI,
};
#[cfg(feature = "libstrophe-0_12_0")]
use crate::{QueueElement, SMState};

#[macro_use]
mod internals;

/// Proxy to the underlying `xmpp_conn_t` struct.
///
/// Most of the methods in this struct mimic the methods of the underlying library. So please see
/// libstrophe docs for [connection] and [handlers] plus [conn.c] and [handler.c] sources.
/// Only where it's not the case or there is some additional logic involved then you can see the
/// method description.
///
/// This struct implements:
///
///   * `Drop` ([xmpp_conn_release]).
///   * `Eq` by comparing internal pointers
///   * `Send`
///
/// [connection]: https://strophe.im/libstrophe/doc/0.12.2/group___connections.html
/// [handlers]: https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html
/// [conn.c]: https://github.com/strophe/libstrophe/blob/0.12.2/src/conn.c
/// [handler.c]: https://github.com/strophe/libstrophe/blob/0.12.2/src/handler.c
/// [xmpp_conn_release]: https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga87b076b11589bc23123096dc83cde6a8
#[derive(Debug)]
pub struct Connection<'cb, 'cx> {
	inner: NonNull<sys::xmpp_conn_t>,
	ctx: Option<Context<'cx, 'cb>>,
	owned: bool,
	fat_handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>,
}

impl<'cb, 'cx> Connection<'cb, 'cx> {
	#[inline]
	/// [xmpp_conn_new](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga0bc7c0e07b52bb7470a97e8d9f9542be)
	pub fn new(ctx: Context<'cx, 'cb>) -> Self {
		unsafe {
			Self::from_owned(
				sys::xmpp_conn_new(ctx.as_ptr()),
				ctx,
				Rc::new(RefCell::new(FatHandlers {
					connection: None,
					timed: Vec::with_capacity(4),
					stanza: Vec::with_capacity(4),
					#[cfg(feature = "libstrophe-0_11_0")]
					cert_fail_handler_id: None,
					#[cfg(feature = "libstrophe-0_12_0")]
					sockopt_handler_id: None,
					#[cfg(feature = "libstrophe-0_12_0")]
					password: vec![],
				})),
			)
		}
	}

	#[inline]
	unsafe fn with_inner(
		inner: *mut sys::xmpp_conn_t,
		ctx: Context<'cx, 'cb>,
		owned: bool,
		handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>,
	) -> Self {
		Connection {
			inner: NonNull::new(inner).expect("Cannot allocate memory for Connection"),
			ctx: Some(ctx),
			owned,
			fat_handlers: handlers,
		}
	}

	#[inline]
	unsafe fn from_owned(
		inner: *mut sys::xmpp_conn_t,
		ctx: Context<'cx, 'cb>,
		handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>,
	) -> Self {
		Self::with_inner(inner, ctx, true, handlers)
	}

	#[inline]
	unsafe fn from_ref_mut(inner: *mut sys::xmpp_conn_t, handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>) -> Self {
		let ctx = Context::from_ref(sys::xmpp_conn_get_context(inner));
		Self::with_inner(inner, ctx, false, handlers)
	}

	unsafe extern "C" fn connection_handler_cb<CB>(
		conn_ptr: *mut sys::xmpp_conn_t,
		event: sys::xmpp_conn_event_t,
		error: c_int,
		stream_error: *mut sys::xmpp_stream_error_t,
		userdata: *mut c_void,
	) where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let connection_handler = void_ptr_as::<ConnectionFatHandler>(userdata);
		if let Some(fat_handlers) = connection_handler.fat_handlers.upgrade() {
			let mut conn = Self::from_ref_mut(conn_ptr, fat_handlers);
			let event = match event {
				sys::xmpp_conn_event_t::XMPP_CONN_RAW_CONNECT => ConnectionEvent::RawConnect,
				sys::xmpp_conn_event_t::XMPP_CONN_CONNECT => ConnectionEvent::Connect,
				sys::xmpp_conn_event_t::XMPP_CONN_DISCONNECT => {
					let stream_error: Option<StreamError> = stream_error.as_ref().map(|e| e.into());
					ConnectionEvent::Disconnect(ConnectionError::from((error, stream_error)))
				}
				sys::xmpp_conn_event_t::XMPP_CONN_FAIL => unreachable!("XMPP_CONN_FAIL is never used in the underlying library"),
			};
			ensure_unique!(
				CB,
				conn_ptr,
				userdata,
				conn.context_detached(),
				&mut conn,
				ConnectionEvent::Connect
			);
			(connection_handler.handler)(conn.context_detached(), &mut conn, event);
		}
	}

	unsafe extern "C" fn timed_handler_cb<CB>(conn_ptr: *mut sys::xmpp_conn_t, userdata: *mut c_void) -> c_int
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb,
	{
		let timed_handler = void_ptr_as::<TimedFatHandler>(userdata);
		if let Some(fat_handlers) = timed_handler.fat_handlers.upgrade() {
			let mut conn = Self::from_ref_mut(conn_ptr, fat_handlers);
			ensure_unique!(CB, conn_ptr, userdata, conn.context_detached(), &mut conn);
			let res = (timed_handler.handler)(conn.context_detached(), &mut conn);
			if matches!(res, HandlerResult::RemoveHandler) {
				Self::drop_fat_handler(&mut conn.fat_handlers.borrow_mut().timed, timed_handler);
			}
			res as c_int
		} else {
			HandlerResult::RemoveHandler as c_int
		}
	}

	unsafe extern "C" fn handler_cb<CB>(
		conn_ptr: *mut sys::xmpp_conn_t,
		stanza: *mut sys::xmpp_stanza_t,
		userdata: *mut c_void,
	) -> c_int
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
	{
		let stanza_handler = void_ptr_as::<StanzaFatHandler>(userdata);
		if let Some(fat_handlers) = stanza_handler.fat_handlers.upgrade() {
			let mut conn = Self::from_ref_mut(conn_ptr, fat_handlers);
			let stanza = Stanza::from_ref(stanza);
			ensure_unique!(CB, conn_ptr, userdata, conn.context_detached(), &mut conn, &stanza);
			let res = (stanza_handler.handler)(conn.context_detached(), &mut conn, &stanza);
			if matches!(res, HandlerResult::RemoveHandler) {
				Self::drop_fat_handler(&mut conn.fat_handlers.borrow_mut().stanza, stanza_handler);
			}
			res as c_int
		} else {
			HandlerResult::RemoveHandler as c_int
		}
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	unsafe extern "C" fn password_handler_cb<CB>(
		pw: *mut c_char,
		pw_max: usize,
		conn_ptr: *mut sys::xmpp_conn_t,
		userdata: *mut c_void,
	) -> c_int
	where
		CB: FnMut(&Connection<'cb, 'cx>, usize) -> Option<String> + Send + 'cb,
	{
		let password_handler = void_ptr_as::<PasswordFatHandler>(userdata);
		if let Some(fat_handlers) = password_handler.fat_handlers.upgrade() {
			let conn = Self::from_ref_mut(conn_ptr, fat_handlers);
			// we need to leave place for the null byte that will be written by libstrophe
			let max_password_len = if pw_max < 1 {
				0
			} else {
				pw_max - 1
			};
			ensure_unique!(CB, conn_ptr, userdata, &conn, max_password_len);
			let result = (password_handler.handler)(&conn, max_password_len);
			if let Some(password) = result {
				if let Ok(password) = CString::new(password) {
					if password.as_bytes().len() <= max_password_len {
						let pass_len = password.as_bytes().len();
						ptr::copy_nonoverlapping(password.as_ptr(), pw, pass_len);
						return pass_len as c_int;
					}
				}
			}
		}
		-1
	}

	fn store_fat_handler<CB: ?Sized, T>(
		fat_handlers: &mut Handlers<FatHandler<'cb, 'cx, CB, T>>,
		fat_handler: FatHandler<'cb, 'cx, CB, T>,
	) -> Option<*const FatHandler<'cb, 'cx, CB, T>> {
		if Self::get_fat_handler_pos_by_callback(fat_handlers, fat_handler.cb_addr).is_none() {
			let handler = Box::new(fat_handler);
			let out = &*handler as _;
			fat_handlers.push(handler);
			Some(out)
		} else {
			None
		}
	}

	fn get_fat_handler_pos<CB: ?Sized, T>(
		fat_handlers: &Handlers<FatHandler<'cb, 'cx, CB, T>>,
		fat_handler_ptr: *const FatHandler<'cb, 'cx, CB, T>,
	) -> Option<usize> {
		fat_handlers.iter().position(|x| fat_handler_ptr == x.as_ref())
	}

	fn get_fat_handler_pos_by_callback<CB: ?Sized, T>(
		fat_handlers: &Handlers<FatHandler<'cb, 'cx, CB, T>>,
		cb_addr: *const (),
	) -> Option<usize> {
		fat_handlers.iter().position(|x| cb_addr == x.cb_addr)
	}

	fn validate_fat_handler<'f, CB: ?Sized, T>(
		fat_handlers: &'f Handlers<FatHandler<'cb, 'cx, CB, T>>,
		fat_handler_ptr: *const FatHandler<'cb, 'cx, CB, T>,
	) -> Option<&'f FatHandler<'cb, 'cx, CB, T>> {
		Self::get_fat_handler_pos(fat_handlers, fat_handler_ptr).map(|pos| fat_handlers[pos].as_ref())
	}

	fn drop_fat_handler<CB: ?Sized, T>(
		fat_handlers: &mut Handlers<FatHandler<'cb, 'cx, CB, T>>,
		fat_handler_ptr: *const FatHandler<'cb, 'cx, CB, T>,
	) -> Option<usize> {
		if let Some(pos) = Self::get_fat_handler_pos(fat_handlers, fat_handler_ptr) {
			fat_handlers.remove(pos);
			Some(pos)
		} else {
			None
		}
	}

	#[inline]
	fn make_fat_handler<CB: ?Sized, T>(&self, handler: Box<CB>, cb_addr: *const (), extra: T) -> FatHandler<'cb, 'cx, CB, T> {
		FatHandler {
			fat_handlers: Rc::downgrade(&self.fat_handlers),
			handler,
			cb_addr,
			extra,
		}
	}

	#[inline]
	unsafe fn context_detached<'a>(&self) -> &'a Context<'cx, 'cb> {
		(self.ctx.as_ref().unwrap() as *const Context).as_ref().unwrap()
	}

	#[inline]
	/// [xmpp_conn_get_flags](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga8acc2ae11389af17229b41b4c39ed16e)
	pub fn flags(&self) -> ConnectionFlags {
		ConnectionFlags::from_bits(unsafe { sys::xmpp_conn_get_flags(self.inner.as_ptr()) }).unwrap()
	}

	#[inline]
	/// [xmpp_conn_set_flags](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga6e36f1cb6ba2e8870ace8d91dd0b1535)
	pub fn set_flags(&mut self, flags: ConnectionFlags) -> Result<()> {
		unsafe { sys::xmpp_conn_set_flags(self.inner.as_mut(), flags.bits()) }.into_result()
	}

	#[inline]
	/// [xmpp_conn_get_jid](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga37a4edf0ec15c78e570165eb65a3cbad)
	pub fn jid(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_conn_get_jid(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_conn_get_bound_jid](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga9b055bfeb4d81c009e4d0fcf60c3596a)
	pub fn bound_jid(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_conn_get_bound_jid(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_conn_set_jid](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gab78bfef71b5c04ba1086da20f79ca61f)
	pub fn set_jid(&mut self, jid: impl AsRef<str>) {
		let jid = FFI(jid.as_ref()).send();
		unsafe { sys::xmpp_conn_set_jid(self.inner.as_mut(), jid.as_ptr()) }
	}

	#[inline]
	/// [xmpp_conn_get_pass](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga6b84d1f6f3ef644378138c163b58ed75)
	pub fn pass(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_conn_get_pass(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_conn_set_pass](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gac5069924deadf5f2e38db01e6e960979)
	pub fn set_pass(&mut self, pass: impl AsRef<str>) {
		let pass = FFI(pass.as_ref()).send();
		unsafe { sys::xmpp_conn_set_pass(self.inner.as_mut(), pass.as_ptr()) }
	}

	#[inline]
	#[deprecated]
	/// [xmpp_conn_disable_tls](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga7f3012810acf47f6713032a26cb97a42)
	pub fn disable_tls(&mut self) {
		unsafe { sys::xmpp_conn_disable_tls(self.inner.as_mut()) }
	}

	#[inline]
	/// [xmpp_conn_is_secured](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaf37c90a76c0840ace266630025c88a82)
	pub fn is_secured(&self) -> bool {
		unsafe { FFI(sys::xmpp_conn_is_secured(self.inner.as_ptr())).receive_bool() }
	}

	#[inline]
	#[cfg_attr(feature = "libstrophe-0_12_0", deprecated(note = "replaced by set_sockopt_callback()"))]
	/// [xmpp_conn_set_keepalive](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga044f1e5d519bff84066317cf8b9fe607)
	pub fn set_keepalive(&mut self, timeout: Duration, interval: Duration) {
		unsafe { sys::xmpp_conn_set_keepalive(self.inner.as_mut(), timeout.as_secs() as _, interval.as_secs() as _) }
	}

	#[cfg(feature = "libstrophe-0_10_0")]
	#[inline]
	/// [xmpp_conn_is_connecting](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaafdd92d7678050c4bf225fa85a442c46)
	pub fn is_connecting(&self) -> bool {
		unsafe { FFI(sys::xmpp_conn_is_connecting(self.inner.as_ptr())).receive_bool() }
	}

	#[cfg(feature = "libstrophe-0_10_0")]
	#[inline]
	/// [xmpp_conn_is_connected](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga1956e25131d68c0134a2ee676ee3be3f)
	pub fn is_connected(&self) -> bool {
		unsafe { FFI(sys::xmpp_conn_is_connected(self.inner.as_ptr())).receive_bool() }
	}

	#[cfg(feature = "libstrophe-0_10_0")]
	#[inline]
	/// [xmpp_conn_is_disconnected](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gae6560b1fe5f4f637035d379e1c9c1007)
	pub fn is_disconnected(&self) -> bool {
		unsafe { FFI(sys::xmpp_conn_is_disconnected(self.inner.as_ptr())).receive_bool() }
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	#[inline]
	/// [xmpp_conn_set_cafile](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga508e9d6fa6b993e337b62af42cb655f6)
	pub fn set_cafile(&mut self, path: impl AsRef<str>) {
		let path = FFI(path.as_ref()).send();
		unsafe { sys::xmpp_conn_set_cafile(self.inner.as_ptr(), path.as_ptr()) }
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	#[inline]
	/// [xmpp_conn_set_capath](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga80b4ecf6a1a364acd26eadd5ab54cb71)
	pub fn set_capath(&mut self, path: impl AsRef<str>) {
		let path = FFI(path.as_ref()).send();
		unsafe { sys::xmpp_conn_set_capath(self.inner.as_ptr(), path.as_ptr()) }
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	/// [xmpp_conn_set_certfail_handler](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga4f24b0fb42ab541f902d5e15b3b59b33)
	/// [xmpp_certfail_handler](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga2e4aa651337c0aaf25b60ea160c2f4bd)
	///
	/// Callback function receives [TlsCert] object object and an error message.
	pub fn set_certfail_handler<CB>(&mut self, handler: CB)
	where
		CB: Fn(&TlsCert, &str) -> CertFailResult + Send + Sync + 'static,
	{
		let callback = internals::certfail_handler_cb::<CB>;
		if let Ok(mut handlers) = CERT_FAIL_HANDLERS.write() {
			let type_id = TypeId::of::<CB>();
			handlers.insert(type_id, Box::new(handler));
			if let Some(prev_handler_id) = self.fat_handlers.borrow_mut().cert_fail_handler_id.replace(type_id) {
				handlers.remove(&prev_handler_id);
			}
		};
		unsafe { sys::xmpp_conn_set_certfail_handler(self.inner.as_ptr(), Some(callback)) }
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	#[inline]
	/// [xmpp_conn_get_peer_cert](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga99415d183ffc99de3157876448d3282a)
	pub fn peer_cert(&self) -> Option<TlsCert> {
		unsafe {
			let cert = sys::xmpp_conn_get_peer_cert(self.inner.as_ptr());
			if cert.is_null() {
				None
			} else {
				Some(TlsCert::from_owned(cert))
			}
		}
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	#[inline]
	/// [xmpp_conn_set_client_cert](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#gac3d770588b083d2053a6361c9e49f235)
	pub fn set_client_cert(&mut self, cert_path: &str, key_path: &str) {
		let cert_path = FFI(cert_path).send();
		let key_path = FFI(key_path).send();
		unsafe {
			sys::xmpp_conn_set_client_cert(self.inner.as_ptr(), cert_path.as_ptr(), key_path.as_ptr());
		}
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	#[inline]
	/// [xmpp_conn_cert_xmppaddr_num](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#gaad61d0db95b0f22876df9403a728c806)
	pub fn cert_xmppaddr_num(&self) -> u32 {
		unsafe { sys::xmpp_conn_cert_xmppaddr_num(self.inner.as_ptr()) }
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	#[inline]
	/// [xmpp_conn_cert_xmppaddr](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga755f47fb1fbe8ce8e43ea93e5bc103a7)
	pub fn cert_xmppaddr(&self, n: u32) -> Option<String> {
		unsafe { FFI(sys::xmpp_conn_cert_xmppaddr(self.inner.as_ptr(), n)).receive_with_free(|x| crate::ALLOC_CONTEXT.free(x)) }
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_set_password_callback](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#gadcd27378977412d49ede93a5542f01e4)
	/// [xmpp_password_callback](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga140b726d8daf4175a009b3f7cb414593)
	///
	/// Callback function receives Connection object and maximum allowed length of the password, it returns `Some(String)` with password
	/// on success or None in case of error. If the returned `String` is longer than maximum allowed length it is ignored and the error
	/// is returned.
	pub fn set_password_callback<CB>(&mut self, handler: Option<CB>)
	where
		CB: Fn(&Connection<'cb, 'cx>, usize) -> Option<String> + Send + 'cb,
	{
		if let Some(handler) = handler {
			let callback = Self::password_handler_cb::<CB>;
			let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, ());
			let fat_handlers = Rc::clone(&self.fat_handlers);
			if let Some(fat_handler_ptr) = Self::store_fat_handler(&mut fat_handlers.borrow_mut().password, handler) {
				unsafe {
					sys::xmpp_conn_set_password_callback(self.inner.as_mut(), Some(callback), fat_handler_ptr as _);
				}
			};
		} else {
			unsafe { sys::xmpp_conn_set_password_callback(self.inner.as_mut(), None, ptr::null_mut()) }
		}
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	/// [xmpp_conn_set_sockopt_callback](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga40d4c1bc7dbd22d356067fd2105ba685)
	/// [xmpp_sockopt_callback](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gab69556790910b0875d9aa8564c415384)
	///
	/// Callback function receives pointer to a system-dependent socket object. See docs above for more details.
	pub fn set_sockopt_callback<CB>(&mut self, handler: CB)
	where
		CB: Fn(*mut c_void) -> SockoptResult + Send + Sync + 'static,
	{
		let callback = internals::sockopt_callback::<CB>;
		if let Ok(mut handlers) = SOCKOPT_HANDLERS.write() {
			let type_id = TypeId::of::<CB>();
			handlers.insert(type_id, Box::new(handler));
			if let Some(prev_handler_id) = self.fat_handlers.borrow_mut().sockopt_handler_id.replace(type_id) {
				handlers.remove(&prev_handler_id);
			}
		};
		unsafe { sys::xmpp_conn_set_sockopt_callback(self.inner.as_mut(), Some(callback)) }
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_sockopt_cb_keepalive](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga044f1e5d519bff84066317cf8b9fe607)
	///
	/// Sets default sockopt_callback function that just uses compile-time internal defaults for the socket timeout. Those
	/// values can be changed with a deprecated [Connection::set_keepalive]. If you use that function then you don't need to call
	/// [Connection::set_default_sockopt_callback] manually because it will be called internally.
	pub fn set_default_sockopt_callback(&mut self) {
		unsafe { sys::xmpp_conn_set_sockopt_callback(self.inner.as_mut(), Some(sys::xmpp_sockopt_cb_keepalive)) }
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_set_password_retries](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#ga0908b5362c1169db0867c5f01e8a64ae)
	pub fn set_password_retries(&mut self, n: u32) {
		unsafe { sys::xmpp_conn_set_password_retries(self.inner.as_ptr(), n) }
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_get_keyfile](https://strophe.im/libstrophe/doc/0.12.2/group___t_l_s.html#gab105ceda87046748f8e9275b066438a3)
	pub fn get_keyfile(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_conn_get_keyfile(self.inner.as_ptr())).receive() }
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_send_queue_len](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga769256a8b1460721deaf19c84b78ead9)
	pub fn send_queue_len(&self) -> i32 {
		unsafe { sys::xmpp_conn_send_queue_len(self.inner.as_ptr()) }
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_send_queue_drop_element](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga0fc31cf27113a905934c7cf8bb9b9c19)
	pub fn send_queue_drop_element(&mut self, which: QueueElement) -> Option<String> {
		unsafe {
			FFI(sys::xmpp_conn_send_queue_drop_element(self.inner.as_ptr(), which))
				.receive_with_free(|x| crate::ALLOC_CONTEXT.free(x))
		}
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_get_sm_state](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaf990c2fd8867258545b182f52df1465e)
	pub fn sm_state(&mut self) -> Option<SMState> {
		let inner = unsafe { sys::xmpp_conn_get_sm_state(self.inner.as_mut()) };
		if inner.is_null() {
			None
		} else {
			Some(unsafe { SMState::from_owned(inner) })
		}
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	#[inline]
	/// [xmpp_conn_set_sm_state](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gace3fa6449c31ce4f5db6ab9f0375eb47)
	pub fn set_sm_state(&mut self, sm_state: SMState) -> Result<()> {
		unsafe { sys::xmpp_conn_set_sm_state(self.inner.as_mut(), sm_state.into_inner()).into_result() }
	}

	/// [xmpp_connect_client](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga9354fc82ccbbce2840fca7efa9603c13)
	/// [xmpp_conn_handler](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#aad7c657ae239a87e2c2b746f99138e99)
	pub fn connect_client<CB>(
		mut self,
		alt_host: Option<&str>,
		alt_port: impl Into<Option<u16>>,
		handler: CB,
	) -> Result<Context<'cx, 'cb>, ConnectClientError<'cb, 'cx>>
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if self.jid().is_none() {
			return Err(ConnectClientError {
				conn: self,
				error: Error::InvalidOperation,
			});
		}
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(self.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut self.fat_handlers.borrow_mut().connection, new_handler);
		let out = unsafe {
			sys::xmpp_connect_client(
				self.inner.as_mut(),
				alt_host.as_ptr(),
				alt_port.val(),
				Some(callback),
				as_void_ptr(self.fat_handlers.borrow().connection.as_ref().unwrap()),
			)
		}
		.into_result();
		match out {
			Ok(_) => {
				let mut out = self.ctx.take().expect("Internal context is empty, it must never happen");
				out.consume_connection(self);
				Ok(out)
			}
			Err(e) => {
				self.fat_handlers.borrow_mut().connection = old_handler;
				Err(ConnectClientError { conn: self, error: e })
			}
		}
	}

	/// [xmpp_connect_component](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaa1cfa1189fdf64bb443c68f0590fd069)
	/// [xmpp_conn_handler](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#aad7c657ae239a87e2c2b746f99138e99)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_component<CB>(
		mut self,
		host: impl AsRef<str>,
		port: impl Into<Option<u16>>,
		handler: CB,
	) -> Result<Context<'cx, 'cb>, ConnectClientError<'cb, 'cx>>
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let host = FFI(host.as_ref()).send();
		let port: Nullable<_> = port.into().into();
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = self.make_fat_handler(Box::new(handler) as _, callback as _, ());
		let old_handler = self.fat_handlers.borrow_mut().connection.replace(new_handler);
		let out = unsafe {
			sys::xmpp_connect_component(
				self.inner.as_mut(),
				host.as_ptr(),
				port.val(),
				Some(callback),
				as_void_ptr(&self.fat_handlers.borrow().connection),
			)
		}
		.into_result();
		match out {
			Ok(_) => {
				let mut out = self.ctx.take().expect("Internal context is empty, it must never happen");
				out.consume_connection(self);
				Ok(out)
			}
			Err(e) => {
				self.fat_handlers.borrow_mut().connection = old_handler;
				Err(ConnectClientError { conn: self, error: e })
			}
		}
	}

	/// [xmpp_connect_raw](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga3873544638e8123c667f074d86dbad5a)
	/// [xmpp_conn_handler](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#aad7c657ae239a87e2c2b746f99138e99)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_raw<CB>(
		mut self,
		alt_host: Option<&str>,
		alt_port: impl Into<Option<u16>>,
		handler: CB,
	) -> Result<Context<'cx, 'cb>, ConnectClientError<'cb, 'cx>>
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if self.jid().is_none() {
			return Err(ConnectClientError {
				conn: self,
				error: Error::InvalidOperation,
			});
		}
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(self.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut self.fat_handlers.borrow_mut().connection, new_handler);
		let out = unsafe {
			sys::xmpp_connect_raw(
				self.inner.as_mut(),
				alt_host.as_ptr(),
				alt_port.val(),
				Some(callback),
				as_void_ptr(self.fat_handlers.borrow().connection.as_ref().unwrap()),
			)
		}
		.into_result();
		match out {
			Ok(_) => {
				let mut out = self.ctx.take().expect("Internal context is empty, it must never happen");
				out.consume_connection(self);
				Ok(out)
			}
			Err(e) => {
				self.fat_handlers.borrow_mut().connection = old_handler;
				Err(ConnectClientError { conn: self, error: e })
			}
		}
	}

	#[inline]
	/// [xmpp_conn_open_stream_default](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga73e477d4abfd439bcd27ddf78d601c0f)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn open_stream_default(&self) -> Result<()> {
		unsafe { sys::xmpp_conn_open_stream_default(self.inner.as_ptr()) }.into_result()
	}

	/// [xmpp_conn_open_stream](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga747589e1fdf44891c601958742d115b7)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn open_stream(&self, attributes: &HashMap<&str, &str>) -> Result<()> {
		let mut storage = Vec::with_capacity(attributes.len() * 2);
		for attr in attributes {
			storage.push(FFI(*attr.0).send());
			storage.push(FFI(*attr.1).send());
		}
		let mut attrs = storage.iter().map(|s| s.as_ptr() as *mut _).collect::<Vec<_>>();
		unsafe { sys::xmpp_conn_open_stream(self.inner.as_ptr(), attrs.as_mut_ptr(), attrs.len()) }.into_result()
	}

	#[inline]
	/// [xmpp_conn_tls_start](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga65a92215a59a365f89e908e90178f7b8)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn tls_start(&self) -> Result<()> {
		unsafe { sys::xmpp_conn_tls_start(self.inner.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_disconnect](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaa635ceddb5941d011e290073f7552355)
	pub fn disconnect(&mut self) {
		unsafe { sys::xmpp_disconnect(self.inner.as_mut()) }
	}

	#[inline]
	/// [xmpp_send_raw_string](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaf67110aced5d20909069d33d17bec025)
	///
	/// Be aware that this method performs a lot of allocations internally so you might want to use
	/// [`send_raw()`](#method.send_raw) instead.
	pub fn send_raw_string(&mut self, data: impl AsRef<str>) {
		let data = FFI(data.as_ref()).send();
		unsafe {
			sys::xmpp_send_raw_string(self.inner.as_mut(), data.as_ptr());
		}
	}

	/// [xmpp_send_raw](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#gaa1be7bdb58f3610b7997f1186d87c896)
	pub fn send_raw(&mut self, data: impl AsRef<[u8]>) {
		let data = data.as_ref();
		#[cfg(feature = "log")]
		if log::log_enabled!(log::Level::Debug) {
			use crate::LogLevel;
			use std::fmt::Write;

			let ctx = unsafe { sys::xmpp_conn_get_context(self.inner.as_ptr()) };
			let mut data_str = "SENT: ".to_owned();
			if let Ok(data) = str::from_utf8(data) {
				write!(&mut data_str, "{}", data).expect("Can't write to string");
			} else {
				write!(&mut data_str, "{:?}", data).expect("Can't write to string");
			}
			unsafe {
				crate::context::ctx_log(ctx, LogLevel::XMPP_LEVEL_DEBUG, "conn", &data_str);
			}
		}
		unsafe {
			sys::xmpp_send_raw(self.inner.as_mut(), data.as_ptr() as _, data.len());
		}
	}

	#[inline]
	/// [xmpp_send](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga0e879d34b2ea28c08cacbb012eadfbc1)
	pub fn send(&mut self, stanza: &Stanza) {
		unsafe { sys::xmpp_send(self.inner.as_mut(), stanza.as_ptr()) }
	}

	/// [xmpp_timed_handler_add](https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html#ga5835cd8c81174d06d35953e8b13edccb)
	/// [xmpp_timed_handler](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#a94af0b39027071eca8c16e9891314bb4)
	///
	/// See [Connection::handler_add] for additional information.
	pub fn timed_handler_add<CB>(&mut self, handler: CB, period: Duration) -> Option<TimedHandlerId<'cb, 'cx, CB>>
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb,
	{
		let callback = Self::timed_handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, ());
		let fat_handlers = Rc::clone(&self.fat_handlers);
		let timed_handlers = &mut fat_handlers.borrow_mut().timed;
		Self::store_fat_handler(timed_handlers, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_timed_handler_add(
					self.inner.as_mut(),
					Some(callback),
					period.as_millis() as c_ulong,
					fat_handler_ptr as _,
				);
			}
			TimedHandlerId(fat_handler_ptr as _)
		})
	}

	/// [xmpp_timed_handler_delete](https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html#gadbc8e82d9d3ee6ab4166ce4dba0ea8dd)
	///
	/// See [Connection::handler_delete] for additional information.
	pub fn timed_handler_delete<CB>(&mut self, handler_id: TimedHandlerId<CB>)
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb,
	{
		#![allow(clippy::needless_pass_by_value)]
		unsafe { sys::xmpp_timed_handler_delete(self.inner.as_mut(), Some(Self::timed_handler_cb::<CB>)) }
		Self::drop_fat_handler(&mut self.fat_handlers.borrow_mut().timed, handler_id.0 as _);
	}

	/// See [Connection::handlers_clear] for additional information.
	pub fn timed_handlers_clear(&mut self) {
		for handler in self.fat_handlers.borrow_mut().timed.drain(..) {
			unsafe { sys::xmpp_timed_handler_delete(self.inner.as_mut(), Some(mem::transmute(handler.cb_addr))) };
		}
		self.fat_handlers.borrow_mut().timed.shrink_to_fit();
	}

	/// [xmpp_id_handler_add](https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html#gafaa44ec48db44b45c5d240c7df4bfaac)
	/// [xmpp_handler](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#a079ae14399be93d363164ad35d434496)
	///
	/// See [Connection::handler_add] for additional information.
	pub fn id_handler_add<CB>(&mut self, handler: CB, id: impl Into<String>) -> Option<IdHandlerId<'cb, 'cx, CB>>
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
	{
		let id = id.into();
		let ffi_id = FFI(id.as_str()).send();
		let callback = Self::handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, Some(id));
		let fat_handlers = Rc::clone(&self.fat_handlers);
		let stanza_handlers = &mut fat_handlers.borrow_mut().stanza;
		Self::store_fat_handler(stanza_handlers, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_id_handler_add(self.inner.as_mut(), Some(callback), ffi_id.as_ptr(), fat_handler_ptr as _);
			}
			IdHandlerId(fat_handler_ptr as _)
		})
	}

	/// [xmpp_id_handler_delete](https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html#gaee081149b7c6889b6b692a44b407d42d)
	///
	/// See [Connection::handler_delete] for additional information.
	pub fn id_handler_delete<CB>(&mut self, handler_id: IdHandlerId<CB>)
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
	{
		#![allow(clippy::needless_pass_by_value)]
		if let Some(fat_handler) = Self::validate_fat_handler(&self.fat_handlers.borrow().stanza, handler_id.0 as _) {
			let id = FFI(fat_handler.extra.as_ref().unwrap().as_str()).send();
			unsafe { sys::xmpp_id_handler_delete(self.inner.as_mut(), Some(Self::handler_cb::<CB>), id.as_ptr()) }
		}
		Self::drop_fat_handler(&mut self.fat_handlers.borrow_mut().stanza, handler_id.0 as _);
	}

	/// See [Connection::handlers_clear] for additional information.
	pub fn id_handlers_clear(&mut self) {
		self.fat_handlers.borrow_mut().stanza.retain(|x| {
			if let Some(ref id) = x.extra {
				unsafe {
					sys::xmpp_id_handler_delete(
						self.inner.as_ptr(),
						Some(mem::transmute(x.cb_addr)),
						FFI(id.as_str()).send().as_ptr(),
					)
				};
				false
			} else {
				true
			}
		});
		self.fat_handlers.borrow_mut().stanza.shrink_to_fit();
	}

	/// [xmpp_handler_add](https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html#ga73235438899b51d265c1d35915c5cd7c)
	/// [xmpp_handler](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#a079ae14399be93d363164ad35d434496)
	///
	/// This function returns [HandlerId] which is later can be used to remove the handler using [Connection::handler_delete].
	pub fn handler_add<CB>(
		&mut self,
		handler: CB,
		ns: Option<&str>,
		name: Option<&str>,
		typ: Option<&str>,
	) -> Option<HandlerId<'cb, 'cx, CB>>
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
	{
		let ns = FFI(ns).send();
		let name = FFI(name).send();
		let typ = FFI(typ).send();
		let callback = Self::handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, None);
		let fat_handlers = Rc::clone(&self.fat_handlers);
		let stanza_handlers = &mut fat_handlers.borrow_mut().stanza;
		Self::store_fat_handler(stanza_handlers, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_handler_add(
					self.inner.as_mut(),
					Some(callback),
					ns.as_ptr(),
					name.as_ptr(),
					typ.as_ptr(),
					fat_handler_ptr as _,
				)
			}
			HandlerId(fat_handler_ptr as _)
		})
	}

	/// [xmpp_handler_delete](https://strophe.im/libstrophe/doc/0.12.2/group___handlers.html#gaf4fa6f67b11dee0158739c907ba71adb)
	///
	/// This version of this function accepts `HandlerId` returned from `add_handler()` function instead of function reference as the underlying
	/// library does. If you can't keep track of those handles, but still want ability to remove handlers, check `handlers_clear()` function.
	pub fn handler_delete<CB>(&mut self, handler_id: HandlerId<CB>)
	where
		CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
	{
		#![allow(clippy::needless_pass_by_value)]
		unsafe { sys::xmpp_handler_delete(self.inner.as_mut(), Some(Self::handler_cb::<CB>)) }
		Self::drop_fat_handler(&mut self.fat_handlers.borrow_mut().stanza, handler_id.0 as _);
	}

	/// Removes all handlers that were set up with `handler_add()`. This function does *not* remove handlers added via `id_handler_add()`. You can use
	/// this function if you can't keep track of specific closure handles returned from `handler_add()`, but want to remove handlers anyway.
	pub fn handlers_clear(&mut self) {
		self.fat_handlers.borrow_mut().stanza.retain(|x| {
			if x.extra.is_none() {
				unsafe { sys::xmpp_handler_delete(self.inner.as_ptr(), Some(mem::transmute(x.cb_addr))) };
				false
			} else {
				true
			}
		});
		self.fat_handlers.borrow_mut().stanza.shrink_to_fit();
	}

	#[allow(dead_code)]
	pub(crate) fn timed_handlers_same<L, R>(_left: L, _right: R) -> bool
	where
		L: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb,
		R: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> HandlerResult + Send + 'cb,
	{
		ptr::eq(
			Self::timed_handler_cb::<L> as *const (),
			Self::timed_handler_cb::<R> as *const (),
		)
	}

	#[allow(dead_code)]
	pub(crate) fn stanza_handlers_same<L, R>(_left: L, _right: R) -> bool
	where
		L: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
		R: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> HandlerResult + Send + 'cb,
	{
		ptr::eq(Self::handler_cb::<L> as *const (), Self::handler_cb::<R> as *const ())
	}

	#[allow(dead_code)]
	pub(crate) fn connection_handlers_same<L, R>(_left: L, _right: R) -> bool
	where
		L: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
		R: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		ptr::eq(
			Self::connection_handler_cb::<L> as *const (),
			Self::connection_handler_cb::<R> as *const (),
		)
	}
}

impl PartialEq for Connection<'_, '_> {
	fn eq(&self, other: &Connection) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Connection<'_, '_> {}

impl Drop for Connection<'_, '_> {
	/// [xmpp_conn_release](https://strophe.im/libstrophe/doc/0.12.2/group___connections.html#ga87b076b11589bc23123096dc83cde6a8)
	fn drop(&mut self) {
		if self.owned {
			#[cfg(feature = "libstrophe-0_11_0")]
			if let Ok(mut handlers) = CERT_FAIL_HANDLERS.write() {
				if let Some(handler_id) = self.fat_handlers.borrow_mut().cert_fail_handler_id.take() {
					handlers.remove(&handler_id);
				}
			}
			#[cfg(feature = "libstrophe-0_12_0")]
			if let Ok(mut handlers) = SOCKOPT_HANDLERS.write() {
				if let Some(handler_id) = self.fat_handlers.borrow_mut().sockopt_handler_id.take() {
					handlers.remove(&handler_id);
				}
			}
			unsafe {
				sys::xmpp_conn_release(self.inner.as_mut());
			}
		}
	}
}

unsafe impl Send for Connection<'_, '_> {}

pub struct HandlerId<'cb, 'cx, CB>(*const FatHandler<'cb, 'cx, CB, ()>);

impl<CB> fmt::Debug for HandlerId<'_, '_, CB> {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		write!(f, "{:?}", self.0)
	}
}

pub struct TimedHandlerId<'cb, 'cx, CB>(*const FatHandler<'cb, 'cx, CB, ()>);

impl<CB> fmt::Debug for TimedHandlerId<'_, '_, CB> {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		write!(f, "{:?}", self.0)
	}
}

pub struct IdHandlerId<'cb, 'cx, CB>(*const FatHandler<'cb, 'cx, CB, Option<String>>);

impl<CB> fmt::Debug for IdHandlerId<'_, '_, CB> {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		write!(f, "{:?}", self.0)
	}
}

#[derive(Debug)]
pub enum ConnectionEvent<'t, 's> {
	RawConnect,
	Connect,
	Disconnect(Option<ConnectionError<'t, 's>>),
	//	Fail(ConnectionError<'t, 's>), // never actually used in the underlying library
}

impl fmt::Display for ConnectionEvent<'_, '_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ConnectionEvent::RawConnect => write!(f, "Raw connect"),
			ConnectionEvent::Connect => write!(f, "Connect"),
			ConnectionEvent::Disconnect(None) => write!(f, "Disconnect"),
			ConnectionEvent::Disconnect(Some(e)) => write!(f, "Disconnect, error: {}", e),
		}
	}
}

#[test]
fn callbacks() {
	{
		let a = |_: &Context, _: &mut Connection| {
			print!("1");
			HandlerResult::KeepHandler
		};
		let b = |_: &Context, _: &mut Connection| {
			print!("2");
			HandlerResult::RemoveHandler
		};

		assert!(Connection::timed_handlers_same(a, a));
		assert!(!Connection::timed_handlers_same(a, b));
	}

	{
		let a = |_: &Context, _: &mut Connection, _: &Stanza| {
			print!("1");
			HandlerResult::KeepHandler
		};
		let b = |_: &Context, _: &mut Connection, _: &Stanza| {
			print!("2");
			HandlerResult::KeepHandler
		};

		assert!(Connection::stanza_handlers_same(a, a));
		assert!(!Connection::stanza_handlers_same(a, b));
	}

	{
		let a = |_: &Context, _: &mut Connection, _: ConnectionEvent| print!("1");
		let b = |_: &Context, _: &mut Connection, _: ConnectionEvent| print!("2");

		assert!(Connection::connection_handlers_same(a, a));
		assert!(!Connection::connection_handlers_same(a, b));
	}
}
