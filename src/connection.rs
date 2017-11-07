use std::{collections, marker, time};
use std::os::raw;
use std::sync::Arc;

use super::{
	error,
	sys,
	as_udata,
	udata_as,
	ConnectionFlags,
	Context,
	ContextRef,
	ConnectionEvent,
	Stanza,
};
use super::ffi_types::{FFI, Nullable};

/// Proxy to underlying `xmpp_conn_t` struct.
///
/// Most of the methods in this struct mimic the methods of the underlying library. So please see
/// libstrophe outdated docs for [connection] and [handlers] plus [conn.c] and [handler.c] sources.
/// Only where it's not the case or there is some additional logic involved then you can see the
/// method description.
///
/// This struct implements:
///   * `Drop` ([xmpp_conn_release]).
///   * `Eq` by comparing internal pointers
///   * `Send`
///
/// [connection]: http://strophe.im/libstrophe/doc/0.8-snapshot/group___connections.html
/// [handlers]: http://strophe.im/libstrophe/doc/0.8-snapshot/group___handlers.html
/// [conn.c]: https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L16
/// [handler.c]: https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L16
/// [xmpp_conn_release]: https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L222
#[derive(Debug, Hash)]
pub struct Connection<'cb> {
	inner: *mut sys::xmpp_conn_t,
	ctx: Arc<Context<'cb>>,
	owned: bool,
	_callbacks: marker::PhantomData<&'cb fn()>,
}

