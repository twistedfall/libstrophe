use std::{
	collections,
	mem,
	os::raw,
	sync::Arc,
	time::Duration,
};
use super::{
	as_void_ptr,
	ConnectionEvent,
	ConnectionFlags,
	Context,
	ContextRef,
	error,
	ffi_types::{FFI, Nullable},
	Stanza,
	sys,
	void_ptr_as,
};

type ConnectionCallback<'cb> = dyn for<'f> FnMut(&'f mut Connection<'cb>, ConnectionEvent, i32, Option<&'f error::StreamError<'f>>) + 'cb;
type ConnectionFatHandler<'cb> = FatHandler<'cb, ConnectionCallback<'cb>, ()>;

type Handlers<H> = Vec<Box<H>>;

type TimedCallback<'cb> = dyn for<'f> FnMut(&'f mut Connection<'cb>) -> bool + 'cb;
type TimedFatHandler<'cb> = FatHandler<'cb, TimedCallback<'cb>, ()>;

type StanzaCallback<'cb> = dyn for<'f, 'cx> FnMut(&'f mut Connection<'cb>, &'f Stanza<'cx>) -> bool + 'cb;
type StanzaFatHandler<'cb> = FatHandler<'cb, StanzaCallback<'cb>, Option<String>>;

struct FatHandlers<'cb> {
	connection: Option<ConnectionFatHandler<'cb>>,
	timed: Handlers<TimedFatHandler<'cb>>,
	stanza: Handlers<StanzaFatHandler<'cb>>,
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
/// [conn.c]: https://github.com/strophe/libstrophe/blob/0.9.2/src/conn.c
/// [handler.c]: https://github.com/strophe/libstrophe/blob/0.9.2/src/handler.c
/// [xmpp_conn_release]: http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga16967e3375efa5032ed2e08b407d8ae9
pub struct Connection<'cb> {
	inner: *mut sys::xmpp_conn_t,
	ctx: Arc<Context<'cb>>,
	owned: bool,
	fat_handlers: Box<FatHandlers<'cb>>,
}

impl<'cb> Connection<'cb> {
	/// [xmpp_conn_new](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga76c161884c23f69cd1d7fc025122cf21)
	pub fn new(ctx: Arc<Context<'cb>>) -> Self {
		unsafe {
			Connection::from_inner(sys::xmpp_conn_new(ctx.as_inner()), ctx, Box::new(FatHandlers {
				connection: None,
				timed: Vec::with_capacity(4),
				stanza: Vec::with_capacity(4),
			}))
		}
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_conn_t, ctx: Arc<Context<'cb>>, owned: bool, handlers: Box<FatHandlers<'cb>>) -> Self {
		if inner.is_null() {
			panic!("Cannot allocate memory for Connection");
		}
		Connection { inner, ctx, owned, fat_handlers: handlers }
	}

	unsafe fn from_inner(inner: *mut sys::xmpp_conn_t, ctx: Arc<Context<'cb>>, handlers: Box<FatHandlers<'cb>>) -> Self {
		Connection::with_inner(
			inner,
			ctx,
			true,
			handlers,
		)
	}

	unsafe fn from_inner_ref_mut(inner: *mut sys::xmpp_conn_t, handlers: Box<FatHandlers<'cb>>) -> Self {
		let ctx = Arc::new(Context::from_inner_ref(sys::xmpp_conn_get_context(inner)));
		Connection::with_inner(inner, ctx, false, handlers)
	}

	extern "C" fn connection_handler_cb<CB>(conn: *mut sys::xmpp_conn_t, event: sys::xmpp_conn_event_t, error: raw::c_int,
	                                        stream_error: *mut sys::xmpp_stream_error_t, userdata: *mut raw::c_void) {
		let connection_handler = unsafe { void_ptr_as::<ConnectionFatHandler>(userdata) };
		let mut conn = unsafe { Connection::from_inner_ref_mut(conn, Box::from_raw(connection_handler.fat_handlers_ptr)) };
		let stream_error: Option<error::StreamError> = unsafe { stream_error.as_ref() }.map(|e| e.into());
		(connection_handler.handler)(&mut conn, event, error, stream_error.as_ref());
	}

