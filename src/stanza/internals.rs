#[cfg(feature = "libstrophe-0_12_0")]
pub fn stanza_get_child_by_path(stanza: *mut sys::xmpp_stanza_t, path: &[&str]) -> *mut sys::xmpp_stanza_t {
	use std::os::raw::c_char;
	use std::ptr;

	use crate::ffi_types::FFI;

	match *path {
		[path0] => unsafe {
			let path0 = FFI(path0).send();
			sys::xmpp_stanza_get_child_by_path(stanza, path0.as_ptr(), ptr::null::<*const c_char>())
		},
		[path0, path1] => unsafe {
			let path0 = FFI(path0).send();
			let path1 = FFI(path1).send();
			sys::xmpp_stanza_get_child_by_path(stanza, path0.as_ptr(), path1.as_ptr(), ptr::null::<*const c_char>())
		},
		[path0, path1, path2] => unsafe {
			let path0 = FFI(path0).send();
			let path1 = FFI(path1).send();
			let path2 = FFI(path2).send();
			sys::xmpp_stanza_get_child_by_path(
				stanza,
				path0.as_ptr(),
				path1.as_ptr(),
				path2.as_ptr(),
				ptr::null::<*const c_char>(),
			)
		},
		[path0, path1, path2, path3] => unsafe {
			let path0 = FFI(path0).send();
			let path1 = FFI(path1).send();
			let path2 = FFI(path2).send();
			let path3 = FFI(path3).send();
			sys::xmpp_stanza_get_child_by_path(
				stanza,
				path0.as_ptr(),
				path1.as_ptr(),
				path2.as_ptr(),
				path3.as_ptr(),
				ptr::null::<*const c_char>(),
			)
		},
		[path0, path1, path2, path3, path4] => unsafe {
			let path0 = FFI(path0).send();
			let path1 = FFI(path1).send();
			let path2 = FFI(path2).send();
			let path3 = FFI(path3).send();
			let path4 = FFI(path4).send();
			sys::xmpp_stanza_get_child_by_path(
				stanza,
				path0.as_ptr(),
				path1.as_ptr(),
				path2.as_ptr(),
				path3.as_ptr(),
				path4.as_ptr(),
				ptr::null::<*const c_char>(),
			)
		},
		[path0, path1, path2, path3, path4, path5] => unsafe {
			let path0 = FFI(path0).send();
			let path1 = FFI(path1).send();
			let path2 = FFI(path2).send();
			let path3 = FFI(path3).send();
			let path4 = FFI(path4).send();
			let path5 = FFI(path5).send();
			sys::xmpp_stanza_get_child_by_path(
				stanza,
				path0.as_ptr(),
				path1.as_ptr(),
				path2.as_ptr(),
				path3.as_ptr(),
				path4.as_ptr(),
				path5.as_ptr(),
				ptr::null::<*const c_char>(),
			)
		},
		_ => panic!("Maximum supported amount of elements in path is 6"),
	}
}
