use std::{fmt, result, str};

use crate::{
	Connection,
	ffi_types::FFI,
	Stanza,
	StanzaMutRef,
};

#[derive(Debug, Fail)]
pub enum Error {
	#[fail(display = "Memory error")]
	MemoryError,
	#[fail(display = "Invalid operation")]
	InvalidOperation,
	#[fail(display = "Internal error")]
	InternalError,
}

#[derive(Debug)]
pub struct ConnectError<'cb, 'cx>{
	pub conn: Connection<'cb, 'cx>,
	pub error: failure::Error,
}

/// `Result` with failure `Error`
pub type Result<T> = result::Result<T, failure::Error>;

/// `Result` for methods that don't return any value on success
pub type EmptyResult = Result<()>;

impl From<i32> for Error {
	fn from(code: i32) -> Self {
		match code {
			sys::XMPP_EMEM => Error::MemoryError,
			sys::XMPP_EINVOP => Error::InvalidOperation,
			sys::XMPP_EINT => Error::InternalError,
			_ => panic!("Invalid value for error"),
		}
	}
}

/// Converts library-specific error code into an `EmptyResult`, for internal use
pub(crate) fn code_to_result(code: i32) -> EmptyResult {
	match code {
		sys::XMPP_EOK => Ok(()),
		_ => Err(Error::from(code).into()),
	}
}

#[derive(Debug)]
pub struct StreamError<'t, 's> {
	pub typ: sys::xmpp_error_type_t,
	pub text: Option<&'t str>,
	pub stanza: StanzaMutRef<'s>,
}

impl<'t> From<&'t sys::xmpp_stream_error_t> for StreamError<'t, 't> {
	fn from(inner: &'t sys::xmpp_stream_error_t) -> Self {
		StreamError {
			typ: inner.type_,
			text: unsafe { FFI(inner.text as _).receive() },
			stanza: unsafe { Stanza::from_inner_ref_mut(inner.stanza) }
		}
	}
}

impl fmt::Display for StreamError<'_, '_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let text_type = match self.typ {
			sys::xmpp_error_type_t::XMPP_SE_BAD_FORMAT => "Bad format",
			sys::xmpp_error_type_t::XMPP_SE_BAD_NS_PREFIX => "Bad namespace prefix",
			sys::xmpp_error_type_t::XMPP_SE_CONFLICT => "Conflict",
			sys::xmpp_error_type_t::XMPP_SE_CONN_TIMEOUT => "Connection timeout",
			sys::xmpp_error_type_t::XMPP_SE_HOST_GONE => "Gone",
			sys::xmpp_error_type_t::XMPP_SE_HOST_UNKNOWN => "Host unknown",
			sys::xmpp_error_type_t::XMPP_SE_IMPROPER_ADDR => "Improper address",
			sys::xmpp_error_type_t::XMPP_SE_INTERNAL_SERVER_ERROR => "Internal server error",
			sys::xmpp_error_type_t::XMPP_SE_INVALID_FROM => "Invalid from",
			sys::xmpp_error_type_t::XMPP_SE_INVALID_ID => "Invalid id",
			sys::xmpp_error_type_t::XMPP_SE_INVALID_NS => "Invalid namespace",
			sys::xmpp_error_type_t::XMPP_SE_INVALID_XML => "Invalid XML",
			sys::xmpp_error_type_t::XMPP_SE_NOT_AUTHORIZED => "Not authorized",
			sys::xmpp_error_type_t::XMPP_SE_POLICY_VIOLATION => "Policy violation",
			sys::xmpp_error_type_t::XMPP_SE_REMOTE_CONN_FAILED => "Connection failed",
			sys::xmpp_error_type_t::XMPP_SE_RESOURCE_CONSTRAINT => "Resource constraint",
			sys::xmpp_error_type_t::XMPP_SE_RESTRICTED_XML => "Restricted XML",
			sys::xmpp_error_type_t::XMPP_SE_SEE_OTHER_HOST => "See other host",
			sys::xmpp_error_type_t::XMPP_SE_SYSTEM_SHUTDOWN => "System shutdown",
			sys::xmpp_error_type_t::XMPP_SE_UNDEFINED_CONDITION => "Undefined condition",
			sys::xmpp_error_type_t::XMPP_SE_UNSUPPORTED_ENCODING => "Unsupported encoding",
			sys::xmpp_error_type_t::XMPP_SE_UNSUPPORTED_STANZA_TYPE => "Unsupported stanza type",
			sys::xmpp_error_type_t::XMPP_SE_UNSUPPORTED_VERSION => "Unsupported version",
			sys::xmpp_error_type_t::XMPP_SE_XML_NOT_WELL_FORMED => "XML is not well formed",
		};
		write!(f, "{}{}", text_type, self.text.map_or_else(|| "".into(), |x| format!(": {}", x)))
	}
}
