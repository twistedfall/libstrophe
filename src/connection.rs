use std::{
	cell::RefCell,
	collections,
	fmt,
	fmt::Write,
	mem,
	os::raw,
	ptr::NonNull,
	rc::{
		Rc,
		Weak,
	},
	result,
	str,
	time::Duration,
};

use crate::{
	as_void_ptr,
	ConnectClientError,
	ConnectionError,
	ConnectionFlags,
	Context,
	Error,
	error::IntoResult,
	FFI,
	ffi_types::Nullable,
	LogLevel,
	Result,
	Stanza,
	StreamError,
	void_ptr_as,
};

type ConnectionCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb;
type ConnectionFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, ConnectionCallback<'cb, 'cx>, ()>;

type Handlers<H> = Vec<Box<H>>;

type TimedCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> bool + Send + 'cb;
type TimedFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, TimedCallback<'cb, 'cx>, ()>;

type StanzaCallback<'cb, 'cx> = dyn FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> bool + Send + 'cb;
type StanzaFatHandler<'cb, 'cx> = FatHandler<'cb, 'cx, StanzaCallback<'cb, 'cx>, Option<String>>;

struct FatHandlers<'cb, 'cx> {
	connection: Option<ConnectionFatHandler<'cb, 'cx>>,
	timed: Handlers<TimedFatHandler<'cb, 'cx>>,
	stanza: Handlers<StanzaFatHandler<'cb, 'cx>>,
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
/// [connection]: http://strophe.im/libstrophe/doc/0.9.2/group___connections.html
/// [handlers]: http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html
/// [conn.c]: https://github.com/strophe/libstrophe/blob/0.10.0/src/conn.c
/// [handler.c]: https://github.com/strophe/libstrophe/blob/0.10.0/src/handler.c
/// [xmpp_conn_release]: http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga16967e3375efa5032ed2e08b407d8ae9
#[derive(Debug)]
pub struct Connection<'cb, 'cx> {
	inner: NonNull<sys::xmpp_conn_t>,
	ctx: Option<Context<'cx, 'cb>>,
	owned: bool,
	fat_handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>,
}

impl<'cb, 'cx> Connection<'cb, 'cx> {
	/// [xmpp_conn_new](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga76c161884c23f69cd1d7fc025122cf21)
	pub fn new(ctx: Context<'cx, 'cb>) -> Self {
		unsafe {
			Self::from_inner(sys::xmpp_conn_new(ctx.as_inner()), ctx, Rc::new(RefCell::new(FatHandlers {
				connection: None,
				timed: Vec::with_capacity(4),
				stanza: Vec::with_capacity(4),
			})))
		}
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_conn_t, ctx: Context<'cx, 'cb>, owned: bool, handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>) -> Self {
		Connection { inner: NonNull::new(inner).expect("Cannot allocate memory for Connection"), ctx: Some(ctx), owned, fat_handlers: handlers }
	}

	unsafe fn from_inner(inner: *mut sys::xmpp_conn_t, ctx: Context<'cx, 'cb>, handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>) -> Self {
		Self::with_inner(
			inner,
			ctx,
			true,
			handlers,
		)
	}

	unsafe fn from_inner_ref_mut(inner: *mut sys::xmpp_conn_t, handlers: Rc<RefCell<FatHandlers<'cb, 'cx>>>) -> Self {
		let ctx = Context::from_inner_ref(sys::xmpp_conn_get_context(inner));
		Self::with_inner(inner, ctx, false, handlers)
	}

