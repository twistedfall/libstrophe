use std::{collections, marker, os::raw, sync::Arc, time::Duration};
use super::{
	as_udata,
	ConnectionEvent,
	ConnectionFlags,
	Context,
	ContextRef,
	error,
	Stanza,
	sys,
	udata_as,
};
use super::ffi_types::{FFI, Nullable};

/// Proxy to underlying `xmpp_conn_t` struct.
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
	cb: Option<Box<FnMut(&mut Connection<'cb>, ConnectionEvent, i32, Option<&error::StreamError>) + 'cb>>,
	_callbacks: marker::PhantomData<&'cb fn()>,
}

impl<'cb> Connection<'cb> {
	/// [xmpp_conn_new](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga76c161884c23f69cd1d7fc025122cf21)
	pub fn new(ctx: Arc<Context<'cb>>) -> Self {
		unsafe {
			Connection::from_inner(sys::xmpp_conn_new(ctx.as_inner()), ctx)
		}
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_conn_t, ctx: Arc<Context<'cb>>, owned: bool) -> Connection<'cb> {
		if inner.is_null() {
			panic!("Cannot allocate memory for Connection");
		}
		Connection { inner, ctx, owned, cb: None, _callbacks: marker::PhantomData }
	}

	pub unsafe fn from_inner(inner: *mut sys::xmpp_conn_t, ctx: Arc<Context<'cb>>) -> Connection<'cb> {
		Connection::with_inner(inner, ctx, true)
	}

	pub unsafe fn from_inner_ref_mut(inner: *mut sys::xmpp_conn_t, ctx: Arc<Context<'cb>>) -> Connection<'cb> {
		Connection::with_inner(inner, ctx, false)
	}

	unsafe extern "C" fn connection_handler_cb<CB>(conn: *const sys::xmpp_conn_t, event: sys::xmpp_conn_event_t, error: raw::c_int,
	                                               stream_error: *const sys::xmpp_stream_error_t, userdata: *const raw::c_void)
		where
			CB: FnMut(&mut Connection<'cb>, ConnectionEvent, i32, Option<&error::StreamError>),
	{
		let ctx = Context::from_inner_ref(sys::xmpp_conn_get_context(conn));
		let mut conn = Connection::from_inner_ref_mut(conn as *mut _, Arc::new(ctx));
		let stream_error: Option<error::StreamError> = stream_error.as_ref().map(|e| e.into());
		udata_as::<CB>(userdata)(&mut conn, event, error, stream_error.as_ref());
	}

	unsafe extern "C" fn timed_handler_cb<CB>(conn: *const sys::xmpp_conn_t, userdata: *const raw::c_void) -> i32
		where
			CB: FnMut(&mut Connection<'cb>) -> bool,
	{
		let ctx = Context::from_inner_ref(sys::xmpp_conn_get_context(conn));
		let mut conn = Connection::from_inner_ref_mut(conn as *mut _, Arc::new(ctx));
		udata_as::<CB>(userdata)(&mut conn) as i32
	}

	unsafe extern "C" fn handler_cb<CB>(conn: *const sys::xmpp_conn_t, stanza: *const sys::xmpp_stanza_t, userdata: *const raw::c_void) -> i32
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool,
	{
		let ctx = Arc::new(Context::from_inner_ref(sys::xmpp_conn_get_context(conn)));
		let mut conn = Connection::from_inner_ref_mut(conn as *mut _, ctx.clone());
		let stanza = Stanza::from_inner_ref(stanza);
		udata_as::<CB>(userdata)(&mut conn, &stanza) as i32
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
	pub fn set_jid<RefStr: AsRef<str>>(&mut self, jid: RefStr) {
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
	pub fn set_pass<RefStr: AsRef<str>>(&mut self, pass: RefStr) {
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
			sys::xmpp_conn_set_keepalive(self.inner, timeout.as_secs() as raw::c_int, interval.as_secs() as raw::c_int)
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
	/// [`WSAGetLastError()`]: https://msdn.microsoft.com/nl-nl/library/windows/desktop/ms741580(v=vs.85).aspx
	pub fn connect_client<U16, CB>(&mut self, alt_host: Option<&str>, alt_port: U16, handler: CB) -> error::EmptyResult
		where
			U16: Into<Option<u16>>,
			CB: FnMut(&mut Connection<'cb>, ConnectionEvent, i32, Option<&error::StreamError>) + 'cb,
	{
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if self.jid().is_none() {
			bail!(error::ErrorKind::InvalidOperation);
		}
		let handler = Box::new(handler);
		let out = error::code_to_result(unsafe {
			sys::xmpp_connect_client(
				self.inner,
				alt_host.as_ptr(),
				alt_port.val(),
				Some(Self::connection_handler_cb::<CB>),
				as_udata(&*handler)
			)
		});
		if out.is_ok() {
			self.cb = Some(handler);
		}
		out
	}

	/// [xmpp_connect_component](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#ga80c8cd7906a48fc27664fcce8f15ed7d)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_component<RefStr, U16, CB>(&mut self, host: RefStr, port: U16, handler: &'cb CB) -> error::EmptyResult
		where
			RefStr: AsRef<str>,
			U16: Into<Option<u16>>,
			CB: FnMut(&mut Connection<'cb>, ConnectionEvent, i32, Option<&error::StreamError>) + 'cb,
	{
		let host = FFI(host.as_ref()).send();
		let port: Nullable<_> = port.into().into();
		error::code_to_result(unsafe {
			sys::xmpp_connect_component(
				self.inner,
				host.as_ptr(),
				port.val(),
				Some(Self::connection_handler_cb::<CB>),
				as_udata(handler)
			)
		})
	}

	/// [xmpp_connect_raw](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gae64b7a2ec8e138a1501bb7bf12089776)
	///
	/// See also [`connect_client()`](#method.connect_client) for additional info.
	pub fn connect_raw<U16, CB>(&mut self, alt_host: Option<&str>, alt_port: U16, handler: &'cb CB) -> error::EmptyResult
		where
			U16: Into<Option<u16>>,
			CB: FnMut(&mut Connection<'cb>, ConnectionEvent, i32, Option<&error::StreamError>) + 'cb,
	{
		let alt_host = FFI(alt_host).send();
		let alt_port: Nullable<_> = alt_port.into().into();
		if self.jid().is_none() {
			bail!(error::ErrorKind::InvalidOperation);
		}
		error::code_to_result(unsafe {
			sys::xmpp_connect_raw(
				self.inner,
				alt_host.as_ptr(),
				alt_port.val(),
				Some(Self::connection_handler_cb::<CB>),
				as_udata(handler)
			)
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
			attrs.push(storage[storage.len() - 2].as_ptr() as *mut _);
			attrs.push(storage[storage.len() - 1].as_ptr() as *mut _);
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
	pub fn send_raw_string<RefStr: AsRef<str>>(&mut self, data: RefStr) {
		let data = FFI(data.as_ref()).send();
		unsafe {
			sys::xmpp_send_raw_string(self.inner, data.as_ptr());
		}
	}

	/// [xmpp_send_raw](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gadd1c8707fa269e6d6845d6b856584add)
	///
	/// Be aware that this method doesn't print debug log line with the message being sent (unlike
	/// [`send_raw_string()`](#method.send_raw_string)).
	pub fn send_raw<RefBytes: AsRef<[u8]>>(&mut self, data: RefBytes) {
		let data = data.as_ref();
		unsafe {
			sys::xmpp_send_raw(self.inner, data.as_ptr() as *const raw::c_char, data.len());
		}
	}

	/// [xmpp_send](http://strophe.im/libstrophe/doc/0.9.2/group___connections.html#gac064b81b22a3166d5b396b5aa8e40b7d)
	pub fn send(&mut self, stanza: &Stanza) { unsafe { sys::xmpp_send(self.inner, stanza.as_inner()) } }

	/// [xmpp_timed_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga0a74b20f2367389e5dc8852b4d3fdcda)
	pub fn timed_handler_add<CB>(&mut self, handler: &'cb CB, period: Duration)
		where
			CB: FnMut(&mut Connection<'cb>) -> bool + 'cb,
	{
		unsafe {
			self.timed_handler_add_unsafe(handler, period)
		}
	}

	/// [xmpp_timed_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga0a74b20f2367389e5dc8852b4d3fdcda)
	///
	/// Please see module level documentation, [Callbacks section][docs] about the reasoning behind
	/// this method.
	///
	/// [docs]: index.html#callbacks
	pub unsafe fn timed_handler_add_unsafe<CB>(&mut self, handler: &CB, period: Duration)
		where
			CB: FnMut(&mut Connection<'cb>) -> bool,
	{
		sys::xmpp_timed_handler_add(
			self.inner,
			Some(Self::timed_handler_cb::<CB>),
			super::duration_as_ms(period),
			as_udata(handler)
		)
	}

	/// [xmpp_timed_handler_delete](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#gae70f02f84a0f232c6a8c2866ecb47b82)
	#[allow(unused_variables)]
	pub fn timed_handler_delete<CB>(&mut self, handler: &CB)
		where
			CB: FnMut(&mut Connection) -> bool,
	{
		unsafe {
			sys::xmpp_timed_handler_delete(self.inner, Some(Self::timed_handler_cb::<CB>))
		}
	}

	/// [xmpp_id_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga2142d78b7d6e8d278eebfa8a63f194a4)
	pub fn id_handler_add<CB, RefStr: AsRef<str>>(&mut self, handler: &'cb CB, id: RefStr)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool + 'cb,
	{
		unsafe {
			self.id_handler_add_unsafe(handler, id)
		}
	}

	/// [xmpp_id_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga2142d78b7d6e8d278eebfa8a63f194a4)
	///
	/// Please see module level documentation, [Callbacks section][docs] about the reasoning behind
	/// this method.
	///
	/// [docs]: index.html#callbacks
	pub unsafe fn id_handler_add_unsafe<CB, RefStr: AsRef<str>>(&mut self, handler: &CB, id: RefStr)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool,
	{
		let id = FFI(id.as_ref()).send();
		sys::xmpp_id_handler_add(self.inner, Some(Self::handler_cb::<CB>), id.as_ptr(), as_udata(handler))
	}

	/// [xmpp_id_handler_delete](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga6bd02f7254b2a53214824d3d5e4f59ce)
	#[allow(unused_variables)]
	pub fn id_handler_delete<CB, RefStr: AsRef<str>>(&mut self, handler: &CB, id: RefStr)
		where
			CB: FnMut(&mut Connection, &Stanza) -> bool,
	{
		let id = FFI(id.as_ref()).send();
		unsafe {
			sys::xmpp_id_handler_delete(self.inner, Some(Self::handler_cb::<CB>), id.as_ptr())
		}
	}

	/// [xmpp_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#gad307e5a22d16ef3d6fa18d503b68944f)
	pub fn handler_add<CB>(&mut self, handler: &'cb CB, ns: Option<&str>, name: Option<&str>, typ: Option<&str>)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool + 'cb,
	{
		unsafe {
			self.handler_add_unsafe(handler, ns, name, typ)
		}
	}

	/// [xmpp_handler_add](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#gad307e5a22d16ef3d6fa18d503b68944f)
	///
	/// Please see module level documentation, [Callbacks section][docs] about the reasoning behind
	/// this method.
	///
	/// [docs]: index.html#callbacks
	pub unsafe fn handler_add_unsafe<CB>(&mut self, handler: &CB, ns: Option<&str>, name: Option<&str>, typ: Option<&str>)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool,
	{
		let ns = FFI(ns).send();
		let name = FFI(name).send();
		let typ = FFI(typ).send();
		sys::xmpp_handler_add(
			self.inner,
			Some(Self::handler_cb::<CB>),
			ns.as_ptr(),
			name.as_ptr(),
			typ.as_ptr(),
			as_udata(handler)
		)
	}

	/// [xmpp_handler_delete](http://strophe.im/libstrophe/doc/0.9.2/group___handlers.html#ga881756ae98f74748212bcbc61e6a4c89)
	#[allow(unused_variables)]
	pub fn handler_delete<CB>(&mut self, handler: &CB)
		where
			CB: FnMut(&mut Connection, &Stanza) -> bool,
	{
		unsafe {
			sys::xmpp_handler_delete(self.inner, Some(Self::handler_cb::<CB>))
		}
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
			}
		}
	}
}

unsafe impl<'cb> Send for Connection<'cb> {}
