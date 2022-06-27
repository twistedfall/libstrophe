use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr::NonNull;
use std::{alloc, ptr};

/// Internal `Context` that only specifies allocation functions and uses null logger. Needed to not pass
/// `Context` to e.g. `Stanza` because it uses only allocation functions from `Context`.
pub struct AllocContext {
	inner: NonNull<sys::xmpp_ctx_t>,
	_memory: Box<sys::xmpp_mem_t>,
}

type AllocUnit = usize;

impl AllocContext {
	#[inline(always)]
	fn calculate_layout(size: usize) -> alloc::Layout {
		// we leave additional sizeof(AllocUnit=usize) bytes in front for the actual allocation size, it's needed later for deallocation
		alloc::Layout::from_size_align(size + size_of::<AllocUnit>(), align_of::<AllocUnit>()).expect("Cannot create layout")
	}

	#[inline(always)]
	unsafe fn write_real_alloc(p: *mut u8, size: usize) -> *mut c_void {
		#![allow(clippy::cast_ptr_alignment)]
		// it's ok to cast it as *mut AllocUnit=usize because we align to it during allocation and p points to the beginning of that buffer
		let out = p as *mut AllocUnit;
		out.write(size);
		out.add(1) as _
	}

	#[inline(always)]
	unsafe fn read_real_alloc(p: *mut c_void) -> (*mut u8, alloc::Layout) {
		if p.is_null() {
			(
				p as _,
				alloc::Layout::from_size_align(0, align_of::<AllocUnit>()).expect("Cannot create layout"),
			)
		} else {
			let memory = (p as *mut AllocUnit).sub(1);
			let size = memory.read();
			(
				memory as _,
				alloc::Layout::from_size_align(size, align_of::<AllocUnit>()).expect("Cannot create layout"),
			)
		}
	}

	unsafe extern "C" fn custom_alloc(size: usize, _userdata: *mut c_void) -> *mut c_void {
		let layout = Self::calculate_layout(size);
		Self::write_real_alloc(alloc::alloc(layout), layout.size())
	}

	unsafe extern "C" fn custom_free(p: *mut c_void, _userdata: *mut c_void) {
		let (p, layout) = Self::read_real_alloc(p);
		alloc::dealloc(p, layout);
	}

	unsafe extern "C" fn custom_realloc(p: *mut c_void, size: usize, _userdata: *mut c_void) -> *mut c_void {
		let (p, layout) = Self::read_real_alloc(p);
		if size > 0 {
			let new_layout = Self::calculate_layout(size);
			let realloc_p = alloc::realloc(p, layout, new_layout.size());
			Self::write_real_alloc(realloc_p, new_layout.size())
		} else {
			if !p.is_null() {
				alloc::dealloc(p, layout);
			}
			ptr::null_mut()
		}
	}

	pub fn get_xmpp_mem_t() -> sys::xmpp_mem_t {
		sys::xmpp_mem_t {
			alloc: Some(Self::custom_alloc),
			free: Some(Self::custom_free),
			realloc: Some(Self::custom_realloc),
			userdata: ptr::null_mut(),
		}
	}

	pub(crate) fn as_ptr(&self) -> *mut sys::xmpp_ctx_t {
		self.inner.as_ptr()
	}

	/// [xmpp_free](https://strophe.im/libstrophe/doc/0.12.2/ctx_8c.html#acc734a5f5f115629c9e7775a4d3796e2)
	///
	/// # Safety
	/// p must be non-null and allocated by the libstrophe library (xmpp_alloc function)
	pub unsafe fn free<T>(&self, p: *mut T) {
		sys::xmpp_free(self.inner.as_ptr(), p as _)
	}
}

impl Default for AllocContext {
	fn default() -> Self {
		let memory = Box::new(Self::get_xmpp_mem_t());
		Self {
			inner: NonNull::new(unsafe { sys::xmpp_ctx_new(memory.as_ref(), ptr::null()) })
				.expect("Cannot allocate memory for Context"),
			_memory: memory,
		}
	}
}

unsafe impl Sync for AllocContext {}
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for AllocContext {}

#[cfg(test)]
mod alloc_test {
	use std::ptr::null_mut;

	use crate::AllocContext;

	#[test]
	fn test_alloc() {
		{
			let alloc_mem = unsafe { AllocContext::custom_alloc(10, null_mut()) };
			assert!(!alloc_mem.is_null());
			let realloc_mem = unsafe { AllocContext::custom_realloc(alloc_mem, 20, null_mut()) };
			assert!(!realloc_mem.is_null());
			let realloc_mem2 = unsafe { AllocContext::custom_realloc(realloc_mem, 20, null_mut()) };
			assert_eq!(realloc_mem2, realloc_mem);
			let realloc_mem3 = unsafe { AllocContext::custom_realloc(realloc_mem2, 10, null_mut()) };
			assert_eq!(realloc_mem3, realloc_mem2);
			unsafe {
				AllocContext::custom_free(realloc_mem3, null_mut());
			}
		}

		{
			let alloc_mem = unsafe { AllocContext::custom_realloc(null_mut(), 10, null_mut()) }; // equal to alloc
			assert!(!alloc_mem.is_null());
			unsafe {
				AllocContext::custom_free(alloc_mem, null_mut());
			}
		}

		{
			let alloc_mem = unsafe { AllocContext::custom_alloc(10, null_mut()) };
			assert!(!alloc_mem.is_null());
			let dealloc_mem = unsafe { AllocContext::custom_realloc(alloc_mem, 0, null_mut()) }; // equal to free
			assert!(dealloc_mem.is_null());
		}

		{
			let alloc_mem = unsafe { AllocContext::custom_realloc(null_mut(), 0, null_mut()) };
			assert!(alloc_mem.is_null());
		}
	}
}
