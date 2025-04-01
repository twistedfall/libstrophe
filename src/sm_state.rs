#[cfg(feature = "libstrophe-0_14")]
use core::ffi::c_uchar;
#[cfg(feature = "libstrophe-0_14")]
use core::fmt;
use core::marker::PhantomData;
use core::ptr::NonNull;

/// Stream Management state of a connection object
pub struct SMState<'cb, 'cx> {
	inner: NonNull<sys::xmpp_sm_state_t>,
	_conn: PhantomData<(&'cb (), &'cx ())>,
}

impl SMState<'_, '_> {
	pub(super) unsafe fn new(inner: *mut sys::xmpp_sm_state_t) -> Self {
		Self {
			inner: NonNull::new(inner).expect("Cannot allocate memory for SMState"),
			_conn: PhantomData,
		}
	}
}

impl Drop for SMState<'_, '_> {
	fn drop(&mut self) {
		unsafe { sys::xmpp_free_sm_state(self.inner.as_mut()) }
	}
}

#[cfg(feature = "libstrophe-0_14")]
pub struct SerializedSmStateRef<'b> {
	pub(crate) buf: &'b [c_uchar],
}

#[cfg(feature = "libstrophe-0_14")]
impl<'b> SerializedSmStateRef<'b> {
	pub fn to_owned(&self) -> SerializedSmState {
		SerializedSmState { buf: self.buf.to_vec() }
	}

	pub fn as_slice(&self) -> &'b [c_uchar] {
		self.buf
	}
}

#[cfg(feature = "libstrophe-0_14")]
impl fmt::Debug for SerializedSmStateRef<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("SerializedSmStateRef")
			.field("buf", &format!("{} bytes", self.buf.len()))
			.finish()
	}
}

#[cfg(feature = "libstrophe-0_14")]
pub struct SerializedSmState {
	pub(crate) buf: Vec<c_uchar>,
}

#[cfg(feature = "libstrophe-0_14")]
impl SerializedSmState {
	pub fn as_slice(&self) -> &[c_uchar] {
		&self.buf
	}
}

#[cfg(feature = "libstrophe-0_14")]
impl fmt::Debug for SerializedSmState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("SerializedSmState")
			.field("buf", &format!("{} bytes", self.buf.len()))
			.finish()
	}
}