	extern "C" fn timed_handler_cb<CB>(conn: *mut sys::xmpp_conn_t, userdata: *mut raw::c_void) -> i32 {
		let timed_handler = unsafe { void_ptr_as::<TimedFatHandler>(userdata) };
		let mut conn = unsafe { Connection::from_inner_ref_mut(conn, Box::from_raw(timed_handler.fat_handlers_ptr)) };
		let res = (timed_handler.handler)(&mut conn);
		if !res {
			Connection::drop_fat_handler(&mut conn.fat_handlers.timed, timed_handler);
		}
		res as _
	}

	extern "C" fn handler_cb<CB>(conn: *mut sys::xmpp_conn_t, stanza: *mut sys::xmpp_stanza_t, userdata: *mut raw::c_void) -> i32 {
		let stanza_handler = unsafe { void_ptr_as::<StanzaFatHandler>(userdata) };
		let mut conn = unsafe { Connection::from_inner_ref_mut(conn, Box::from_raw(stanza_handler.fat_handlers_ptr)) };
		let stanza = unsafe { Stanza::from_inner_ref(stanza) };
		let res = (stanza_handler.handler)(&mut conn, &stanza);
		if !res {
			Connection::drop_fat_handler(&mut conn.fat_handlers.stanza, stanza_handler);
		}
		res as _
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

	fn get_fat_handler_pos<CB: ?Sized, T>(fat_handlers: &Handlers<FatHandler<'cb, CB, T>>, fat_handler_ptr: *const FatHandler<'cb, CB, T>) -> Option<usize> {
		fat_handlers.iter().position(|x| fat_handler_ptr == x.as_ref())
	}

	fn get_fat_handler_pos_by_callback<CB: ?Sized, T>(fat_handlers: &Handlers<FatHandler<'cb, CB, T>>, cb_addr: *const ()) -> Option<usize> {
		fat_handlers.iter().position(|x| cb_addr == x.cb_addr)
	}

	fn validate_fat_handler<'f, CB: ?Sized, T>(fat_handlers: &'f Handlers<FatHandler<'cb, CB, T>>, fat_handler_ptr: *const FatHandler<'cb, CB, T>) -> Option<&'f FatHandler<'cb, CB, T>> {
		Connection::get_fat_handler_pos(fat_handlers, fat_handler_ptr).map(|pos| {
			fat_handlers[pos].as_ref()
		})
	}

	fn drop_fat_handler<CB: ?Sized, T>(fat_handlers: &mut Handlers<FatHandler<'cb, CB, T>>, fat_handler_ptr: *const FatHandler<'cb, CB, T>) -> Option<usize> {
		if let Some(pos) = Connection::get_fat_handler_pos(fat_handlers, fat_handler_ptr) {
			fat_handlers.remove(pos);
			Some(pos)
		} else {
			None
		}
	}

