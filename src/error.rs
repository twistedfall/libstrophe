use core::error::Error as StdError;
use core::ffi::{c_char, c_int};
use core::fmt;
use core::str::Utf8Error;
use std::sync::Mutex;

use crate::{Connection, FFI, Stanza, StanzaMutRef};

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub enum Error {
	MemoryError,
	InvalidOperation,
	InternalError,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::MemoryError => write!(f, "Memory error"),
			Error::InvalidOperation => write!(f, "Invalid operation"),
			Error::InternalError => write!(f, "Internal error"),
		}
	}
}

impl StdError for Error {}

impl From<c_int> for Error {
	fn from(code: c_int) -> Self {
		match code {
			sys::XMPP_EMEM => Error::MemoryError,
			sys::XMPP_EINVOP => Error::InvalidOperation,
			sys::XMPP_EINT => Error::InternalError,
			_ => panic!("Invalid value for error"),
		}
	}
}

impl From<Error> for fmt::Error {
	fn from(_s: Error) -> Self {
		Self
	}
}

/// [std::result::Result] with the crate's [Error]
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub enum ToTextError {
	StropheError(Error),
	Utf8Error(Utf8Error),
}

impl fmt::Display for ToTextError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ToTextError::StropheError(e) => write!(f, "Strophe error: {e}"),
			ToTextError::Utf8Error(e) => write!(f, "UTF-8 error: {e}"),
		}
	}
}

impl StdError for ToTextError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			ToTextError::StropheError(e) => Some(e),
			ToTextError::Utf8Error(e) => Some(e),
		}
	}
}

impl From<Utf8Error> for ToTextError {
	fn from(s: Utf8Error) -> Self {
		ToTextError::Utf8Error(s)
	}
}

impl From<Error> for ToTextError {
	fn from(s: Error) -> Self {
		ToTextError::StropheError(s)
	}
}

#[derive(Debug)]
pub struct ConnectClientError<'cb, 'cx> {
	pub conn: Connection<'cb, 'cx>,
	pub error: Error,
}

fn error_type_to_str(typ: sys::xmpp_error_type_t) -> &'static str {
	match typ {
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

/// Error of the stream. Inspect the `typ` for the specific error type. `text` contains additional
/// text information about the error. `stanza` is the original error stanza sent by the server,
/// most probably you don't need to process it because data from it is already in `typ` and `text`.
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
			text: unsafe { FFI(inner.text as *const c_char).receive() },
			stanza: unsafe { Stanza::from_ref_mut(inner.stanza) },
		}
	}
}

impl fmt::Display for StreamError<'_, '_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", error_type_to_str(self.typ))?;
		if let Some(text) = self.text {
			write!(f, ": {text}")
		} else {
			Ok(())
		}
	}
}

impl StreamError<'_, '_> {
	pub fn to_owned(&self) -> OwnedStreamError {
		OwnedStreamError {
			typ: self.typ,
			text: self.text.map(|x| x.to_owned()),
			stanza: Mutex::new(self.stanza.clone()),
		}
	}
}

impl StdError for StreamError<'_, '_> {}

/// Owned version of [`StreamError`]. `stanza` is guarded by Mutex to make the error type `Sync`.
///
/// [`StreamError`]: struct.StreamError.html
#[derive(Debug)]
pub struct OwnedStreamError {
	pub typ: sys::xmpp_error_type_t,
	pub text: Option<String>,
	pub stanza: Mutex<Stanza>,
}

impl From<&StreamError<'_, '_>> for OwnedStreamError {
	fn from(s: &StreamError) -> Self {
		s.to_owned()
	}
}

impl From<StreamError<'_, '_>> for OwnedStreamError {
	fn from(s: StreamError) -> Self {
		s.to_owned()
	}
}

impl fmt::Display for OwnedStreamError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", error_type_to_str(self.typ))?;
		if let Some(ref text) = self.text {
			write!(f, ": {text}")
		} else {
			Ok(())
		}
	}
}

impl Clone for OwnedStreamError {
	fn clone(&self) -> Self {
		OwnedStreamError {
			typ: self.typ,
			text: self.text.clone(),
			stanza: Mutex::new(self.stanza.lock().expect("Cannot lock Mutex for cloning").clone()),
		}
	}
}

impl StdError for OwnedStreamError {}

/// Converts library-specific error code into an `Result<()>`, for internal use
pub(crate) trait IntoResult {
	fn into_result(self) -> Result<()>;
}

impl IntoResult for c_int {
	fn into_result(self) -> Result<()> {
		match self {
			sys::XMPP_EOK => Ok(()),
			_ => Err(Error::from(self)),
		}
	}
}

#[derive(Debug)]
pub enum ConnectionError<'t, 's> {
	Aborted,
	TimedOut,
	ConnectionReset,
	TLS(i32),
	Stream(StreamError<'t, 's>),
}

impl<'t, 's> ConnectionError<'t, 's> {
	pub(crate) fn from((code, stream_error): (i32, Option<StreamError<'t, 's>>)) -> Option<Self> {
		match code {
			0 => stream_error.map(ConnectionError::Stream),
			103 /* ECONNABORTED */ => Some(ConnectionError::Aborted),
			104 /* ECONNRESET */ => Some(ConnectionError::ConnectionReset),
			110 /* ETIMEDOUT */ => Some(ConnectionError::TimedOut),
			code => Some(ConnectionError::TLS(code)),
		}
	}
}

impl fmt::Display for ConnectionError<'_, '_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ConnectionError::Aborted => write!(f, "Connection aborted"),
			ConnectionError::TimedOut => write!(f, "Connection timed out"),
			ConnectionError::ConnectionReset => write!(f, "Connection reset"),
			ConnectionError::TLS(e) => write!(f, "TLS error: {e}"),
			ConnectionError::Stream(e) => write!(f, "Stream error: {e}"),
		}
	}
}

impl StdError for ConnectionError<'_, '_> {}

#[derive(Clone, Debug)]
pub enum OwnedConnectionError {
	Aborted,
	TimedOut,
	ConnectionReset,
	TLS(i32),
	Stream(OwnedStreamError),
}

impl From<ConnectionError<'_, '_>> for OwnedConnectionError {
	fn from(s: ConnectionError<'_, '_>) -> Self {
		match s {
			ConnectionError::Aborted => OwnedConnectionError::Aborted,
			ConnectionError::TimedOut => OwnedConnectionError::TimedOut,
			ConnectionError::ConnectionReset => OwnedConnectionError::ConnectionReset,
			ConnectionError::TLS(e) => OwnedConnectionError::TLS(e),
			ConnectionError::Stream(e) => OwnedConnectionError::Stream(e.into()),
		}
	}
}

impl fmt::Display for OwnedConnectionError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			OwnedConnectionError::Aborted => write!(f, "Connection aborted"),
			OwnedConnectionError::TimedOut => write!(f, "Connection timed out"),
			OwnedConnectionError::ConnectionReset => write!(f, "Connection reset"),
			OwnedConnectionError::TLS(e) => write!(f, "TLS error: {e}"),
			OwnedConnectionError::Stream(e) => write!(f, "Stream error: {e}"),
		}
	}
}

impl StdError for OwnedConnectionError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		if let OwnedConnectionError::Stream(e) = self {
			Some(e)
		} else {
			None
		}
	}
}
