use std::mem::ManuallyDrop;
use std::ptr::NonNull;

pub struct SMState {
	inner: NonNull<sys::xmpp_sm_state_t>,
	owned: bool,
}

impl SMState {
	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_sm_state_t, owned: bool) -> Self {
		Self {
			inner: NonNull::new(inner).expect("Cannot allocate memory for SMState"),
			owned,
		}
	}

	#[inline]
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
