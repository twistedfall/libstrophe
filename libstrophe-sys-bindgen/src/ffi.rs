/* automatically generated by rust-bindgen 0.65.1 */

pub const XMPP_NS_CLIENT: &[u8; 14usize] = b"jabber:client\0";
pub const XMPP_NS_COMPONENT: &[u8; 24usize] = b"jabber:component:accept\0";
pub const XMPP_NS_STREAMS: &[u8; 33usize] = b"http://etherx.jabber.org/streams\0";
pub const XMPP_NS_STREAMS_IETF: &[u8; 36usize] = b"urn:ietf:params:xml:ns:xmpp-streams\0";
pub const XMPP_NS_STANZAS_IETF: &[u8; 36usize] = b"urn:ietf:params:xml:ns:xmpp-stanzas\0";
pub const XMPP_NS_TLS: &[u8; 32usize] = b"urn:ietf:params:xml:ns:xmpp-tls\0";
pub const XMPP_NS_SASL: &[u8; 33usize] = b"urn:ietf:params:xml:ns:xmpp-sasl\0";
pub const XMPP_NS_BIND: &[u8; 33usize] = b"urn:ietf:params:xml:ns:xmpp-bind\0";
pub const XMPP_NS_SESSION: &[u8; 36usize] = b"urn:ietf:params:xml:ns:xmpp-session\0";
pub const XMPP_NS_AUTH: &[u8; 15usize] = b"jabber:iq:auth\0";
pub const XMPP_NS_DISCO_INFO: &[u8; 38usize] = b"http://jabber.org/protocol/disco#info\0";
pub const XMPP_NS_DISCO_ITEMS: &[u8; 39usize] = b"http://jabber.org/protocol/disco#items\0";
pub const XMPP_NS_ROSTER: &[u8; 17usize] = b"jabber:iq:roster\0";
pub const XMPP_NS_REGISTER: &[u8; 19usize] = b"jabber:iq:register\0";
pub const XMPP_NS_SM: &[u8; 14usize] = b"urn:xmpp:sm:3\0";
pub const XMPP_EOK: i32 = 0;
pub const XMPP_EMEM: i32 = -1;
pub const XMPP_EINVOP: i32 = -2;
pub const XMPP_EINT: i32 = -3;
pub const XMPP_CONN_FLAG_DISABLE_TLS: u32 = 1;
pub const XMPP_CONN_FLAG_MANDATORY_TLS: u32 = 2;
pub const XMPP_CONN_FLAG_LEGACY_SSL: u32 = 4;
pub const XMPP_CONN_FLAG_TRUST_TLS: u32 = 8;
pub const XMPP_CONN_FLAG_LEGACY_AUTH: u32 = 16;
pub const XMPP_CONN_FLAG_DISABLE_SM: u32 = 32;
pub const XMPP_SHA1_DIGEST_SIZE: u32 = 20;
extern "C" {
	pub fn xmpp_initialize();
}
extern "C" {
	pub fn xmpp_shutdown();
}
extern "C" {
	pub fn xmpp_version_check(major: ::std::os::raw::c_int, minor: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
pub type xmpp_mem_t = _xmpp_mem_t;
pub type xmpp_log_t = _xmpp_log_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_ctx_t {
	_unused: [u8; 0],
}
pub type xmpp_ctx_t = _xmpp_ctx_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_tlscert_t {
	_unused: [u8; 0],
}
pub type xmpp_tlscert_t = _xmpp_tlscert_t;
extern "C" {
	pub fn xmpp_ctx_new(mem: *const xmpp_mem_t, log: *const xmpp_log_t) -> *mut xmpp_ctx_t;
}
extern "C" {
	pub fn xmpp_ctx_free(ctx: *mut xmpp_ctx_t);
}
extern "C" {
	pub fn xmpp_ctx_set_verbosity(ctx: *mut xmpp_ctx_t, level: ::std::os::raw::c_int);
}
extern "C" {
	pub fn xmpp_free(ctx: *const xmpp_ctx_t, p: *mut ::std::os::raw::c_void);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_mem_t {
	pub alloc: ::std::option::Option<
		unsafe extern "C" fn(size: usize, userdata: *mut ::std::os::raw::c_void) -> *mut ::std::os::raw::c_void,
	>,
	pub free: ::std::option::Option<unsafe extern "C" fn(p: *mut ::std::os::raw::c_void, userdata: *mut ::std::os::raw::c_void)>,
	pub realloc: ::std::option::Option<
		unsafe extern "C" fn(
			p: *mut ::std::os::raw::c_void,
			size: usize,
			userdata: *mut ::std::os::raw::c_void,
		) -> *mut ::std::os::raw::c_void,
	>,
	pub userdata: *mut ::std::os::raw::c_void,
}
#[test]
fn bindgen_test_layout__xmpp_mem_t() {
	const UNINIT: ::std::mem::MaybeUninit<_xmpp_mem_t> = ::std::mem::MaybeUninit::uninit();
	let ptr = UNINIT.as_ptr();
	assert_eq!(
		::std::mem::size_of::<_xmpp_mem_t>(),
		32usize,
		concat!("Size of: ", stringify!(_xmpp_mem_t))
	);
	assert_eq!(
		::std::mem::align_of::<_xmpp_mem_t>(),
		8usize,
		concat!("Alignment of ", stringify!(_xmpp_mem_t))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).alloc) as usize - ptr as usize },
		0usize,
		concat!("Offset of field: ", stringify!(_xmpp_mem_t), "::", stringify!(alloc))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).free) as usize - ptr as usize },
		8usize,
		concat!("Offset of field: ", stringify!(_xmpp_mem_t), "::", stringify!(free))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).realloc) as usize - ptr as usize },
		16usize,
		concat!("Offset of field: ", stringify!(_xmpp_mem_t), "::", stringify!(realloc))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).userdata) as usize - ptr as usize },
		24usize,
		concat!("Offset of field: ", stringify!(_xmpp_mem_t), "::", stringify!(userdata))
	);
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum xmpp_log_level_t {
	XMPP_LEVEL_DEBUG = 0,
	XMPP_LEVEL_INFO = 1,
	XMPP_LEVEL_WARN = 2,
	XMPP_LEVEL_ERROR = 3,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum xmpp_conn_type_t {
	XMPP_UNKNOWN = 0,
	XMPP_CLIENT = 1,
	XMPP_COMPONENT = 2,
}
pub type xmpp_log_handler = ::std::option::Option<
	unsafe extern "C" fn(
		userdata: *mut ::std::os::raw::c_void,
		level: xmpp_log_level_t,
		area: *const ::std::os::raw::c_char,
		msg: *const ::std::os::raw::c_char,
	),
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_log_t {
	pub handler: xmpp_log_handler,
	pub userdata: *mut ::std::os::raw::c_void,
}
#[test]
fn bindgen_test_layout__xmpp_log_t() {
	const UNINIT: ::std::mem::MaybeUninit<_xmpp_log_t> = ::std::mem::MaybeUninit::uninit();
	let ptr = UNINIT.as_ptr();
	assert_eq!(
		::std::mem::size_of::<_xmpp_log_t>(),
		16usize,
		concat!("Size of: ", stringify!(_xmpp_log_t))
	);
	assert_eq!(
		::std::mem::align_of::<_xmpp_log_t>(),
		8usize,
		concat!("Alignment of ", stringify!(_xmpp_log_t))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).handler) as usize - ptr as usize },
		0usize,
		concat!("Offset of field: ", stringify!(_xmpp_log_t), "::", stringify!(handler))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).userdata) as usize - ptr as usize },
		8usize,
		concat!("Offset of field: ", stringify!(_xmpp_log_t), "::", stringify!(userdata))
	);
}
extern "C" {
	pub fn xmpp_get_default_logger(level: xmpp_log_level_t) -> *mut xmpp_log_t;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_conn_t {
	_unused: [u8; 0],
}
pub type xmpp_conn_t = _xmpp_conn_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_stanza_t {
	_unused: [u8; 0],
}
pub type xmpp_stanza_t = _xmpp_stanza_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_sm_t {
	_unused: [u8; 0],
}
pub type xmpp_sm_state_t = _xmpp_sm_t;
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum xmpp_conn_event_t {
	XMPP_CONN_CONNECT = 0,
	XMPP_CONN_RAW_CONNECT = 1,
	XMPP_CONN_DISCONNECT = 2,
	XMPP_CONN_FAIL = 3,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum xmpp_error_type_t {
	XMPP_SE_BAD_FORMAT = 0,
	XMPP_SE_BAD_NS_PREFIX = 1,
	XMPP_SE_CONFLICT = 2,
	XMPP_SE_CONN_TIMEOUT = 3,
	XMPP_SE_HOST_GONE = 4,
	XMPP_SE_HOST_UNKNOWN = 5,
	XMPP_SE_IMPROPER_ADDR = 6,
	XMPP_SE_INTERNAL_SERVER_ERROR = 7,
	XMPP_SE_INVALID_FROM = 8,
	XMPP_SE_INVALID_ID = 9,
	XMPP_SE_INVALID_NS = 10,
	XMPP_SE_INVALID_XML = 11,
	XMPP_SE_NOT_AUTHORIZED = 12,
	XMPP_SE_POLICY_VIOLATION = 13,
	XMPP_SE_REMOTE_CONN_FAILED = 14,
	XMPP_SE_RESOURCE_CONSTRAINT = 15,
	XMPP_SE_RESTRICTED_XML = 16,
	XMPP_SE_SEE_OTHER_HOST = 17,
	XMPP_SE_SYSTEM_SHUTDOWN = 18,
	XMPP_SE_UNDEFINED_CONDITION = 19,
	XMPP_SE_UNSUPPORTED_ENCODING = 20,
	XMPP_SE_UNSUPPORTED_STANZA_TYPE = 21,
	XMPP_SE_UNSUPPORTED_VERSION = 22,
	XMPP_SE_XML_NOT_WELL_FORMED = 23,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum xmpp_cert_element_t {
	XMPP_CERT_VERSION = 0,
	XMPP_CERT_SERIALNUMBER = 1,
	XMPP_CERT_SUBJECT = 2,
	XMPP_CERT_ISSUER = 3,
	XMPP_CERT_NOTBEFORE = 4,
	XMPP_CERT_NOTAFTER = 5,
	XMPP_CERT_KEYALG = 6,
	XMPP_CERT_SIGALG = 7,
	XMPP_CERT_FINGERPRINT_SHA1 = 8,
	XMPP_CERT_FINGERPRINT_SHA256 = 9,
	XMPP_CERT_ELEMENT_MAX = 10,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct xmpp_stream_error_t {
	pub type_: xmpp_error_type_t,
	pub text: *mut ::std::os::raw::c_char,
	pub stanza: *mut xmpp_stanza_t,
}
#[test]
fn bindgen_test_layout_xmpp_stream_error_t() {
	const UNINIT: ::std::mem::MaybeUninit<xmpp_stream_error_t> = ::std::mem::MaybeUninit::uninit();
	let ptr = UNINIT.as_ptr();
	assert_eq!(
		::std::mem::size_of::<xmpp_stream_error_t>(),
		24usize,
		concat!("Size of: ", stringify!(xmpp_stream_error_t))
	);
	assert_eq!(
		::std::mem::align_of::<xmpp_stream_error_t>(),
		8usize,
		concat!("Alignment of ", stringify!(xmpp_stream_error_t))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize },
		0usize,
		concat!("Offset of field: ", stringify!(xmpp_stream_error_t), "::", stringify!(type_))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).text) as usize - ptr as usize },
		8usize,
		concat!("Offset of field: ", stringify!(xmpp_stream_error_t), "::", stringify!(text))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).stanza) as usize - ptr as usize },
		16usize,
		concat!("Offset of field: ", stringify!(xmpp_stream_error_t), "::", stringify!(stanza))
	);
}
pub type xmpp_conn_handler = ::std::option::Option<
	unsafe extern "C" fn(
		conn: *mut xmpp_conn_t,
		event: xmpp_conn_event_t,
		error: ::std::os::raw::c_int,
		stream_error: *mut xmpp_stream_error_t,
		userdata: *mut ::std::os::raw::c_void,
	),
