use std::{error, fmt, result, str};
use std::error::Error as StdError;
use super::{Stanza, StanzaMutRef, sys};
use super::ffi_types::FFI;

error_chain! {
	types {
		Error, ErrorKind, ResultExt;
	}

	errors {
		MemoryError
		InvalidOperation
		InternalError
	}

	foreign_links {
		Utf8Error(str::Utf8Error);
	}
}

/// `Result` with module-specific `Error`
pub type Result<T> = result::Result<T, Error>;

/// `Result` for methods that don't return any value
pub type EmptyResult = result::Result<(), Error>;

impl From<i32> for Error {
	fn from(code: i32) -> Self {
		match code {
			sys::XMPP_EMEM => ErrorKind::MemoryError.into(),
			sys::XMPP_EINVOP => ErrorKind::InvalidOperation.into(),
			sys::XMPP_EINT => ErrorKind::InternalError.into(),
			_ => panic!("Invalid value for error"),
		}
	}
}

/// Converts library-specific error code into an `EmptyResult`, for internal use
pub fn code_to_result(code: i32) -> EmptyResult {
	match code {
		sys::XMPP_EOK => Ok(()),
		_ => Err(code.into()),
	}
}

#[derive(Debug)]
pub struct StreamError<'i> {
	pub typ: sys::xmpp_error_type_t,
	pub text: Option<&'i str>,
	pub stanza: StanzaMutRef<'i>,
}

impl<'i> From<&'i sys::xmpp_stream_error_t> for StreamError<'i> {
	fn from(inner: &'i sys::xmpp_stream_error_t) -> Self {
		StreamError {
			typ: inner.type_,
			text: unsafe { FFI(inner.text as _).receive() },
			stanza: unsafe { Stanza::from_inner_ref_mut(inner.stanza) }
		}
	}
}

impl<'i> fmt::Display for StreamError<'i> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}{}", self.description(), self.text.as_ref().map_or_else(|| "".into(), |x| format!(": {}", x)))
	}
}

impl<'i> error::Error for StreamError<'i> {
	fn description(&self) -> &str {
		match self.typ {
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
		}
	}
}
