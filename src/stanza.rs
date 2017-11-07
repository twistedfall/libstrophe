use std::{collections, ffi, fmt, marker, mem, ops, ptr, slice};
use std::os::raw;
use std::sync::Arc;

use super::{
	error,
	sys,
	Context,
	ContextRef,
};
use super::ffi_types::FFI;

/// Proxy to underlying `xmpp_stanza_t` struct.
///
/// Most of the methods in this struct mimic the methods of the underlying library. So please see
/// libstrophe [outdated docs] and [sources]. Only where it's not the case or there is some
/// additional logic involved then you can see the method description.
///
/// This struct implements:
///
///   * `Display` ([xmpp_stanza_to_text])
///   * `Eq` by comparing internal pointers
///   * `Drop` ([xmpp_stanza_release])
///   * `Clone` ([xmpp_stanza_copy])
///   * `Send`
///
/// [outdated docs]: http://strophe.im/libstrophe/doc/0.8-snapshot/group___stanza.html
/// [sources]: https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L16
/// [xmpp_stanza_to_text]: https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L399
/// [xmpp_stanza_release]: https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L163
/// [xmpp_stanza_copy]: https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L110
#[derive(Debug, Hash)]
pub struct Stanza<'cx> {
	inner: *mut sys::xmpp_stanza_t,
	owned: bool,
	_ctx: marker::PhantomData<&'cx Context<'cx>>
}

impl<'cx> Stanza<'cx> {
	/// [xmpp_stanza_new](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L26)
	///
	/// The newly created stanza is not really useful until you assign an internal type to it. To do
	/// that you must call [`set_text()`] to make it `XMPP_STANZA_TEXT` stanza or [`set_name()`] to make
	/// it `XMPP_STANZA_TAG` stanza.
	///
	/// [`set_text()`]: struct.Stanza.html#method.set_text
	/// [`set_name()`]: struct.Stanza.html#method.set_name
	pub fn new(ctx: &'cx Context) -> Stanza<'cx> {
		unsafe { Stanza::from_inner(sys::xmpp_stanza_new(ctx.as_inner() as *mut _)) }
	}

	/// [xmpp_presence_new](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1205)
	pub fn new_presence(ctx: &'cx Context) -> Stanza<'cx> {
		unsafe { Stanza::from_inner(sys::xmpp_presence_new(ctx.as_inner() as *mut _)) }
	}

	/// [xmpp_iq_new](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1188)
	pub fn new_iq(ctx: &'cx Context, typ: Option<&str>, id: Option<&str>) -> Stanza<'cx>
	{
		let typ = FFI(typ).send();
		let id = FFI(id).send();
		unsafe {
			Stanza::from_inner(
				sys::xmpp_iq_new(
					ctx.as_inner() as *mut _,
					typ.as_ptr(),
					id.as_ptr()
				)
			)
		}
	}

	/// [xmpp_message_new](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1101)
	pub fn new_message(ctx: &'cx Context, typ: Option<&str>, id: Option<&str>, to: Option<&str>) -> Stanza<'cx>
	{
		let typ = FFI(typ).send();
		let to = FFI(to).send();
		let id = FFI(id).send();
		unsafe {
			Stanza::from_inner(
				sys::xmpp_message_new(
					ctx.as_inner() as *mut _,
					typ.as_ptr(),
					to.as_ptr(),
					id.as_ptr(),
				)
			)
		}
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_stanza_t, owned: bool) -> Stanza<'cx> {
		if inner.is_null() {
			panic!("Cannot allocate memory for Stanza")
		}
		Stanza { inner, owned, _ctx: marker::PhantomData }
	}

	/// Create an owning stanza from the raw pointer, for internal use
	pub unsafe fn from_inner(inner: *mut sys::xmpp_stanza_t) -> Stanza<'cx> {
		Stanza::with_inner(inner, true)
	}

	/// Create a borrowing stanza from the constant raw pointer, for internal use
	pub unsafe fn from_inner_ref(inner: *const sys::xmpp_stanza_t) -> StanzaRef<'cx> {
		Stanza::with_inner(inner as *mut _, false).into()
	}

	/// Create a borrowing stanza from the mutable raw pointer, for internal use
	pub unsafe fn from_inner_ref_mut(inner: *mut sys::xmpp_stanza_t) -> StanzaMutRef<'cx> {
		Stanza::with_inner(inner, false).into()
	}

