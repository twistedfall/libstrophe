use std::{
	alloc,
	mem,
	os::raw,
	ptr,
	ptr::NonNull,
};

/// Internal `Context` that only specifies allocation functions and uses null logger. Needed to not pass
/// `Context` to e.g. `Stanza` because it uses onlyh allocation functions from `Context`.
pub struct AllocContext {
	inner: NonNull<sys::xmpp_ctx_t>,
	_memory: Box<sys::xmpp_mem_t>,
}

impl AllocContext {
	#[inline(always)]
	fn calculate_layout(size: usize) -> alloc::Layout {
		// we leave additional sizeof(usize) bytes in front for the actual allocation size, it's needed later for deallocation
		alloc::Layout::from_size_align(size + mem::size_of_val(&size), mem::align_of_val(&size)).expect("Cannot create layout")
	}

	#[inline(always)]
	fn write_real_alloc(p: *mut u8, size: usize) -> *mut raw::c_void {
		#![allow(clippy::cast_ptr_alignment)]
		// it's ok to cast it as *mut usize because we align to usize during allocation and p points to the beginning of that buffer
		let out = p as *mut usize;
		unsafe {
			out.write(size);
			out.add(1) as _
		}
	}

	#[inline(always)]
	fn read_real_alloc(p: *mut raw::c_void) -> (*mut u8, alloc::Layout) {
		let memory: *mut usize = unsafe { (p as *mut usize).sub(1) };
		let size = unsafe { memory.read() };
		(memory as _, alloc::Layout::from_size_align(size, mem::align_of_val(&size)).expect("Cannot create layout"))
	}

	extern "C" fn custom_alloc(size: usize, _userdata: *mut raw::c_void) -> *mut raw::c_void {
		let layout = Self::calculate_layout(size);
		Self::write_real_alloc(unsafe { alloc::alloc(layout) }, layout.size())
	}

	extern "C" fn custom_free(p: *mut raw::c_void, _userdata: *mut raw::c_void) {
		let (p, layout) = Self::read_real_alloc(p);
		unsafe { alloc::dealloc(p, layout) };
	}

	extern "C" fn custom_realloc(p: *mut raw::c_void, size: usize, _userdata: *mut raw::c_void) -> *mut raw::c_void {
		let new_layout = Self::calculate_layout(size);
		let (p, layout) = Self::read_real_alloc(p);
		Self::write_real_alloc(unsafe { alloc::realloc(p, layout, new_layout.size()) }, new_layout.size())
	}

	pub fn get_xmpp_mem_t() -> Box<sys::xmpp_mem_t> {
		Box::new(sys::xmpp_mem_t {
			alloc: Some(Self::custom_alloc),
			free: Some(Self::custom_free),
			realloc: Some(Self::custom_realloc),
			userdata: ptr::null_mut(),
		})
	}

	pub fn as_inner(&self) -> *mut sys::xmpp_ctx_t { self.inner.as_ptr() }

	/// [xmpp_free](https://github.com/strophe/libstrophe/blob/0.9.2/src/ctx.c#L214)
	pub unsafe fn free<T>(&self, p: *mut T) {
		sys::xmpp_free(self.inner.as_ptr(), p as _)
	}
}

impl Default for AllocContext {
	fn default() -> Self {
		let memory = Self::get_xmpp_mem_t();
		Self {
			inner: NonNull::new(unsafe { sys::xmpp_ctx_new(memory.as_ref(), ptr::null()) }).expect("Cannot allocate memory for Context"),
			_memory: memory,
		}
	}
}

unsafe impl Sync for AllocContext {}
