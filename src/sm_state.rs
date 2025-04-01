#[cfg(feature = "libstrophe-0_14")]
use core::ffi::c_uchar;
#[cfg(feature = "libstrophe-0_14")]
use core::fmt;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

pub struct SMState {
	inner: NonNull<sys::xmpp_sm_state_t>,
	owned: bool,
}

impl SMState {
	unsafe fn with_inner(inner: *mut sys::xmpp_sm_state_t, owned: bool) -> Self {
		Self {
			inner: NonNull::new(inner).expect("Cannot allocate memory for SMState"),
			owned,
		}
	}

	pub(super) unsafe fn from_owned(inner: *mut sys::xmpp_sm_state_t) -> Self {
		Self::with_inner(inner, true)
	}

	pub(super) fn into_inner(self) -> *mut sys::xmpp_sm_state_t {
		unsafe { ManuallyDrop::new(self).inner.as_mut() }
	}
}

impl Drop for SMState {
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				sys::xmpp_free_sm_state(self.inner.as_mut());
			}
		}
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
impl fmt::Debug for SerializedSmState {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("SerializedSmState")
			.field("buf", &format!("{} bytes", self.buf.len()))
			.finish()
	}
}