	unsafe extern "C" fn connection_handler_cb<CB>(conn: *mut sys::xmpp_conn_t, event: sys::xmpp_conn_event_t, error: raw::c_int,
	                                        stream_error: *mut sys::xmpp_stream_error_t, userdata: *mut raw::c_void) {
		ensure_unique!(CB);
		let connection_handler = void_ptr_as::<ConnectionFatHandler>(userdata);
		if let Some(fat_handlers) = connection_handler.fat_handlers.upgrade() {
			let mut conn = Self::from_inner_ref_mut(conn, fat_handlers);
			let event = match event {
				sys::xmpp_conn_event_t::XMPP_CONN_RAW_CONNECT => ConnectionEvent::RawConnect,
				sys::xmpp_conn_event_t::XMPP_CONN_CONNECT => ConnectionEvent::Connect,
				sys::xmpp_conn_event_t::XMPP_CONN_DISCONNECT => {
					let stream_error: Option<StreamError> = stream_error.as_ref().map(|e| e.into());
					ConnectionEvent::Disconnect(ConnectionError::from((error, stream_error)))
				},
				sys::xmpp_conn_event_t::XMPP_CONN_FAIL => unreachable!("XMPP_CONN_FAIL is never used in the underlying library"),
			};
			(connection_handler.handler)(conn.context_detached(), &mut conn, event);
		}
	}

	unsafe extern "C" fn timed_handler_cb<CB>(conn: *mut sys::xmpp_conn_t, userdata: *mut raw::c_void) -> i32 {
		ensure_unique!(CB);
		let timed_handler = void_ptr_as::<TimedFatHandler>(userdata);
		if let Some(fat_handlers) = timed_handler.fat_handlers.upgrade() {
			let mut conn = Self::from_inner_ref_mut(conn, fat_handlers);
			let res = (timed_handler.handler)(conn.context_detached(), &mut conn);
			if !res {
				Self::drop_fat_handler(&mut conn.fat_handlers.borrow_mut().timed, timed_handler);
			}
			res as _
		} else {
			0
		}
	}

	unsafe extern "C" fn handler_cb<CB>(conn: *mut sys::xmpp_conn_t, stanza: *mut sys::xmpp_stanza_t, userdata: *mut raw::c_void) -> i32 {
		ensure_unique!(CB);
		let stanza_handler = void_ptr_as::<StanzaFatHandler>(userdata);
		if let Some(fat_handlers) = stanza_handler.fat_handlers.upgrade() {
			let mut conn = Self::from_inner_ref_mut(conn, fat_handlers);
			let stanza = Stanza::from_inner_ref(stanza);
			let res = (stanza_handler.handler)(conn.context_detached(), &mut conn, &stanza);
			if !res {
				Self::drop_fat_handler(&mut conn.fat_handlers.borrow_mut().stanza, stanza_handler);
			}
			res as _
		} else {
			0
		}
	}

	fn store_fat_handler<CB: ?Sized, T>(fat_handlers: &mut Handlers<FatHandler<'cb, 'cx, CB, T>>, fat_handler: FatHandler<'cb, 'cx, CB, T>) -> Option<*const FatHandler<'cb, 'cx, CB, T>> {
		if Self::get_fat_handler_pos_by_callback(fat_handlers, fat_handler.cb_addr).is_none() {
			let handler = Box::new(fat_handler);
			let out = &*handler as _;
			fat_handlers.push(handler);
			Some(out)
		} else {
			None
		}
	}

	fn get_fat_handler_pos<CB: ?Sized, T>(fat_handlers: &Handlers<FatHandler<'cb, 'cx, CB, T>>, fat_handler_ptr: *const FatHandler<'cb, 'cx, CB, T>) -> Option<usize> {
		#![allow(clippy::ptr_arg)]
		fat_handlers.iter().position(|x| fat_handler_ptr == x.as_ref())
	}

	fn get_fat_handler_pos_by_callback<CB: ?Sized, T>(fat_handlers: &Handlers<FatHandler<'cb, 'cx, CB, T>>, cb_addr: *const ()) -> Option<usize> {
		#![allow(clippy::ptr_arg)]
		fat_handlers.iter().position(|x| cb_addr == x.cb_addr)
	}