>;
pub type xmpp_certfail_handler = ::std::option::Option<
	unsafe extern "C" fn(cert: *const xmpp_tlscert_t, errormsg: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int,
>;
pub type xmpp_password_callback = ::std::option::Option<
	unsafe extern "C" fn(
		pw: *mut ::std::os::raw::c_char,
		pw_max: usize,
		conn: *mut xmpp_conn_t,
		userdata: *mut ::std::os::raw::c_void,
	) -> ::std::os::raw::c_int,
>;
pub type xmpp_sockopt_callback = ::std::option::Option<
	unsafe extern "C" fn(conn: *mut xmpp_conn_t, sock: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int,
>;
extern "C" {
	pub fn xmpp_sockopt_cb_keepalive(conn: *mut xmpp_conn_t, sock: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_send_error(conn: *mut xmpp_conn_t, type_: xmpp_error_type_t, text: *mut ::std::os::raw::c_char);
}
extern "C" {
	pub fn xmpp_conn_new(ctx: *mut xmpp_ctx_t) -> *mut xmpp_conn_t;
}
extern "C" {
	pub fn xmpp_conn_clone(conn: *mut xmpp_conn_t) -> *mut xmpp_conn_t;
}
extern "C" {
	pub fn xmpp_conn_release(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_get_flags(conn: *const xmpp_conn_t) -> ::std::os::raw::c_long;
}
extern "C" {
	pub fn xmpp_conn_set_flags(conn: *mut xmpp_conn_t, flags: ::std::os::raw::c_long) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_get_jid(conn: *const xmpp_conn_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_conn_get_bound_jid(conn: *const xmpp_conn_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_conn_set_jid(conn: *mut xmpp_conn_t, jid: *const ::std::os::raw::c_char);
}
extern "C" {
	pub fn xmpp_conn_set_cafile(conn: *mut xmpp_conn_t, path: *const ::std::os::raw::c_char);
}
extern "C" {
	pub fn xmpp_conn_set_capath(conn: *mut xmpp_conn_t, path: *const ::std::os::raw::c_char);
}
extern "C" {
	pub fn xmpp_conn_set_certfail_handler(conn: *mut xmpp_conn_t, hndl: xmpp_certfail_handler);
}
extern "C" {
	pub fn xmpp_conn_get_peer_cert(conn: *mut xmpp_conn_t) -> *mut xmpp_tlscert_t;
}
extern "C" {
	pub fn xmpp_conn_set_password_callback(
		conn: *mut xmpp_conn_t,
		cb: xmpp_password_callback,
		userdata: *mut ::std::os::raw::c_void,
	);
}
extern "C" {
	pub fn xmpp_conn_set_password_retries(conn: *mut xmpp_conn_t, retries: ::std::os::raw::c_uint);
}
extern "C" {
	pub fn xmpp_conn_get_keyfile(conn: *const xmpp_conn_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_conn_set_client_cert(
		conn: *mut xmpp_conn_t,
		cert: *const ::std::os::raw::c_char,
		key: *const ::std::os::raw::c_char,
	);
}
extern "C" {
	pub fn xmpp_conn_cert_xmppaddr_num(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_uint;
}
extern "C" {
	pub fn xmpp_conn_cert_xmppaddr(conn: *mut xmpp_conn_t, n: ::std::os::raw::c_uint) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_conn_get_pass(conn: *const xmpp_conn_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_conn_set_pass(conn: *mut xmpp_conn_t, pass: *const ::std::os::raw::c_char);
}
extern "C" {
	pub fn xmpp_conn_get_context(conn: *mut xmpp_conn_t) -> *mut xmpp_ctx_t;
}
extern "C" {
	pub fn xmpp_conn_disable_tls(conn: *mut xmpp_conn_t);
}
extern "C" {
	pub fn xmpp_conn_is_secured(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_set_sockopt_callback(conn: *mut xmpp_conn_t, callback: xmpp_sockopt_callback);
}
extern "C" {
	pub fn xmpp_conn_is_connecting(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_is_connected(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_is_disconnected(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_send_queue_len(conn: *const xmpp_conn_t) -> ::std::os::raw::c_int;
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum xmpp_queue_element_t {
	XMPP_QUEUE_OLDEST = -1,
	XMPP_QUEUE_YOUNGEST = -2,
}
extern "C" {
	pub fn xmpp_conn_send_queue_drop_element(conn: *mut xmpp_conn_t, which: xmpp_queue_element_t) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_conn_get_sm_state(conn: *mut xmpp_conn_t) -> *mut xmpp_sm_state_t;
}
extern "C" {
	pub fn xmpp_conn_set_sm_state(conn: *mut xmpp_conn_t, sm_state: *mut xmpp_sm_state_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_free_sm_state(sm_state: *mut xmpp_sm_state_t);
}
extern "C" {
	pub fn xmpp_connect_client(
		conn: *mut xmpp_conn_t,
		altdomain: *const ::std::os::raw::c_char,
		altport: ::std::os::raw::c_ushort,
		callback: xmpp_conn_handler,
		userdata: *mut ::std::os::raw::c_void,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_connect_component(
		conn: *mut xmpp_conn_t,
		server: *const ::std::os::raw::c_char,
		port: ::std::os::raw::c_ushort,
		callback: xmpp_conn_handler,
		userdata: *mut ::std::os::raw::c_void,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_connect_raw(
		conn: *mut xmpp_conn_t,
		altdomain: *const ::std::os::raw::c_char,
		altport: ::std::os::raw::c_ushort,
		callback: xmpp_conn_handler,
		userdata: *mut ::std::os::raw::c_void,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_open_stream_default(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_open_stream(
		conn: *mut xmpp_conn_t,
		attributes: *mut *mut ::std::os::raw::c_char,
		attributes_len: usize,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_conn_tls_start(conn: *mut xmpp_conn_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_disconnect(conn: *mut xmpp_conn_t);
}
extern "C" {
	pub fn xmpp_send(conn: *mut xmpp_conn_t, stanza: *mut xmpp_stanza_t);
}
extern "C" {
	pub fn xmpp_send_raw_string(conn: *mut xmpp_conn_t, fmt: *const ::std::os::raw::c_char, ...);
}
extern "C" {
	pub fn xmpp_send_raw(conn: *mut xmpp_conn_t, data: *const ::std::os::raw::c_char, len: usize);
}
pub type xmpp_timed_handler = ::std::option::Option<
	unsafe extern "C" fn(conn: *mut xmpp_conn_t, userdata: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int,
>;
extern "C" {
	pub fn xmpp_timed_handler_add(
		conn: *mut xmpp_conn_t,
		handler: xmpp_timed_handler,
		period: ::std::os::raw::c_ulong,
		userdata: *mut ::std::os::raw::c_void,
	);
}
extern "C" {
	pub fn xmpp_timed_handler_delete(conn: *mut xmpp_conn_t, handler: xmpp_timed_handler);
}
pub type xmpp_global_timed_handler = ::std::option::Option<
	unsafe extern "C" fn(ctx: *mut xmpp_ctx_t, userdata: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int,
>;
extern "C" {
	pub fn xmpp_global_timed_handler_add(
		ctx: *mut xmpp_ctx_t,
		handler: xmpp_global_timed_handler,
		period: ::std::os::raw::c_ulong,
		userdata: *mut ::std::os::raw::c_void,
	);
}
extern "C" {
	pub fn xmpp_global_timed_handler_delete(ctx: *mut xmpp_ctx_t, handler: xmpp_global_timed_handler);
}
pub type xmpp_handler = ::std::option::Option<
	unsafe extern "C" fn(
		conn: *mut xmpp_conn_t,
		stanza: *mut xmpp_stanza_t,
		userdata: *mut ::std::os::raw::c_void,
	) -> ::std::os::raw::c_int,
>;
extern "C" {
	pub fn xmpp_handler_add(
		conn: *mut xmpp_conn_t,
		handler: xmpp_handler,
		ns: *const ::std::os::raw::c_char,
		name: *const ::std::os::raw::c_char,
		type_: *const ::std::os::raw::c_char,
		userdata: *mut ::std::os::raw::c_void,
	);
}
extern "C" {
	pub fn xmpp_handler_delete(conn: *mut xmpp_conn_t, handler: xmpp_handler);
}
extern "C" {
	pub fn xmpp_id_handler_add(
		conn: *mut xmpp_conn_t,
		handler: xmpp_handler,
		id: *const ::std::os::raw::c_char,
		userdata: *mut ::std::os::raw::c_void,
	);
}
extern "C" {
	pub fn xmpp_id_handler_delete(conn: *mut xmpp_conn_t, handler: xmpp_handler, id: *const ::std::os::raw::c_char);
}
extern "C" {
	pub fn xmpp_stanza_new(ctx: *mut xmpp_ctx_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_new_from_string(ctx: *mut xmpp_ctx_t, str_: *const ::std::os::raw::c_char) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_clone(stanza: *mut xmpp_stanza_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_copy(stanza: *const xmpp_stanza_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_release(stanza: *mut xmpp_stanza_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_get_context(stanza: *const xmpp_stanza_t) -> *mut xmpp_ctx_t;
}
extern "C" {
	pub fn xmpp_stanza_is_text(stanza: *mut xmpp_stanza_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_is_tag(stanza: *mut xmpp_stanza_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_to_text(
		stanza: *mut xmpp_stanza_t,
		buf: *mut *mut ::std::os::raw::c_char,
		buflen: *mut usize,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_get_children(stanza: *mut xmpp_stanza_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_get_child_by_name(stanza: *mut xmpp_stanza_t, name: *const ::std::os::raw::c_char) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_get_child_by_ns(stanza: *mut xmpp_stanza_t, ns: *const ::std::os::raw::c_char) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_get_child_by_name_and_ns(
		stanza: *mut xmpp_stanza_t,
		name: *const ::std::os::raw::c_char,
		ns: *const ::std::os::raw::c_char,
	) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_get_child_by_path(stanza: *mut xmpp_stanza_t, ...) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_get_next(stanza: *mut xmpp_stanza_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_add_child(stanza: *mut xmpp_stanza_t, child: *mut xmpp_stanza_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_add_child_ex(
		stanza: *mut xmpp_stanza_t,
		child: *mut xmpp_stanza_t,
		do_clone: ::std::os::raw::c_int,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_get_attribute(
		stanza: *mut xmpp_stanza_t,
		name: *const ::std::os::raw::c_char,
	) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_attribute_count(stanza: *mut xmpp_stanza_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_get_attributes(
		stanza: *mut xmpp_stanza_t,
		attr: *mut *const ::std::os::raw::c_char,
		attrlen: ::std::os::raw::c_int,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_get_text(stanza: *mut xmpp_stanza_t) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_text_ptr(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_name(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_set_attribute(
		stanza: *mut xmpp_stanza_t,
		key: *const ::std::os::raw::c_char,
		value: *const ::std::os::raw::c_char,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_name(stanza: *mut xmpp_stanza_t, name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_text(stanza: *mut xmpp_stanza_t, text: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_text_with_size(
		stanza: *mut xmpp_stanza_t,
		text: *const ::std::os::raw::c_char,
		size: usize,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_del_attribute(stanza: *mut xmpp_stanza_t, name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_get_ns(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_type(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_id(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_to(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_get_from(stanza: *mut xmpp_stanza_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_stanza_set_ns(stanza: *mut xmpp_stanza_t, ns: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_id(stanza: *mut xmpp_stanza_t, id: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_type(stanza: *mut xmpp_stanza_t, type_: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_to(stanza: *mut xmpp_stanza_t, to: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_set_from(stanza: *mut xmpp_stanza_t, from: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_stanza_reply(stanza: *mut xmpp_stanza_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_stanza_reply_error(
		stanza: *mut xmpp_stanza_t,
		error_type: *const ::std::os::raw::c_char,
		condition: *const ::std::os::raw::c_char,
		text: *const ::std::os::raw::c_char,
	) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_message_new(
		ctx: *mut xmpp_ctx_t,
		type_: *const ::std::os::raw::c_char,
		to: *const ::std::os::raw::c_char,
		id: *const ::std::os::raw::c_char,
	) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_message_get_body(msg: *mut xmpp_stanza_t) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_message_set_body(msg: *mut xmpp_stanza_t, text: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_iq_new(
		ctx: *mut xmpp_ctx_t,
		type_: *const ::std::os::raw::c_char,
		id: *const ::std::os::raw::c_char,
	) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_presence_new(ctx: *mut xmpp_ctx_t) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_error_new(
		ctx: *mut xmpp_ctx_t,
		type_: xmpp_error_type_t,
		text: *const ::std::os::raw::c_char,
	) -> *mut xmpp_stanza_t;
}
extern "C" {
	pub fn xmpp_jid_new(
		ctx: *mut xmpp_ctx_t,
		node: *const ::std::os::raw::c_char,
		domain: *const ::std::os::raw::c_char,
		resource: *const ::std::os::raw::c_char,
	) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_jid_bare(ctx: *mut xmpp_ctx_t, jid: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_jid_node(ctx: *mut xmpp_ctx_t, jid: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_jid_domain(ctx: *mut xmpp_ctx_t, jid: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_jid_resource(ctx: *mut xmpp_ctx_t, jid: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_run_once(ctx: *mut xmpp_ctx_t, timeout: ::std::os::raw::c_ulong);
}
extern "C" {
	pub fn xmpp_run(ctx: *mut xmpp_ctx_t);
}
extern "C" {
	pub fn xmpp_stop(ctx: *mut xmpp_ctx_t);
}
extern "C" {
	pub fn xmpp_ctx_set_timeout(ctx: *mut xmpp_ctx_t, timeout: ::std::os::raw::c_ulong);
}
extern "C" {
	pub fn xmpp_tlscert_get_ctx(cert: *const xmpp_tlscert_t) -> *mut xmpp_ctx_t;
}
extern "C" {
	pub fn xmpp_tlscert_get_conn(cert: *const xmpp_tlscert_t) -> *mut xmpp_conn_t;
}
extern "C" {
	pub fn xmpp_tlscert_get_pem(cert: *const xmpp_tlscert_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_tlscert_get_dnsname(cert: *const xmpp_tlscert_t, n: usize) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_tlscert_get_string(cert: *const xmpp_tlscert_t, elmnt: xmpp_cert_element_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_tlscert_get_description(elmnt: xmpp_cert_element_t) -> *const ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_tlscert_free(cert: *mut xmpp_tlscert_t);
}
extern "C" {
	pub fn xmpp_uuid_gen(ctx: *mut xmpp_ctx_t) -> *mut ::std::os::raw::c_char;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_sha1_t {
	_unused: [u8; 0],
}
pub type xmpp_sha1_t = _xmpp_sha1_t;
extern "C" {
	pub fn xmpp_sha1(ctx: *mut xmpp_ctx_t, data: *const ::std::os::raw::c_uchar, len: usize) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_sha1_digest(data: *const ::std::os::raw::c_uchar, len: usize, digest: *mut ::std::os::raw::c_uchar);
}
extern "C" {
	pub fn xmpp_sha1_new(ctx: *mut xmpp_ctx_t) -> *mut xmpp_sha1_t;
}
extern "C" {
	pub fn xmpp_sha1_free(sha1: *mut xmpp_sha1_t);
}
extern "C" {
	pub fn xmpp_sha1_update(sha1: *mut xmpp_sha1_t, data: *const ::std::os::raw::c_uchar, len: usize);
}
extern "C" {
	pub fn xmpp_sha1_final(sha1: *mut xmpp_sha1_t);
}
extern "C" {
	pub fn xmpp_sha1_to_string(sha1: *mut xmpp_sha1_t, s: *mut ::std::os::raw::c_char, slen: usize)
		-> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_sha1_to_string_alloc(sha1: *mut xmpp_sha1_t) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_sha1_to_digest(sha1: *mut xmpp_sha1_t, digest: *mut ::std::os::raw::c_uchar);
}
extern "C" {
	pub fn xmpp_base64_encode(
		ctx: *mut xmpp_ctx_t,
		data: *const ::std::os::raw::c_uchar,
		len: usize,
	) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_base64_decode_str(
		ctx: *mut xmpp_ctx_t,
		base64: *const ::std::os::raw::c_char,
		len: usize,
	) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_base64_decode_bin(
		ctx: *mut xmpp_ctx_t,
		base64: *const ::std::os::raw::c_char,
		len: usize,
		out: *mut *mut ::std::os::raw::c_uchar,
		outlen: *mut usize,
	);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _xmpp_rand_t {
	_unused: [u8; 0],
}
pub type xmpp_rand_t = _xmpp_rand_t;
extern "C" {
	pub fn xmpp_rand_new(ctx: *mut xmpp_ctx_t) -> *mut xmpp_rand_t;
}
extern "C" {
	pub fn xmpp_rand_free(ctx: *mut xmpp_ctx_t, rand: *mut xmpp_rand_t);
}
extern "C" {
	pub fn xmpp_rand(rand: *mut xmpp_rand_t) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_rand_bytes(rand: *mut xmpp_rand_t, output: *mut ::std::os::raw::c_uchar, len: usize);
}
extern "C" {
	pub fn xmpp_rand_nonce(rand: *mut xmpp_rand_t, output: *mut ::std::os::raw::c_char, len: usize);
}
pub type __gnuc_va_list = __builtin_va_list;
pub type va_list = __builtin_va_list;
extern "C" {
	pub fn xmpp_alloc(ctx: *const xmpp_ctx_t, size: usize) -> *mut ::std::os::raw::c_void;
}
extern "C" {
	pub fn xmpp_realloc(ctx: *const xmpp_ctx_t, p: *mut ::std::os::raw::c_void, size: usize) -> *mut ::std::os::raw::c_void;
}
extern "C" {
	pub fn xmpp_strdup(ctx: *const xmpp_ctx_t, s: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_strndup(ctx: *const xmpp_ctx_t, s: *const ::std::os::raw::c_char, len: usize) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_strtok_r(
		s: *mut ::std::os::raw::c_char,
		delim: *const ::std::os::raw::c_char,
		saveptr: *mut *mut ::std::os::raw::c_char,
	) -> *mut ::std::os::raw::c_char;
}
extern "C" {
	pub fn xmpp_snprintf(
		str_: *mut ::std::os::raw::c_char,
		count: usize,
		fmt: *const ::std::os::raw::c_char,
		...
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_vsnprintf(
		str_: *mut ::std::os::raw::c_char,
		count: usize,
		fmt: *const ::std::os::raw::c_char,
		arg: *mut __va_list_tag,
	) -> ::std::os::raw::c_int;
}
extern "C" {
	pub fn xmpp_log(
		ctx: *const xmpp_ctx_t,
		level: xmpp_log_level_t,
		area: *const ::std::os::raw::c_char,
		fmt: *const ::std::os::raw::c_char,
		ap: *mut __va_list_tag,
	);
}
extern "C" {
	pub fn xmpp_error(ctx: *const xmpp_ctx_t, area: *const ::std::os::raw::c_char, fmt: *const ::std::os::raw::c_char, ...);
}
extern "C" {
	pub fn xmpp_warn(ctx: *const xmpp_ctx_t, area: *const ::std::os::raw::c_char, fmt: *const ::std::os::raw::c_char, ...);
}
extern "C" {
	pub fn xmpp_info(ctx: *const xmpp_ctx_t, area: *const ::std::os::raw::c_char, fmt: *const ::std::os::raw::c_char, ...);
}
extern "C" {
	pub fn xmpp_debug(ctx: *const xmpp_ctx_t, area: *const ::std::os::raw::c_char, fmt: *const ::std::os::raw::c_char, ...);
}
extern "C" {
	pub fn xmpp_debug_verbose(
		level: ::std::os::raw::c_int,
		ctx: *const xmpp_ctx_t,
		area: *const ::std::os::raw::c_char,
		fmt: *const ::std::os::raw::c_char,
		...
	);
}
extern "C" {
	pub fn xmpp_conn_set_keepalive(conn: *mut xmpp_conn_t, timeout: ::std::os::raw::c_int, interval: ::std::os::raw::c_int);
}
pub type __builtin_va_list = [__va_list_tag; 1usize];
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __va_list_tag {
	pub gp_offset: ::std::os::raw::c_uint,
	pub fp_offset: ::std::os::raw::c_uint,
	pub overflow_arg_area: *mut ::std::os::raw::c_void,
	pub reg_save_area: *mut ::std::os::raw::c_void,
}
#[test]
fn bindgen_test_layout___va_list_tag() {
	const UNINIT: ::std::mem::MaybeUninit<__va_list_tag> = ::std::mem::MaybeUninit::uninit();
	let ptr = UNINIT.as_ptr();
	assert_eq!(
		::std::mem::size_of::<__va_list_tag>(),
		24usize,
		concat!("Size of: ", stringify!(__va_list_tag))
	);
	assert_eq!(
		::std::mem::align_of::<__va_list_tag>(),
		8usize,
		concat!("Alignment of ", stringify!(__va_list_tag))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).gp_offset) as usize - ptr as usize },
		0usize,
		concat!("Offset of field: ", stringify!(__va_list_tag), "::", stringify!(gp_offset))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).fp_offset) as usize - ptr as usize },
		4usize,
		concat!("Offset of field: ", stringify!(__va_list_tag), "::", stringify!(fp_offset))
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).overflow_arg_area) as usize - ptr as usize },
		8usize,
		concat!(
			"Offset of field: ",
			stringify!(__va_list_tag),
			"::",
			stringify!(overflow_arg_area)
		)
	);
	assert_eq!(
		unsafe { ::std::ptr::addr_of!((*ptr).reg_save_area) as usize - ptr as usize },
		16usize,
		concat!(
			"Offset of field: ",
			stringify!(__va_list_tag),
			"::",
			stringify!(reg_save_area)
		)
	);
}