	fn make_fat_handler<CB: ?Sized, T>(&self, handler: Box<CB>, cb_addr: *const (), extra: T) -> FatHandler<'cb, CB, T> {
		FatHandler {
			fat_handlers_ptr: &*self.fat_handlers as *const _ as _,
			handler,
			cb_addr,
			extra
		}
	}

	/// [xmpp_conn_get_flags](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaa9724ae412b01562dc81c31fd178d2f3)
	pub fn flags(&self) -> ConnectionFlags {
		ConnectionFlags::from_bits(unsafe { sys::xmpp_conn_get_flags(self.inner) }).unwrap()
	}

	/// [xmpp_conn_set_flags](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga92761a101e721df9b923e9c35f6ad949)
	pub fn set_flags(&mut self, flags: ConnectionFlags) -> error::EmptyResult {
		error::code_to_result(unsafe {
			sys::xmpp_conn_set_flags(self.inner, flags.bits())
		})
	}

	/// [xmpp_conn_get_jid](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga52b5fee898fc6ef06ba1ea7f9a507a39)
	pub fn jid(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_jid(self.inner)).receive()
		}
	}

	/// [xmpp_conn_get_bound_jid](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga9bc4f0527c6aa7ac4d5b112be6189889)
	pub fn bound_jid(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_bound_jid(self.inner)).receive()
		}
	}

	/// [xmpp_conn_set_jid](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga8dff6d97ac458d5f3fc901d688d86084)
	pub fn set_jid(&mut self, jid: impl AsRef<str>) {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			sys::xmpp_conn_set_jid(self.inner, jid.as_ptr())
		}
	}

	/// [xmpp_conn_get_pass](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaa5e1ce97c2d8ad50380b92b2ca204dec)
	pub fn pass(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_pass(self.inner)).receive()
		}
	}

	/// [xmpp_conn_set_pass](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gac64373b712d9d8af12e57b753d9b3bfc)
	pub fn set_pass(&mut self, pass: impl AsRef<str>) {
		let pass = FFI(pass.as_ref()).send();
		unsafe {
			sys::xmpp_conn_set_pass(self.inner, pass.as_ptr())
		}
	}

	/// Get reference to context associated with this connection.
	pub fn context(&self) -> ContextRef<'cb> {
		self.ctx.clone().into()
	}

	/// [xmpp_conn_disable_tls](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga1730868abcec1c6c63f2351bb81a43ac)
	pub fn disable_tls(&mut self) {
		unsafe {
			sys::xmpp_conn_disable_tls(self.inner)
		}
	}

	/// [xmpp_conn_is_secured](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga331bfca7c9c9ce3a17c909e770a73b02)
	pub fn is_secured(&self) -> bool {
		unsafe {
			FFI(sys::xmpp_conn_is_secured(self.inner)).receive_bool()
		}
	}

	/// [xmpp_conn_set_keepalive](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga0c75095f31ee66febcf71cad1e60d4f6)
	pub fn set_keepalive(&mut self, timeout: Duration, interval: Duration) {
		unsafe {
			sys::xmpp_conn_set_keepalive(self.inner, timeout.as_secs() as _, interval.as_secs() as _)
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
	pub fn connect_client<CB>(&mut self, alt_host: Option<&str>, alt_port: impl Into<Option<u16>>, handler: CB) -> error::EmptyResult
		where
			CB: for<'f> FnMut(&'f mut Connection<'cb>, ConnectionEvent, i32, Option<&'f error::StreamError<'f>>) + 'cb,
	{
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if self.jid().is_none() {
			return Err(error::Error::InvalidOperation.into());
		}
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(self.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut self.fat_handlers.connection, new_handler);
		error::code_to_result(unsafe {
			sys::xmpp_connect_client(
				self.inner,
				alt_host.as_ptr(),
				alt_port.val(),
				Some(callback),
				as_void_ptr(self.fat_handlers.connection.as_ref().unwrap()),
			)
		}).map_err(|x| {
			self.fat_handlers.connection = old_handler;
			x
		})
	}

	/// [xmpp_connect_component](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga80c8cd7906a48fc27664fcce8f15ed7d)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_component<CB>(&mut self, host: impl AsRef<str>, port: impl Into<Option<u16>>, handler: CB) -> error::EmptyResult
		where
			CB: for<'f> FnMut(&'f mut Connection<'cb>, ConnectionEvent, i32, Option<&'f error::StreamError<'f>>) + 'cb,
	{
		let host = FFI(host.as_ref()).send();
		let port: Nullable<_> = port.into().into();
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(self.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut self.fat_handlers.connection, new_handler);
		error::code_to_result(unsafe {
			sys::xmpp_connect_component(
				self.inner,
				host.as_ptr(),
				port.val(),
				Some(callback),
				as_void_ptr(&self.fat_handlers.connection),
			)
		}).map_err(|x| {
			self.fat_handlers.connection = old_handler;
			x
		})
	}

	/// [xmpp_connect_raw](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gae64b7a2ec8e138a1501bb7bf12089776)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_raw<CB>(&mut self, alt_host: Option<&str>, alt_port: impl Into<Option<u16>>, handler: CB) -> error::EmptyResult
		where
			CB: for<'f> FnMut(&'f mut Connection<'cb>, ConnectionEvent, i32, Option<&'f error::StreamError<'f>>) + 'cb,
	{
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if self.jid().is_none() {
			return Err(error::Error::InvalidOperation.into());
		}
		let callback = Self::connection_handler_cb::<CB>;
		let new_handler = Some(self.make_fat_handler(Box::new(handler) as _, callback as _, ()));
		let old_handler = mem::replace(&mut self.fat_handlers.connection, new_handler);
		error::code_to_result(unsafe {
			sys::xmpp_connect_raw(
				self.inner,
				alt_host.as_ptr(),
				alt_port.val(),
				Some(callback),
				as_void_ptr(self.fat_handlers.connection.as_ref().unwrap()),
			)
		}).map_err(|x| {
			self.fat_handlers.connection = old_handler;
			x
		})
	}

	/// [xmpp_conn_open_stream_default](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gaac72cb61c7a69499fd1387c1d499c08e)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn open_stream_default(&self) -> error::EmptyResult {
		error::code_to_result(unsafe {
			sys::xmpp_conn_open_stream_default(self.inner)
		})
	}

	/// [xmpp_conn_open_stream](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga85ccb1c2d95caf29dff0c9b70424c53e)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn open_stream(&self, attributes: &collections::HashMap<&str, &str>) -> error::EmptyResult {
		let mut storage = Vec::with_capacity(attributes.len() * 2);
		let mut attrs = Vec::with_capacity(attributes.len() * 2);
		for attr in attributes {
			storage.push(FFI(*attr.0).send());
			storage.push(FFI(*attr.1).send());
			attrs.push(storage[storage.len() - 2].as_ptr() as _);
			attrs.push(storage[storage.len() - 1].as_ptr() as _);
		}
		error::code_to_result(unsafe {
			sys::xmpp_conn_open_stream(
				self.inner,
				attrs.as_mut_ptr(),
				attrs.len(),
			)
		})
	}

	/// [xmpp_conn_tls_start](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga759b1ae6fd1e40335afd8acc26d3858f)
	///
	/// Related to [`connect_raw()`](#method.connect_raw).
	pub fn tls_start(&self) -> error::EmptyResult {
		error::code_to_result(unsafe {
			sys::xmpp_conn_tls_start(self.inner)
		})
	}

	/// [xmpp_disconnect](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga809ee4c8bb95e86ec2119db1052849ce)
	pub fn disconnect(&mut self) {
		unsafe {
			sys::xmpp_disconnect(self.inner)
		}
	}

	/// [xmpp_send_raw_string](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga606ee8f53d8992941d93ecf27e41595b)
	///
	/// Be aware that this method performs a lot of allocations internally so you might want to use
	/// [`send_raw()`](#method.send_raw) instead.
	pub fn send_raw_string(&mut self, data: impl AsRef<str>) {
		let data = FFI(data.as_ref()).send();
		unsafe {
			sys::xmpp_send_raw_string(self.inner, data.as_ptr());
		}
	}

	/// [xmpp_send_raw](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gadd1c8707fa269e6d6845d6b856584add)
	///
	/// Be aware that this method doesn't print debug log line with the message being sent (unlike
	/// [`send_raw_string()`](#method.send_raw_string)).
	pub fn send_raw(&mut self, data: impl AsRef<[u8]>) {
		let data = data.as_ref();
		unsafe {
			sys::xmpp_send_raw(self.inner, data.as_ptr() as _, data.len());
		}
	}

	/// [xmpp_send](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gac064b81b22a3166d5b396b5aa8e40b7d)
	pub fn send(&mut self, stanza: &Stanza) { unsafe { sys::xmpp_send(self.inner, stanza.as_inner()) } }

	/// [xmpp_timed_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga0a74b20f2367389e5dc8852b4d3fdcda)
	///
	/// See `handler_add()` for additional information.
	pub fn timed_handler_add<CB>(&mut self, handler: CB, period: Duration) -> Option<TimedHandlerId<'cb, CB>>
		where
			CB: for<'f> FnMut(&'f mut Connection<'cb>) -> bool + 'cb,
	{
		let callback = Self::timed_handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, ());
		Connection::store_fat_handler(&mut self.fat_handlers.timed, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_timed_handler_add(
					self.inner,
					Some(callback),
					super::duration_as_ms(period),
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
		unsafe {
			sys::xmpp_timed_handler_delete(self.inner, Some(Self::timed_handler_cb::<CB>))
		}
		Connection::drop_fat_handler(&mut self.fat_handlers.timed, handler_id.0 as _);
	}

	/// See `handlers_clear()` for additional information.
	pub fn timed_handlers_clear(&mut self) {
		for handler in self.fat_handlers.timed.drain(..) {
			unsafe { sys::xmpp_timed_handler_delete(self.inner, Some(mem::transmute(handler.cb_addr))) };
		}
		self.fat_handlers.timed.shrink_to_fit();
	}

	/// [xmpp_id_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga2142d78b7d6e8d278eebfa8a63f194a4)
	///
	/// See `handler_add()` for additional information.
	pub fn id_handler_add<CB>(&mut self, handler: CB, id: impl Into<String>) -> Option<IdHandlerId<'cb, CB>>
		where
			CB: for<'f, 'cx> FnMut(&'f mut Connection<'cb>, &'f Stanza<'cx>) -> bool + 'cb,
	{
		let id = id.into();
		let ffi_id = FFI(id.as_str()).send();
		let callback = Self::handler_cb::<CB>;
		let handler = self.make_fat_handler(
			Box::new(handler) as _,
			callback as _,
			Some(id),
		);
		Connection::store_fat_handler(&mut self.fat_handlers.stanza, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_id_handler_add(
					self.inner,
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
		if let Some(fat_handler) = Connection::validate_fat_handler(&mut self.fat_handlers.stanza, handler_id.0 as _) {
			let id = FFI(fat_handler.extra.as_ref().unwrap().as_str()).send();
			unsafe {
				sys::xmpp_id_handler_delete(self.inner, Some(Self::handler_cb::<CB>), id.as_ptr())
			}
		}
		Connection::drop_fat_handler(&mut self.fat_handlers.stanza, handler_id.0 as _);
	}

	/// See `handlers_clear()` for additional information.
	pub fn id_handlers_clear(&mut self) {
		self.fat_handlers.stanza.retain(|x| {
			if let Some(ref id) = x.extra {
				unsafe { sys::xmpp_id_handler_delete(self.inner, Some(mem::transmute(x.cb_addr)), FFI(id.as_str()).send().as_ptr()) };
				false
			} else {
				true
			}
		});
		self.fat_handlers.stanza.shrink_to_fit();
	}

	/// [xmpp_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#gad307e5a22d16ef3d6fa18d503b68944f)
	///
	/// This function returns `HandlerId` which is later can be used to remove the handler using `handler_delete()`.
	pub fn handler_add<CB>(&mut self, handler: CB, ns: Option<&str>, name: Option<&str>, typ: Option<&str>) -> Option<HandlerId<'cb, 'cx, CB>>
		where
			CB: for<'f, 'cx> FnMut(&'f mut Connection<'cb>, &'f Stanza<'cx>) -> bool + 'cb,
	{
		let ns = FFI(ns).send();
		let name = FFI(name).send();
		let typ = FFI(typ).send();
		let callback = Self::handler_cb::<CB>;
		let handler = self.make_fat_handler(Box::new(handler) as _, callback as _, None);
		Connection::store_fat_handler(&mut self.fat_handlers.stanza, handler).map(|fat_handler_ptr| {
			unsafe {
				sys::xmpp_handler_add(
					self.inner,
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
		unsafe {
			sys::xmpp_handler_delete(self.inner, Some(Self::handler_cb::<CB>))
		}
		Connection::drop_fat_handler(&mut self.fat_handlers.stanza, handler_id.0 as _);
	}

	/// Removes all handlers that were set up with `handler_add()`. This function does *not* remove handlers added via `id_handler_add()`. You can use
	/// this function if you can't keep track of specific closure handles returned from `handler_add()`, but want to remove handlers anyway.
	pub fn handlers_clear(&mut self) {
		self.fat_handlers.stanza.retain(|x| {
			if x.extra.is_none() {
				unsafe { sys::xmpp_handler_delete(self.inner, Some(mem::transmute(x.cb_addr))) };
				false
			} else {
				true
			}
		});
		self.fat_handlers.stanza.shrink_to_fit();
	}
}

impl<'cb> PartialEq for Connection<'cb> {
	fn eq(&self, other: &Connection) -> bool {
		self.inner == other.inner
	}
}

impl<'cb> Eq for Connection<'cb> {}

impl<'cb> Drop for Connection<'cb> {
	/// [xmpp_conn_release](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga16967e3375efa5032ed2e08b407d8ae9)
	fn drop(&mut self) {
		unsafe {
			if self.owned {
				sys::xmpp_conn_release(self.inner);
			} else {
				// intentionally leak handlers because they are owned by other Connection
				Box::into_raw(mem::replace(&mut self.fat_handlers, Box::new(FatHandlers {
					connection: None,
					timed: Vec::with_capacity(0),
					stanza: Vec::with_capacity(0),
				})));
			}
		}
	}
}

unsafe impl<'cb> Send for Connection<'cb> {}

pub struct HandlerId<'cb, CB>(*const FatHandler<'cb, CB, ()>);

pub struct TimedHandlerId<'cb, CB>(*const FatHandler<'cb, CB, ()>);

pub struct IdHandlerId<'cb, CB>(*const FatHandler<'cb, CB, Option<String>>);

pub struct FatHandler<'cb, CB: ?Sized, T> {
	fat_handlers_ptr: *mut FatHandlers<'cb>,
	handler: Box<CB>,
	cb_addr: *const (),
	extra: T,
}
