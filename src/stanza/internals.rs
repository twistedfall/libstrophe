#[cfg(feature = "libstrophe-0_12_0")]
pub fn stanza_get_child_by_path(stanza: *mut sys::xmpp_stanza_t, path: &[&str]) -> *mut sys::xmpp_stanza_t {
	use std::os::raw::c_char;
	use std::ptr;

	use crate::ffi_types::FFI;

	macro_rules! call_with_paths {
		() => {
			unsafe {
				sys::xmpp_stanza_get_child_by_path(stanza, ptr::null::<*const c_char>())
			}
		};

		( $($path: ident),* ) => {
			{
				$(
					let $path = FFI($path).send();
				)*
				unsafe {
					sys::xmpp_stanza_get_child_by_path(stanza, $($path.as_ptr()),*, ptr::null::<*const c_char>())
				}
			}
		};
	}

	match *path {
		[] => {
			call_with_paths!()
		}
		[path0] => {
			call_with_paths!(path0)
		}
		[path0, path1] => {
			call_with_paths!(path0, path1)
		}
		[path0, path1, path2] => {
			call_with_paths!(path0, path1, path2)
		}
		[path0, path1, path2, path3] => {
			call_with_paths!(path0, path1, path2, path3)
		}
		[path0, path1, path2, path3, path4] => {
			call_with_paths!(path0, path1, path2, path3, path4)
		}
		[path0, path1, path2, path3, path4, path5] => {
			call_with_paths!(path0, path1, path2, path3, path4, path5)
		}
		[path0, path1, path2, path3, path4, path5, path6] => {
			call_with_paths!(path0, path1, path2, path3, path4, path5, path6)
		}
		[path0, path1, path2, path3, path4, path5, path6, path7] => {
			call_with_paths!(path0, path1, path2, path3, path4, path5, path6, path7)
		}
		[path0, path1, path2, path3, path4, path5, path6, path7, path8] => {
			call_with_paths!(path0, path1, path2, path3, path4, path5, path6, path7, path8)
		}
		[path0, path1, path2, path3, path4, path5, path6, path7, path8, path9] => {
			call_with_paths!(path0, path1, path2, path3, path4, path5, path6, path7, path8, path9)
		}
		_ => panic!("Maximum supported amount of elements in path is 10"),
	}
}