	fn validate_fat_handler<'f, CB: ?Sized, T>(fat_handlers: &'f Handlers<FatHandler<'cb, 'cx, CB, T>>, fat_handler_ptr: *const FatHandler<'cb, 'cx, CB, T>) -> Option<&'f FatHandler<'cb, 'cx, CB, T>> {
		#![allow(clippy::ptr_arg)]
		Self::get_fat_handler_pos(fat_handlers, fat_handler_ptr).map(|pos| {
			fat_handlers[pos].as_ref()
		})
	}

	fn drop_fat_handler<CB: ?Sized, T>(fat_handlers: &mut Handlers<FatHandler<'cb, 'cx, CB, T>>, fat_handler_ptr: *const FatHandler<'cb, 'cx, CB, T>) -> Option<usize> {
		if let Some(pos) = Self::get_fat_handler_pos(fat_handlers, fat_handler_ptr) {
			fat_handlers.remove(pos);
			Some(pos)
		} else {
			None
		}
	}

	fn make_fat_handler<CB: ?Sized, T>(&self, handler: Box<CB>, cb_addr: *const (), extra: T) -> FatHandler<'cb, 'cx, CB, T> {
		FatHandler {
			fat_handlers: Rc::downgrade(&self.fat_handlers),
			handler,
			cb_addr,
			extra
		}
	}

	unsafe fn context_detached<'a>(&self) -> &'a Context<'cx, 'cb> {
		(self.ctx.as_ref().unwrap() as *const Context).as_ref().unwrap()
	}

	/// [xmpp_conn_get_flags](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaa9724ae412b01562dc81c31fd178d2f3)
	pub fn flags(&self) -> ConnectionFlags {
		ConnectionFlags::from_bits(unsafe { sys::xmpp_conn_get_flags(self.inner.as_ptr()) }).unwrap()
	}

	/// [xmpp_conn_set_flags](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga92761a101e721df9b923e9c35f6ad949)
	pub fn set_flags(&mut self, flags: ConnectionFlags) -> Result<()> {
		unsafe {
			sys::xmpp_conn_set_flags(self.inner.as_mut(), flags.bits())
		}.into_result()
	}

	/// [xmpp_conn_get_jid](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga52b5fee898fc6ef06ba1ea7f9a507a39)
	pub fn jid(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_jid(self.inner.as_ptr())).receive()
		}
	}

	/// [xmpp_conn_get_bound_jid](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga9bc4f0527c6aa7ac4d5b112be6189889)
	pub fn bound_jid(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_bound_jid(self.inner.as_ptr())).receive()
		}
	}

	/// [xmpp_conn_set_jid](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga8dff6d97ac458d5f3fc901d688d86084)
	pub fn set_jid(&mut self, jid: impl AsRef<str>) {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			sys::xmpp_conn_set_jid(self.inner.as_mut(), jid.as_ptr())
		}
	}

	/// [xmpp_conn_get_pass](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaa5e1ce97c2d8ad50380b92b2ca204dec)
	pub fn pass(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_pass(self.inner.as_ptr())).receive()
		}
	}

	/// [xmpp_conn_set_pass](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gac64373b712d9d8af12e57b753d9b3bfc)
	pub fn set_pass(&mut self, pass: impl AsRef<str>) {
		let pass = FFI(pass.as_ref()).send();
		unsafe {
			sys::xmpp_conn_set_pass(self.inner.as_mut(), pass.as_ptr())
		}
	}

	/// [xmpp_conn_disable_tls](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga1730868abcec1c6c63f2351bb81a43ac)
	pub fn disable_tls(&mut self) {
		unsafe {
			sys::xmpp_conn_disable_tls(self.inner.as_mut())
		}
	}

	/// [xmpp_conn_is_secured](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga331bfca7c9c9ce3a17c909e770a73b02)
	pub fn is_secured(&self) -> bool {
		unsafe {
			FFI(sys::xmpp_conn_is_secured(self.inner.as_ptr())).receive_bool()
		}
	}

	/// [xmpp_conn_set_keepalive](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga0c75095f31ee66febcf71cad1e60d4f6)
	pub fn set_keepalive(&mut self, timeout: Duration, interval: Duration) {
		unsafe {
			sys::xmpp_conn_set_keepalive(self.inner.as_mut(), timeout.as_secs() as _, interval.as_secs() as _)
		}
	}

	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_conn_is_connecting](https://github.com/strophe/libstrophe/blob/0.10.0/src/conn.c#L1035-L1039)
	pub fn is_connecting(&self) -> bool {
		unsafe {
			FFI(sys::xmpp_conn_is_connecting(self.inner.as_ptr())).receive_bool()
		}
	}

	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_conn_is_connected](https://github.com/strophe/libstrophe/blob/0.10.0/src/conn.c#L1045-L1049)
	pub fn is_connected(&self) -> bool {
		unsafe {
			FFI(sys::xmpp_conn_is_connected(self.inner.as_ptr())).receive_bool()
		}
	}

	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_conn_is_disconnected](https://github.com/strophe/libstrophe/blob/0.10.0/src/conn.c#L1055-L1059)
	pub fn is_disconnected(&self) -> bool {
		unsafe {
			FFI(sys::xmpp_conn_is_disconnected(self.inner.as_ptr())).receive_bool()
		}
	}

	/// [xmpp_connect_client](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaf81281d31f398cfac039f32429061c03)
	///
	/// Pre-last `error: i32` argument in the handler contains a TLS error code that can be passed
	/// together with `XMPP_CONN_DISCONNECT` event. The specific meaning if that code depends on the
	/// underlying TLS implementation:
	///
	///   * for `openssl` it's the result of [`SSL_get_error()`]
	///   * for `schannel` it's the result of [`WSAGetLastError()`]
	///
	/// Additionally the following OS dependent error constants can be set for `error`:
	///
	///   * `ETIMEDOUT`/`WSAETIMEDOUT`
	///   * `ECONNRESET`/`WSAECONNRESET`
	///   * `ECONNABORTED`/`WSAECONNABORTED`
	///
	/// [`SSL_get_error()`]: https://www.openssl.org/docs/manmaster/man3/SSL_get_error.html#RETURN-VALUES
	/// [`WSAGetLastError()`]: https://docs.microsoft.com/en-us/windows/desktop/api/winsock/nf-winsock-wsagetlasterror#return-value
	pub fn connect_client<CB>(self, alt_host: Option<&str>, alt_port: impl Into<Option<u16>>, handler: CB) -> Result<Context<'cx, 'cb>, ConnectClientError<'cb, 'cx>>
		where
			CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let mut me = self; // hack to not expose mutability via function interface
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if me.jid().is_none() {
			return Err(ConnectClientError {
				conn: me,
				error: Error::InvalidOperation,
			});
		}
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(me.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut me.fat_handlers.borrow_mut().connection, new_handler);
		let out = unsafe {
			sys::xmpp_connect_client(
				me.inner.as_mut(),
				alt_host.as_ptr(),
				alt_port.val(),
				Some(callback),
				as_void_ptr(me.fat_handlers.borrow().connection.as_ref().unwrap()),
			)
		}.into_result();
		match out {
			Ok(_) => {
				let mut out = me.ctx.take().expect("Internal context is empty, it must never happen");
				out.consume_connection(me);
				Ok(out)
			},
			Err(e) => {
				me.fat_handlers.borrow_mut().connection = old_handler;
				Err(ConnectClientError {
					conn: me,
					error: e,
				})
			}
		}
	}

	/// [xmpp_connect_component](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga80c8cd7906a48fc27664fcce8f15ed7d)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_component<CB>(self, host: impl AsRef<str>, port: impl Into<Option<u16>>, handler: CB) -> Result<Context<'cx, 'cb>, ConnectClientError<'cb, 'cx>>
		where
			CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let mut me = self; // hack to not expose mutability via function interface
		let host = FFI(host.as_ref()).send();
		let port: Nullable<_> = port.into().into();
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(me.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut me.fat_handlers.borrow_mut().connection, new_handler);
		let out = unsafe {
			sys::xmpp_connect_component(
				me.inner.as_mut(),
				host.as_ptr(),
				port.val(),
				Some(callback),
				as_void_ptr(&me.fat_handlers.borrow().connection),
			)
		}.into_result();
		match out {
			Ok(_) => {
				let mut out = me.ctx.take().expect("Internal context is empty, it must never happen");
				out.consume_connection(me);
				Ok(out)
			},
			Err(e) => {
				me.fat_handlers.borrow_mut().connection = old_handler;
				Err(ConnectClientError {
					conn: me,
					error: e,
				})
			}
		}
	}

	/// [xmpp_connect_raw](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gae64b7a2ec8e138a1501bb7bf12089776)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_raw<CB>(self, alt_host: Option<&str>, alt_port: impl Into<Option<u16>>, handler: CB) -> Result<Context<'cx, 'cb>, ConnectClientError<'cb, 'cx>>
		where
			CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, ConnectionEvent) + Send + 'cb,
	{
		let mut me = self; // hack to not expose mutability via function interface
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if me.jid().is_none() {
			return Err(ConnectClientError {
				conn: me,
				error: Error::InvalidOperation,
			});
		}
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(me.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut me.fat_handlers.borrow_mut().connection, new_handler);
		let out = unsafe {
			sys::xmpp_connect_raw(
				me.inner.as_mut(),
				alt_host.as_ptr(),
				alt_port.val(),
				Some(callback),
				as_void_ptr(me.fat_handlers.borrow().connection.as_ref().unwrap()),
			)
		}.into_result();
		match out {
			Ok(_) => {
				let mut out = me.ctx.take().expect("Internal context is empty, it must never happen");
				out.consume_connection(me);
				Ok(out)
			},
			Err(e) => {
				me.fat_handlers.borrow_mut().connection = old_handler;
				Err(ConnectClientError {
					conn: me,
					error: e,
				})
			}
		}
	}

	/// [xmpp_conn_open_stream_default](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaac72cb61c7a69499fd1387c1d499c08e)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn open_stream_default(&self) -> Result<()> {
		unsafe {
			sys::xmpp_conn_open_stream_default(self.inner.as_ptr())
		}.into_result()
	}

	/// [xmpp_conn_open_stream](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga85ccb1c2d95caf29dff0c9b70424c53e)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn open_stream(&self, attributes: &collections::HashMap<&str, &str>) -> Result<()> {
		let mut storage = Vec::with_capacity(attributes.len() * 2);
		let mut attrs = Vec::with_capacity(attributes.len() * 2);
		for attr in attributes {
			storage.push(FFI(*attr.0).send());
			storage.push(FFI(*attr.1).send());
			attrs.push(storage[storage.len() - 2].as_ptr() as _);
			attrs.push(storage[storage.len() - 1].as_ptr() as _);
		}
		unsafe {
			sys::xmpp_conn_open_stream(
				self.inner.as_ptr(),
				attrs.as_mut_ptr(),
				attrs.len(),
			)
		}.into_result()
	}

	/// [xmpp_conn_tls_start](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga759b1ae6fd1e40335afd8acc26d3858f)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn tls_start(&self) -> Result<()> {
		unsafe {
			sys::xmpp_conn_tls_start(self.inner.as_ptr())
		}.into_result()
	}

	/// [xmpp_disconnect](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga809ee4c8bb95e86ec2119db1052849ce)
	pub fn disconnect(&mut self) {
		unsafe {
			sys::xmpp_disconnect(self.inner.as_mut())
		}
	}

	/// [xmpp_send_raw_string](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga606ee8f53d8992941d93ecf27e41595b)
	///
	/// Be aware that this method performs a lot of allocations internally so you might want to use
	/// [`send_raw()`](#method.send_raw) instead.
	pub fn send_raw_string(&mut self, data: impl AsRef<str>) {
		let data = FFI(data.as_ref()).send();
		unsafe {
			sys::xmpp_send_raw_string(self.inner.as_mut(), data.as_ptr());
		}
	}

	/// [xmpp_send_raw](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gadd1c8707fa269e6d6845d6b856584add)
	pub fn send_raw(&mut self, data: impl AsRef<[u8]>) {
		let data = data.as_ref();
		if log::log_enabled!(log::Level::Debug) {
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

	/// [xmpp_send](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gac064b81b22a3166d5b396b5aa8e40b7d)
	pub fn send(&mut self, stanza: &Stanza) { unsafe { sys::xmpp_send(self.inner.as_mut(), stanza.as_inner()) } }

	/// [xmpp_timed_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga0a74b20f2367389e5dc8852b4d3fdcda)
	///
	/// See `handler_add()` for additional information.
	pub fn timed_handler_add<CB>(&mut self, handler: CB, period: Duration) -> Option<TimedHandlerId<'cb, 'cx, CB>>
		where
			CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>) -> bool + Send + 'cb,
	{
		let callback = Self::timed_handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, ());
		Self::store_fat_handler(&mut self.fat_handlers.borrow_mut().timed, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_timed_handler_add(
					self.inner.as_ptr(),
					Some(callback),
					period.as_millis() as raw::c_ulong,
					fat_handler_ptr as _,
				);
			}
			TimedHandlerId(fat_handler_ptr as _)
		})
	}

	/// [xmpp_timed_handler_delete](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#gae70f02f84a0f232c6a8c2866ecb47b82)
	///
	/// See `handler_delete()` for additional information.
	pub fn timed_handler_delete<CB>(&mut self, handler_id: TimedHandlerId<CB>) {
		#![allow(clippy::needless_pass_by_value)]
		unsafe {
			sys::xmpp_timed_handler_delete(self.inner.as_mut(), Some(Self::timed_handler_cb::<CB>))
		}
		Self::drop_fat_handler(&mut self.fat_handlers.borrow_mut().timed, handler_id.0 as _);
	}

	/// See `handlers_clear()` for additional information.
	pub fn timed_handlers_clear(&mut self) {
		for handler in self.fat_handlers.borrow_mut().timed.drain(..) {
			unsafe { sys::xmpp_timed_handler_delete(self.inner.as_mut(), Some(mem::transmute(handler.cb_addr))) };
		}
		self.fat_handlers.borrow_mut().timed.shrink_to_fit();
	}

	/// [xmpp_id_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga2142d78b7d6e8d278eebfa8a63f194a4)
	///
	/// See `handler_add()` for additional information.
	pub fn id_handler_add<CB>(&mut self, handler: CB, id: impl Into<String>) -> Option<IdHandlerId<'cb, 'cx, CB>>
		where
			CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> bool + Send + 'cb,
	{
		let id = id.into();
		let ffi_id = FFI(id.as_str()).send();
		let callback = Self::handler_cb::<CB>;
		let handler = self.make_fat_handler(
			Box::new(handler) as _,
			callback as _,
			Some(id),
		);
		Self::store_fat_handler(&mut self.fat_handlers.borrow_mut().stanza, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_id_handler_add(
					self.inner.as_ptr(),
					Some(callback),
					ffi_id.as_ptr(),
					fat_handler_ptr as _,
				);
			}
			IdHandlerId(fat_handler_ptr as _)
		})
	}

	/// [xmpp_id_handler_delete](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga6bd02f7254b2a53214824d3d5e4f59ce)
	///
	/// See `handler_delete()` for additional information.
	pub fn id_handler_delete<CB>(&mut self, handler_id: IdHandlerId<CB>) {
		#![allow(clippy::needless_pass_by_value)]
		if let Some(fat_handler) = Self::validate_fat_handler(&self.fat_handlers.borrow().stanza, handler_id.0 as _) {
			let id = FFI(fat_handler.extra.as_ref().unwrap().as_str()).send();
			unsafe {
				sys::xmpp_id_handler_delete(self.inner.as_mut(), Some(Self::handler_cb::<CB>), id.as_ptr())
			}
		}
		Self::drop_fat_handler(&mut self.fat_handlers.borrow_mut().stanza, handler_id.0 as _);
	}

	/// See `handlers_clear()` for additional information.
	pub fn id_handlers_clear(&mut self) {
		self.fat_handlers.borrow_mut().stanza.retain(|x| {
			if let Some(ref id) = x.extra {
				unsafe { sys::xmpp_id_handler_delete(self.inner.as_ptr(), Some(mem::transmute(x.cb_addr)), FFI(id.as_str()).send().as_ptr()) };
				false
			} else {
				true
			}
		});
		self.fat_handlers.borrow_mut().stanza.shrink_to_fit();
	}

	/// [xmpp_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#gad307e5a22d16ef3d6fa18d503b68944f)
	///
	/// This function returns `HandlerId` which is later can be used to remove the handler using `handler_delete()`.
	pub fn handler_add<CB>(&mut self, handler: CB, ns: Option<&str>, name: Option<&str>, typ: Option<&str>) -> Option<HandlerId<'cb, 'cx, CB>>
		where
			CB: FnMut(&Context<'cx, 'cb>, &mut Connection<'cb, 'cx>, &Stanza) -> bool + Send + 'cb,
	{
		let ns = FFI(ns).send();
		let name = FFI(name).send();
		let typ = FFI(typ).send();
		let callback = Self::handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, None);
		Self::store_fat_handler(&mut self.fat_handlers.borrow_mut().stanza, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_handler_add(
					self.inner.as_ptr(),
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

	/// [xmpp_handler_delete](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga881756ae98f74748212bcbc61e6a4c89)
	///
	/// This version of this function accepts `HandlerId` returned from `add_handler()` function instead of function reference as the underlying
	/// library does. If you can't keep track of those handles, but still want ability to remove handlers, check `handlers_clear()` function.
	pub fn handler_delete<CB>(&mut self, handler_id: HandlerId<CB>) {
		#![allow(clippy::needless_pass_by_value)]
		unsafe {
			sys::xmpp_handler_delete(self.inner.as_mut(), Some(Self::handler_cb::<CB>))
		}
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
}

impl PartialEq for Connection<'_, '_> {
	fn eq(&self, other: &Connection) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Connection<'_, '_> {}

impl Drop for Connection<'_, '_> {
	/// [xmpp_conn_release](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga16967e3375efa5032ed2e08b407d8ae9)
	fn drop(&mut self) {
		if self.owned {
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

pub struct FatHandler<'cb, 'cx, CB: ?Sized, T> {
	fat_handlers: Weak<RefCell<FatHandlers<'cb, 'cx>>>,
	handler: Box<CB>,
	cb_addr: *const (),
	extra: T,
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
		fn timed_eq<L, R>(_left: L, _right: R) -> bool {
			let ptr_left = Connection::timed_handler_cb::<L> as *const ();
			let ptr_right = Connection::timed_handler_cb::<R> as *const ();
			ptr_left == ptr_right
		}

		let a = |_: &Context, _: &mut Connection| { true };
		let b = |_: &Context, _: &mut Connection| { true };

		assert!(timed_eq(a, a));
		assert!(!timed_eq(a, b));
	}

	{
		fn handler_eq<L, R>(_left: L, _right: R) -> bool {
			let ptr_left = Connection::handler_cb::<L> as *const ();
			let ptr_right = Connection::handler_cb::<R> as *const ();
			ptr_left == ptr_right
		}

		let a = |_: &Context, _: &mut Connection| { true };
		let b = |_: &Context, _: &mut Connection| { true };

		assert!(handler_eq(a, a));
		assert!(!handler_eq(a, b));
	}

	{
		fn connection_eq<L, R>(_left: L, _right: R) -> bool {
			let ptr_left = Connection::connection_handler_cb::<L> as *const ();
			let ptr_right = Connection::connection_handler_cb::<R> as *const ();
			ptr_left == ptr_right
		}

		let a = |_: &Context, _: &mut Connection, _: ConnectionEvent| {};
		let b = |_: &Context, _: &mut Connection, _: ConnectionEvent| {};

		assert!(connection_eq(a, a));
		assert!(!connection_eq(a, b));
	}
}