	/// Return internal raw pointer to stanza, for internal use
	pub fn as_inner(&self) -> *const sys::xmpp_stanza_t { self.inner }

	/// Return context for this `Stanza`
	///
	/// The underlying library does not provide direct access to its context so this method works
	/// this around by relying on some of the library internals. With the new version this might need
	/// rewriting.
	pub fn context(&self) -> ContextRef<'cx> {
		// hack to reach unexposed context reference stored inside C structure
		#[repr(C)]
		struct StanzaLayout {
			repr: raw::c_int,
			ctx: *mut sys::xmpp_ctx_t,
		}
		let ctx_ref = unsafe {
			(*(self.inner as *mut StanzaLayout)).ctx
		};
		Arc::new(unsafe { Context::from_inner_ref_mut(ctx_ref) }).into()
	}

	/// [xmpp_stanza_is_text](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L199)
	pub fn is_text(&self) -> bool {
		FFI(unsafe { sys::xmpp_stanza_is_text(self.inner) }).receive_bool()
	}

	/// [xmpp_stanza_is_tag](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L212)
	pub fn is_tag(&self) -> bool {
		FFI(unsafe { sys::xmpp_stanza_is_tag(self.inner) }).receive_bool()
	}

	/// [xmpp_stanza_to_text](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L399)
	pub fn to_text(&self) -> error::Result<String> {
		let buf: *mut raw::c_char = unsafe { mem::uninitialized() };
		let mut buflen: usize = unsafe { mem::uninitialized() };
		error::code_to_result(unsafe {
			sys::xmpp_stanza_to_text(self.inner as *const _ as *mut _, &buf, &mut buflen)
		}).and_then(|_| {
			let buf = unsafe { ffi::CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(buf as *mut u8, buflen + 1)) };
			let out = buf.to_str()?.to_owned();
			unsafe {
				self.context().free(buf.as_ptr() as *mut raw::c_char);
			}
			Ok(out)
		})
	}

	/// [xmpp_stanza_set_name](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L457)
	///
	/// Be aware that calling this method changes the internal type of stanza to `XMPP_STANZA_TAG`.
	pub fn set_name<RefStr: AsRef<str>>(&mut self, name: RefStr) -> error::EmptyResult {
		let name = FFI(name.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_name(self.inner, name.as_ptr())
		})
	}

	/// [xmpp_stanza_get_name](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L480)
	pub fn name(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_stanza_get_name(self.inner)).receive()
		}
	}

	/// [xmpp_stanza_get_attribute_count](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L496)
	pub fn attribute_count(&self) -> i32 {
		unsafe {
			sys::xmpp_stanza_get_attribute_count(self.inner)
		}
	}

	/// [xmpp_stanza_set_attribute](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L558)
	pub fn set_attribute<RefStr1: AsRef<str>, RefStr2: AsRef<str>>(&mut self, name: RefStr1, value: RefStr2) -> error::EmptyResult {
		let name = FFI(name.as_ref()).send();
		let value = FFI(value.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_attribute(self.inner, name.as_ptr(), value.as_ptr())
		})
	}

	/// [xmpp_stanza_get_attribute](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L992)
	pub fn get_attribute<RefStr: AsRef<str>>(&self, name: RefStr) -> Option<&str> {
		let name = FFI(name.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_stanza_get_attribute(self.inner, name.as_ptr())).receive()
		}
	}

	/// [xmpp_stanza_get_attributes](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L513)
	///
	/// This method returns data as `HashMap` unlike underlying function.
	pub fn attributes(&self) -> collections::HashMap<&str, &str> {
		let count = self.attribute_count();
		let mut out = collections::HashMap::with_capacity(count as usize);
		unsafe {
			let mut arr = vec![ptr::null() as *const raw::c_char; count as usize * 2];
			sys::xmpp_stanza_get_attributes(self.inner, arr.as_mut_ptr(), count * 2);
			let mut iter = arr.into_iter();
			loop {
				if let Some(key) = iter.next() {
					if let Some(val) = iter.next() {
						out.insert(FFI(key).receive().unwrap(), FFI(val).receive().unwrap());
					} else {
						break
					}
				} else {
					break
				}
			}
		}
		out
	}

	/// [xmpp_stanza_del_attribute](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1015)
	pub fn del_attribute<RefStr: AsRef<str>>(&mut self, name: RefStr) -> error::EmptyResult {
		let name = FFI(name.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_del_attribute(self.inner, name.as_ptr())
		})
	}

	/// [xmpp_stanza_set_text_with_size](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L667)
	///
	/// Be aware that calling this method changes the internal type of stanza to `XMPP_STANZA_TEXT`.
	pub fn set_text<RefStr: AsRef<str>>(&mut self, text: RefStr) -> error::EmptyResult {
		let text = text.as_ref();
		error::code_to_result(unsafe { sys::xmpp_stanza_set_text_with_size(self.inner, text.as_ptr() as *const raw::c_char, text.len()) })
	}

	/// [xmpp_stanza_get_text](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L854)
	pub fn text(&self) -> Option<String> {
		unsafe {
			let text = sys::xmpp_stanza_get_text(self.inner);
			text.as_mut().and_then(|x| {
				let out = ffi::CStr::from_ptr(x).to_owned().into_string().ok();
				self.context().free(x);
				out
			})
		}
	}


	/// [xmpp_stanza_set_id](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L921)
	pub fn set_id<RefStr: AsRef<str>>(&mut self, id: RefStr) -> error::EmptyResult {
		let id = FFI(id.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_id(self.inner, id.as_ptr())
		})
	}

	/// [xmpp_stanza_get_id](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L699)
	pub fn id(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_id(self.inner)).receive() } }

	/// [xmpp_stanza_set_ns](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L592)
	pub fn set_ns<RefStr: AsRef<str>>(&mut self, ns: RefStr) -> error::EmptyResult {
		let ns = FFI(ns.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_ns(self.inner, ns.as_ptr())
		})
	}

	/// [xmpp_stanza_get_ns](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L714)
	pub fn ns(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_ns(self.inner)).receive() } }

	/// [xmpp_stanza_set_type](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L939)
	pub fn set_stanza_type<RefStr: AsRef<str>>(&mut self, typ: RefStr) -> error::EmptyResult {
		let typ = FFI(typ.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_type(self.inner, typ.as_ptr())
		})
	}

	/// [xmpp_stanza_get_type](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L729)
	pub fn stanza_type(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_type(self.inner)).receive() } }

	/// [xmpp_stanza_set_to](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L956)
	pub fn set_to<RefStr: AsRef<str>>(&mut self, to: RefStr) -> error::EmptyResult {
		let to = FFI(to.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_to(self.inner, to.as_ptr())
		})
	}

	/// [xmpp_stanza_get_to](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L744)
	pub fn to(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_to(self.inner)).receive() } }

	/// [xmpp_stanza_set_from](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L974)
	pub fn set_from<RefStr: AsRef<str>>(&mut self, from: RefStr) -> error::EmptyResult {
		let from = FFI(from.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_from(self.inner, from.as_ptr())
		})
	}

	/// [xmpp_stanza_get_from](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L759)
	pub fn from(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_from(self.inner)).receive() } }

	/// [xmpp_stanza_get_children](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L825)
	pub fn get_first_child(&self) -> Option<StanzaRef> {
		unsafe {
			sys::xmpp_stanza_get_children(self.inner).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_children](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L825)
	pub fn get_first_child_mut(&self) -> Option<StanzaMutRef> {
		unsafe {
			sys::xmpp_stanza_get_children(self.inner).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	/// [xmpp_stanza_get_child_by_ns](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L799)
	pub fn get_child_by_ns<RefStr: AsRef<str>>(&self, ns: RefStr) -> Option<StanzaRef> {
		let ns = FFI(ns.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_ns(self.inner, ns.as_ptr()).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_child_by_ns](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L799)
	pub fn get_child_by_ns_mut<RefStr: AsRef<str>>(&mut self, ns: RefStr) -> Option<StanzaMutRef> {
		let ns = FFI(ns.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_ns(self.inner, ns.as_ptr()).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	/// [xmpp_stanza_get_child_by_name](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L774)
	pub fn get_child_by_name<RefStr: AsRef<str>>(&self, name: RefStr) -> Option<StanzaRef> {
		let name = FFI(name.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_name(self.inner, name.as_ptr()).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_child_by_name](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L774)
	pub fn get_child_by_name_mut<RefStr: AsRef<str>>(&mut self, name: RefStr) -> Option<StanzaMutRef> {
		let name = FFI(name.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_name(self.inner, name.as_ptr()).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	// todo children iterator

	/// [xmpp_stanza_get_next](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L841)
	pub fn get_next(&self) -> Option<StanzaRef> {
		unsafe {
			sys::xmpp_stanza_get_next(self.inner).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_next](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L841)
	pub fn get_next_mut(&mut self) -> Option<StanzaMutRef> {
		unsafe {
			sys::xmpp_stanza_get_next(self.inner).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	/// [xmpp_stanza_add_child](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L609)
	pub fn add_child(&mut self, child: Stanza) -> error::EmptyResult {
		error::code_to_result(unsafe {
			sys::xmpp_stanza_add_child(self.inner, child.inner as *const _ as *mut _)
		})
	}

	/// [xmpp_stanza_reply](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1036)
	pub fn reply(&self) -> Stanza<'cx> {
		unsafe {
			Stanza::from_inner(sys::xmpp_stanza_reply(self.inner))
		}
	}

	/// [xmpp_message_set_body](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1144)
	pub fn set_body<RefStr: AsRef<str>>(&mut self, body: RefStr) -> error::EmptyResult {
		let body = FFI(body.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_message_set_body(self.inner, body.as_ptr())
		})
	}

	/// [xmpp_message_get_body](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L1119)
	pub fn body(&self) -> Option<String> {
		unsafe {
			let text = sys::xmpp_message_get_body(self.inner as *const _ as *mut _);
			text.as_mut().and_then(|x| {
				let out = ffi::CStr::from_ptr(x).to_owned().into_string().ok();
				self.context().free(x);
				out
			})
		}
	}
}

impl<'cx> fmt::Display for Stanza<'cx> {
	/// [xmpp_stanza_to_text](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L399)
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let buf: *mut raw::c_char = unsafe { mem::uninitialized() };
		let mut buflen: usize = unsafe { mem::uninitialized() };
		let result = error::code_to_result(unsafe { sys::xmpp_stanza_to_text(self.inner as *const _ as *mut _, &buf, &mut buflen) });
		if result.is_ok() {
			let buf = unsafe { ffi::CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(buf as *mut u8, buflen + 1)) };
			let out = write!(f, "{}", buf.to_str().map_err(|_| fmt::Error)?);
			unsafe {
				self.context().free(buf.as_ptr() as *mut raw::c_char);
			}
			out
		} else {
			Err(fmt::Error)
		}
	}
}

impl<'cx> Clone for Stanza<'cx> {
	fn clone(&self) -> Self {
		unsafe { Stanza::from_inner(sys::xmpp_stanza_copy(self.inner)) }
	}
}

impl<'cx> PartialEq for Stanza<'cx> {
	fn eq(&self, other: &Stanza) -> bool {
		self.inner == other.inner
	}
}

impl<'cx> Eq for Stanza<'cx> {}

impl<'cx> Drop for Stanza<'cx> {
	/// [xmpp_stanza_release](https://github.com/strophe/libstrophe/blob/0.9.1/src/stanza.c#L163)
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				sys::xmpp_stanza_release(self.inner);
			}
		}
	}
}

unsafe impl<'cx> Send for Stanza<'cx> {}

impl<'cx> Into<StanzaRef<'cx>> for Stanza<'cx> {
	fn into(self) -> StanzaRef<'cx> {
		StanzaRef(self)
	}
}

impl<'cx> Into<StanzaMutRef<'cx>> for Stanza<'cx> {
	fn into(self) -> StanzaMutRef<'cx> {
		StanzaMutRef(self)
	}
}

/// Wrapper for constant ref to [`Stanza`], implements `Deref` to [`Stanza`]
///
/// You can obtain such objects by calling [`Stanza`] child search methods.
///
/// [`Stanza`]: struct.Stanza.html
#[derive(Debug)]
pub struct StanzaRef<'cx>(Stanza<'cx>);

impl<'cx> ops::Deref for StanzaRef<'cx> {
	type Target = Stanza<'cx>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}


/// Wrapper for mutable ref to [`Stanza`], implements `Deref` to [`Stanza`]
///
/// You can obtain such objects by calling [`Stanza`] child search methods.
///
/// [`Stanza`]: struct.Stanza.html
#[derive(Debug)]
pub struct StanzaMutRef<'cx>(Stanza<'cx>);

impl<'cx> ops::Deref for StanzaMutRef<'cx> {
	type Target = Stanza<'cx>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'cx> ops::DerefMut for StanzaMutRef<'cx> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