impl<'cb> Connection<'cb> {
	/// [xmpp_conn_new](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L76)
	pub fn new(ctx: Arc<Context<'cb>>) -> Connection<'cb> {
		unsafe {
			Connection::from_inner(sys::xmpp_conn_new(ctx.as_inner()), ctx)
		}
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_conn_t, ctx: Arc<Context<'cb>>, owned: bool) -> Connection<'cb> {
		if inner.is_null() {
			panic!("Cannot allocate memory for Connection");
		}
		Connection { inner, ctx, owned, _callbacks: marker::PhantomData }
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

	/// [xmpp_conn_get_flags](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L899)
	pub fn flags(&self) -> ConnectionFlags {
		ConnectionFlags::from_bits(unsafe { sys::xmpp_conn_get_flags(self.inner) }).unwrap()
	}

	/// [xmpp_conn_set_flags](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L918)
	pub fn set_flags(&mut self, flags: ConnectionFlags) -> error::EmptyResult {
		error::code_to_result(unsafe {
			sys::xmpp_conn_set_flags(self.inner, flags.bits())
		})
	}

	/// [xmpp_conn_get_jid](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L320)
	pub fn jid(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_jid(self.inner)).receive()
		}
	}

	/// [xmpp_conn_get_bound_jid](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L333)
	pub fn bound_jid(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_bound_jid(self.inner)).receive()
		}
	}


	/// [xmpp_conn_set_jid](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L351)
	pub fn set_jid<RefStr: AsRef<str>>(&mut self, jid: RefStr) {
		let jid = FFI(jid.as_ref()).send();
		unsafe {
			sys::xmpp_conn_set_jid(self.inner, jid.as_ptr())
		}
	}

	/// [xmpp_conn_get_pass](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L368)
	pub fn pass(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_conn_get_pass(self.inner)).receive()
		}
	}

	/// [xmpp_conn_set_pass](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L381)
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

	/// [xmpp_conn_disable_tls](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L958)
	pub fn disable_tls(&mut self) {
		unsafe {
			sys::xmpp_conn_disable_tls(self.inner)
		}
	}

	/// [xmpp_conn_is_secured](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L977)
	pub fn is_secured(&self) -> bool {
		unsafe {
			FFI(sys::xmpp_conn_is_secured(self.inner)).receive_bool()
		}
	}

	/// [xmpp_conn_set_keepalive](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L194)
	pub fn set_keepalive(&mut self, timeout: time::Duration, interval: time::Duration) {
		unsafe {
			sys::xmpp_conn_set_keepalive(self.inner, timeout.as_secs() as raw::c_int, interval.as_secs() as raw::c_int)
		}
	}

	/// [xmpp_connect_client](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L408)
	///
	/// Last `error: i32` argument in the handler contains a TLS error code that can be passed together
	/// with `XMPP_CONN_DISCONNECT` event. The specific meaning if that code depends on the underlying
	/// TLS implementation:
	///   * for `openssl` it's the result of [`SSL_get_error()`]
	///   * for `schannel` it's the result of [`WSAGetLastError()`]
	///
	/// Additionally the following OS dependent error constants can be set for `error`:
	///   * `ETIMEDOUT`/`WSAETIMEDOUT`
	///   * `ECONNRESET`/`WSAECONNRESET`
	///   * `ECONNABORTED`/`WSAECONNABORTED`
	///
	/// [`SSL_get_error()`]: https://wiki.openssl.org/index.php/Manual:SSL_get_error(3)
	/// [`WSAGetLastError()`]: https://msdn.microsoft.com/nl-nl/library/windows/desktop/ms741580(v=vs.85).aspx
	pub fn connect_client<U16, CB>(&mut self, alt_host: Option<&str>, alt_port: U16, handler: &'cb CB) -> error::EmptyResult
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
			sys::xmpp_connect_client(
				self.inner,
				alt_host.as_ptr(),
				alt_port.val(),
				Some(Self::connection_handler_cb::<CB>),
				as_udata(handler)
			)
		})
	}

	/// [xmpp_connect_component](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L484)
	///
	/// See also `connect_client()` for additional info.
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

	/// [xmpp_connect_raw](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L529)
	///
	/// See also `connect_client()` for additional info.
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

	/// [xmpp_conn_open_stream](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L605)
	///
	/// Related to `connect_raw()`.
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

	/// [xmpp_conn_tls_start](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L640)
	///
	/// Related to `connect_raw()`.
	pub fn tls_start(&self) -> error::EmptyResult {
		error::code_to_result(unsafe {
			sys::xmpp_conn_tls_start(self.inner)
		})
	}

	/// [xmpp_disconnect](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L702)
	pub fn disconnect(&mut self) {
		unsafe {
			sys::xmpp_disconnect(self.inner)
		}
	}

	/// [xmpp_send_raw_string](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L725)
	///
	/// Be aware that this method performs a lot of allocations internally so you might want to use
	/// `send_raw_bytes()` instead.
	pub fn send_raw_string<RefStr: AsRef<str>>(&mut self, data: RefStr) {
		let data = FFI(data.as_ref()).send();
		unsafe {
			sys::xmpp_send_raw_string(self.inner, data.as_ptr());
		}
	}

	/// [xmpp_send_raw](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L776)
	///
	/// Be aware that this method doesn't print debug log line with the message being sent (unlike
	/// [`send_raw_string()`]).
	pub fn send_raw<RefBytes: AsRef<[u8]>>(&mut self, data: RefBytes) {
		let data = data.as_ref();
		unsafe {
			sys::xmpp_send_raw(self.inner, data.as_ptr() as *const raw::c_char, data.len());
		}
	}

	/// [xmpp_send](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L822)
	pub fn send(&mut self, stanza: &Stanza) { unsafe { sys::xmpp_send(self.inner, stanza.as_inner()) } }

	/// [xmpp_timed_handler_add](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L464)
	pub fn timed_handler_add<CB>(&mut self, handler: &'cb CB, period: time::Duration)
		where
			CB: FnMut(&mut Connection<'cb>) -> bool + 'cb,
	{
		unsafe {
			self.timed_handler_add_unsafe(handler, period)
		}
	}

	/// [xmpp_timed_handler_add](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L464)
	pub unsafe fn timed_handler_add_unsafe<CB>(&mut self, handler: &CB, period: time::Duration)
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

	/// [timed_handler_delete](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L248)
	#[allow(unused_variables)]
	pub fn timed_handler_delete<CB>(&mut self, handler: &CB)
		where
			CB: FnMut(&mut Connection) -> bool,
	{
		unsafe {
			sys::xmpp_timed_handler_delete(self.inner, Some(Self::timed_handler_cb::<CB>))
		}
	}

	/// [xmpp_id_handler_add](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L505)
	pub fn id_handler_add<CB, RefStr: AsRef<str>>(&mut self, handler: &'cb CB, id: RefStr)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool + 'cb,
	{
		unsafe {
			self.id_handler_add_unsafe(handler, id)
		}
	}

	/// [xmpp_id_handler_add](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L505)
	pub unsafe fn id_handler_add_unsafe<CB, RefStr: AsRef<str>>(&mut self, handler: &CB, id: RefStr)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool,
	{
		let id = FFI(id.as_ref()).send();
		sys::xmpp_id_handler_add(self.inner, Some(Self::handler_cb::<CB>), id.as_ptr(), as_udata(handler))
	}

	/// [xmpp_id_handler_delete](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L324)
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

	/// [xmpp_handler_add](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L546)
	pub fn handler_add<CB>(&mut self, handler: &'cb CB, ns: Option<&str>, name: Option<&str>, typ: Option<&str>)
		where
			CB: FnMut(&mut Connection<'cb>, &Stanza) -> bool + 'cb,
	{
		unsafe {
			self.handler_add_unsafe(handler, ns, name, typ)
		}
	}

	/// [xmpp_handler_add](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L546)
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

	/// [xmpp_handler_delete](https://github.com/strophe/libstrophe/blob/0.9.1/src/handler.c#L427)
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
	/// [xmpp_conn_release](https://github.com/strophe/libstrophe/blob/0.9.1/src/conn.c#L222)
	fn drop(&mut self) {
		unsafe {
			if self.owned {
				sys::xmpp_conn_release(self.inner);
			}
		}
	}
}

unsafe impl<'cb> Send for Connection<'cb> {}
