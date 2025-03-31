use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr::NonNull;
use std::{fmt, ops, ptr, slice};

use crate::error::IntoResult;
use crate::{Error, ErrorType, Result, ToTextError, ALLOC_CONTEXT, FFI};

mod internals;

/// Proxy to the underlying `xmpp_stanza_t` struct.
///
/// Most of the methods in this struct mimic the methods of the underlying library. So please see
/// libstrophe [docs] and [sources]. Only where it's not the case or there is some additional logic
/// involved then you can see the method description.
///
/// This struct implements:
///
///   * `Display` ([xmpp_stanza_to_text])
///   * `Eq` by comparing internal pointers
///   * `Hash` by hashing internal pointer
///   * `Drop` ([xmpp_stanza_release])
///   * `Clone` ([xmpp_stanza_copy])
///   * `Send`
///
/// [docs]: https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html
/// [sources]: https://github.com/strophe/libstrophe/blob/0.12.2/src/stanza.c
/// [xmpp_stanza_to_text]: https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga2918484877ac34d483cc14cf5e957fad
/// [xmpp_stanza_release]: https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gaa231317e56af0b974d7a28793ebefb83
/// [xmpp_stanza_copy]: https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga1ab1122fdf477d9d755fc094b4fbf6b6
#[derive(Debug, Eq)]
pub struct Stanza {
	inner: NonNull<sys::xmpp_stanza_t>,
	owned: bool,
}

impl Stanza {
	#[inline]
	/// [xmpp_stanza_new](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga0422973c3e2f6851c1ff1868e476f118)
	///
	/// The newly created stanza is not really useful until you assign an internal type to it. To do
	/// that you must call [`set_text()`] to make it `XMPP_STANZA_TEXT` stanza or [`set_name()`] to make
	/// it `XMPP_STANZA_TAG` stanza.
	///
	/// [`set_text()`]: struct.Stanza.html#method.set_text
	/// [`set_name()`]: struct.Stanza.html#method.set_name
	pub fn new() -> Self {
		unsafe { Stanza::from_owned(sys::xmpp_stanza_new(ALLOC_CONTEXT.as_ptr())) }
	}

	#[inline]
	/// [xmpp_presence_new](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gac47cc19072d39056d24d579b3a96b5db)
	pub fn new_presence() -> Self {
		unsafe { Stanza::from_owned(sys::xmpp_presence_new(ALLOC_CONTEXT.as_ptr())) }
	}

	#[inline]
	/// [xmpp_iq_new](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gad3fccc89f1367c5014dce97238d75f4d)
	pub fn new_iq(typ: Option<&str>, id: Option<&str>) -> Self {
		let typ = FFI(typ).send();
		let id = FFI(id).send();
		unsafe { Stanza::from_owned(sys::xmpp_iq_new(ALLOC_CONTEXT.as_ptr(), typ.as_ptr(), id.as_ptr())) }
	}

	#[inline]
	/// [xmpp_message_new](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga850783185475f4324423f2af876774eb)
	pub fn new_message(typ: Option<&str>, id: Option<&str>, to: Option<&str>) -> Self {
		let typ = FFI(typ).send();
		let to = FFI(to).send();
		let id = FFI(id).send();
		unsafe {
			Stanza::from_owned(sys::xmpp_message_new(
				ALLOC_CONTEXT.as_ptr(),
				typ.as_ptr(),
				to.as_ptr(),
				id.as_ptr(),
			))
		}
	}

	#[inline]
	#[cfg(feature = "libstrophe-0_9_3")]
	/// [xmpp_error_new](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga867086f16735eff5220a116d9d5e353b)
	pub fn new_error(typ: ErrorType, text: Option<&str>) -> Self {
		let text = FFI(text).send();
		unsafe { Stanza::from_owned(sys::xmpp_error_new(ALLOC_CONTEXT.as_ptr(), typ, text.as_ptr())) }
	}

	#[inline]
	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_stanza_new_from_string](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga334bbf723f451d1ca7ffb747029c0b4a)
	pub fn from_str(s: impl AsRef<str>) -> Self {
		#![allow(clippy::should_implement_trait)]
		let s = FFI(s.as_ref()).send();
		unsafe { Stanza::from_owned(sys::xmpp_stanza_new_from_string(ALLOC_CONTEXT.as_ptr(), s.as_ptr())) }
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_stanza_t, owned: bool) -> Self {
		let mut out = Stanza {
			inner: NonNull::new(inner).expect("Cannot allocate memory for Stanza"),
			owned,
		};
		if owned {
			out.set_alloc_context();
		}
		out
	}

	#[inline]
	/// Create an owning stanza from the raw pointer
	/// # Safety
	/// inner must be a valid pointer to a previously allocated xmpp_stanza_t and you must make sure
	/// that there are no other usages of that pointer after calling this function.
	pub unsafe fn from_owned(inner: *mut sys::xmpp_stanza_t) -> Self {
		Stanza::with_inner(inner, true)
	}

	#[inline]
	/// Create a borrowing stanza from the constant raw pointer
	/// # Safety
	/// inner must be a valid pointer to a previously allocated xmpp_stanza_t and you must make sure
	/// that Self doesn't outlive the stanza behind that pointer
	pub unsafe fn from_ref<'st>(inner: *const sys::xmpp_stanza_t) -> StanzaRef<'st> {
		Stanza::with_inner(inner as _, false).into()
	}

	#[inline]
	/// Create a borrowing stanza from the mutable raw pointer
	/// # Safety
	/// inner must be a valid pointer to a previously allocated mutable xmpp_stanza_t and you must
	/// make sure that Self doesn't outlive the stanza behind that pointer
	pub unsafe fn from_ref_mut<'st>(inner: *mut sys::xmpp_stanza_t) -> StanzaMutRef<'st> {
		Stanza::with_inner(inner, false).into()
	}

	/// Return internal raw pointer to stanza, for internal use
	pub(crate) fn as_ptr(&self) -> *mut sys::xmpp_stanza_t {
		self.inner.as_ptr()
	}

	/// Reset Stanza context to the 'static global ALLOC_CONTEXT to make it independent of whatever context it was created with
	///
	/// Generally libstrophe's `xmpp_stanza_t` needs `xmpp_ctx_t` only for allocation so it's possible to make `Stanza` 'static
	/// and not dependent on a particular context by using global `ALLOC_CONTEXT`. That's what is done when a new `Stanza` is
	/// created through methods of this struct, but when `Stanza` is copied (through `clone()` or `reply()`) we don't control
	/// the initial context and it is set by the libstrophe itself (e.g. in callback of `xmpp_handler_add`). In this case it
	/// receives the context that is tied to the one running the connection and it is not 'static. This function fixes that
	/// situation by overwriting the `ctx` reference for current stanza (including one in attributes hash table) and all of
	/// its children.
	fn set_alloc_context(&mut self) {
		#[allow(non_camel_case_types)]
		#[repr(C)]
		// this is dependent on internal representation in versions 0.9.3 to 0.12.2 (libstrophe-0_12_0), update if needed
		struct xmpp_stanza_t {
			rf: c_int,
			ctx: *mut sys::xmpp_ctx_t,
			typ: c_int,
			prev: *mut sys::xmpp_stanza_t,
			next: *mut sys::xmpp_stanza_t,
			children: *mut sys::xmpp_stanza_t,
			parent: *mut sys::xmpp_stanza_t,
			data: *mut c_char,
			attributes: *mut hash_t,
		}

		#[allow(non_camel_case_types)]
		#[repr(C)]
		struct hash_t {
			rf: c_uint,
			ctx: *mut sys::xmpp_ctx_t,
		}

		let inner = unsafe { (self.inner.as_ptr() as *mut xmpp_stanza_t).as_mut() }.expect("Null pointer for Stanza context");
		let alloc_ctx = ALLOC_CONTEXT.as_ptr();
		inner.ctx = alloc_ctx;
		if let Some(attrs) = unsafe { inner.attributes.as_mut() } {
			attrs.ctx = alloc_ctx;
		}
		for mut child in self.children_mut() {
			child.set_alloc_context();
		}
	}

	#[inline]
	/// [xmpp_stanza_is_text](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga3607a9a49c3614b8599b5ec469a65740)
	pub fn is_text(&self) -> bool {
		FFI(unsafe { sys::xmpp_stanza_is_text(self.inner.as_ptr()) }).receive_bool()
	}

	#[inline]
	/// [xmpp_stanza_is_tag](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gae39816bfdd86e97ca2b03c52813a5a12)
	pub fn is_tag(&self) -> bool {
		FFI(unsafe { sys::xmpp_stanza_is_tag(self.inner.as_ptr()) }).receive_bool()
	}

	#[inline]
	/// [xmpp_stanza_to_text](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga2918484877ac34d483cc14cf5e957fad)
	pub fn to_text(&self) -> Result<String, ToTextError> {
		stanza_to_text(self.inner.as_ptr(), |buf| Ok(buf.to_str()?.to_owned()))
	}

	#[inline]
	/// [xmpp_stanza_set_name](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga8331fbddc0f2fc7286a267ef60c69df2)
	///
	/// Be aware that calling this method changes the internal type of stanza to `XMPP_STANZA_TAG`.
	pub fn set_name(&mut self, name: impl AsRef<str>) -> Result<()> {
		let name = FFI(name.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_name(self.inner.as_mut(), name.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_name](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gabaffb70aab506a9f2912edb50c43c172)
	pub fn name(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_stanza_get_name(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_stanza_get_attribute_count](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga015147125e509724524c0aecb536c59c)
	pub fn attribute_count(&self) -> i32 {
		unsafe { sys::xmpp_stanza_get_attribute_count(self.inner.as_ptr()) }
	}

	#[inline]
	/// [xmpp_stanza_set_attribute](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga06ab477beba98d2f5b66d54e530bfa2d)
	pub fn set_attribute(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Result<()> {
		let name = FFI(name.as_ref()).send();
		let value = FFI(value.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_attribute(self.inner.as_mut(), name.as_ptr(), value.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_attribute](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gad836688d17b7c5af148f32823e72b81b)
	pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<&str> {
		let name = FFI(name.as_ref()).send();
		unsafe { FFI(sys::xmpp_stanza_get_attribute(self.inner.as_ptr(), name.as_ptr())).receive() }
	}

	/// [xmpp_stanza_get_attributes](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga2eea8820dcf9b3e2440a06de55a35850)
	///
	/// This method returns data as `HashMap` unlike underlying function.
	pub fn attributes(&self) -> HashMap<&str, &str> {
		let count = self.attribute_count();
		let mut out = HashMap::with_capacity(count as _);
		let mut arr = vec![ptr::null() as _; count as usize * 2];
		unsafe {
			sys::xmpp_stanza_get_attributes(self.inner.as_ptr(), arr.as_mut_ptr(), count * 2);
		}
		let mut iter = arr.into_iter();
		while let (Some(key), Some(val)) = (iter.next(), iter.next()) {
			out.insert(
				unsafe { FFI(key).receive() }.expect("Null pointer received for key in attributes() call"),
				unsafe { FFI(val).receive() }.expect("Null pointer received for value in attributes() call"),
			);
		}
		out
	}

	#[inline]
	/// [xmpp_stanza_del_attribute](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gae335c3ea4b5517d2e4cdfdc5cc41e143)
	pub fn del_attribute(&mut self, name: impl AsRef<str>) -> Result<()> {
		let name = FFI(name.as_ref()).send();
		unsafe { sys::xmpp_stanza_del_attribute(self.inner.as_mut(), name.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_set_text_with_size](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga779812852611947d0181cd4c58644ef1)
	///
	/// Be aware that calling this method changes the internal type of stanza to `XMPP_STANZA_TEXT`.
	pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
		let text = text.as_ref();
		unsafe { sys::xmpp_stanza_set_text_with_size(self.inner.as_mut(), text.as_ptr() as _, text.len()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_text](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga95476dfe39bb7acd8e6c534ef3877602)
	pub fn text(&self) -> Option<String> {
		unsafe { FFI(sys::xmpp_stanza_get_text(self.inner.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x)) }
	}

	#[inline]
	/// [xmpp_stanza_set_id](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gaa19a4d40d3383881b3266631dd9f2a0d)
	pub fn set_id(&mut self, id: impl AsRef<str>) -> Result<()> {
		let id = FFI(id.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_id(self.inner.as_mut(), id.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_id](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga7aebb4b618deb9fc24b9dae5418a0267)
	pub fn id(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_stanza_get_id(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_stanza_set_ns](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga2e55fd5671aa9803959ec19518a9adcf)
	pub fn set_ns(&mut self, ns: impl AsRef<str>) -> Result<()> {
		let ns = FFI(ns.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_ns(self.inner.as_mut(), ns.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_ns](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gabea315ad8a9a924489701ec5496ce4a7)
	pub fn ns(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_stanza_get_ns(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_stanza_set_type](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga30d9a7a46ec52c8c8675d31a6af1273b)
	pub fn set_stanza_type(&mut self, typ: impl AsRef<str>) -> Result<()> {
		let typ = FFI(typ.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_type(self.inner.as_mut(), typ.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_type](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga017c704661831141826ed8b4d3daba75)
	pub fn stanza_type(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_stanza_get_type(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_stanza_set_to](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga095ac729f5b65795cae689f7462a0a89)
	pub fn set_to(&mut self, to: impl AsRef<str>) -> Result<()> {
		let to = FFI(to.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_to(self.inner.as_mut(), to.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_to](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gabb04ada2be97b6d9440b28971dc5872d)
	pub fn to(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_stanza_get_to(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_stanza_set_from](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga368fd01efb2aa6ad0ec2c36233d2c551)
	pub fn set_from(&mut self, from: impl AsRef<str>) -> Result<()> {
		let from = FFI(from.as_ref()).send();
		unsafe { sys::xmpp_stanza_set_from(self.inner.as_mut(), from.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_get_from](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga03f3a2873592536e69c16905edbcd228)
	pub fn from(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_stanza_get_from(self.inner.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_stanza_get_children](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga35bbf13870c6551ec16a2d24b4d6b9e4)
	pub fn get_first_child(&self) -> Option<StanzaRef> {
		unsafe { sys::xmpp_stanza_get_children(self.inner.as_ptr()).as_ref() }.map(|x| unsafe { Self::from_ref(x) })
	}

	#[inline]
	/// [xmpp_stanza_get_children](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga35bbf13870c6551ec16a2d24b4d6b9e4)
	pub fn get_first_child_mut(&mut self) -> Option<StanzaMutRef> {
		unsafe { sys::xmpp_stanza_get_children(self.inner.as_mut()).as_mut() }.map(|x| unsafe { Self::from_ref_mut(x) })
	}

	#[inline]
	/// [xmpp_stanza_get_child_by_ns](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga09791fe5c7a5b3f4d90a95a46621eb1d)
	pub fn get_child_by_ns(&self, ns: impl AsRef<str>) -> Option<StanzaRef> {
		let ns = FFI(ns.as_ref()).send();
		unsafe { sys::xmpp_stanza_get_child_by_ns(self.inner.as_ptr(), ns.as_ptr()).as_ref() }.map(|x| unsafe { Self::from_ref(x) })
	}

	#[inline]
	/// [xmpp_stanza_get_child_by_ns](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga09791fe5c7a5b3f4d90a95a46621eb1d)
	pub fn get_child_by_ns_mut(&mut self, ns: impl AsRef<str>) -> Option<StanzaMutRef> {
		let ns = FFI(ns.as_ref()).send();
		unsafe { sys::xmpp_stanza_get_child_by_ns(self.inner.as_mut(), ns.as_ptr()).as_mut() }
			.map(|x| unsafe { Self::from_ref_mut(x) })
	}

	#[inline]
	/// [xmpp_stanza_get_child_by_name](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga19933d39585d91285e02c0c5fff41082)
	pub fn get_child_by_name(&self, name: impl AsRef<str>) -> Option<StanzaRef> {
		let name = FFI(name.as_ref()).send();
		unsafe { sys::xmpp_stanza_get_child_by_name(self.inner.as_ptr(), name.as_ptr()).as_ref() }
			.map(|x| unsafe { Self::from_ref(x) })
	}

	#[inline]
	/// [xmpp_stanza_get_child_by_name](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga19933d39585d91285e02c0c5fff41082)
	pub fn get_child_by_name_mut(&mut self, name: impl AsRef<str>) -> Option<StanzaMutRef> {
		let name = FFI(name.as_ref()).send();
		unsafe { sys::xmpp_stanza_get_child_by_name(self.inner.as_mut(), name.as_ptr()).as_mut() }
			.map(|x| unsafe { Self::from_ref_mut(x) })
	}

	#[inline]
	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_stanza_get_child_by_name_and_ns](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gaf00933e114ada170c526f11589d3e072)
	pub fn get_child_by_name_and_ns(&self, name: impl AsRef<str>, ns: impl AsRef<str>) -> Option<StanzaRef> {
		let name = FFI(name.as_ref()).send();
		let ns = FFI(ns.as_ref()).send();
		unsafe { sys::xmpp_stanza_get_child_by_name_and_ns(self.inner.as_ptr(), name.as_ptr(), ns.as_ptr()).as_ref() }
			.map(|x| unsafe { Self::from_ref(x) })
	}

	#[inline]
	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_stanza_get_child_by_name_and_ns](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gaf00933e114ada170c526f11589d3e072)
	pub fn get_child_by_name_and_ns_mut(&mut self, name: impl AsRef<str>, ns: impl AsRef<str>) -> Option<StanzaMutRef> {
		let name = FFI(name.as_ref()).send();
		let ns = FFI(ns.as_ref()).send();
		unsafe { sys::xmpp_stanza_get_child_by_name_and_ns(self.inner.as_mut(), name.as_ptr(), ns.as_ptr()).as_mut() }
			.map(|x| unsafe { Self::from_ref_mut(x) })
	}

	#[cfg(feature = "libstrophe-0_12_0")]
	/// [xmpp_stanza_get_child_by_path](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga12567a82abab6a54c396ea56cb895981)
	///
	/// Due to internal limitations (vararg call in C) this function supports a maximum of 10 elements
	/// in the `path` slice.
	pub fn get_child_by_path(&self, path: &[&str]) -> Option<StanzaRef> {
		let res = internals::stanza_get_child_by_path(self.inner.as_ptr(), path);
		unsafe { res.as_ref() }.map(|x| unsafe { Self::from_ref(x) })
	}

	#[inline]
	#[cfg(feature = "libstrophe-0_12_0")]
	/// [xmpp_stanza_get_child_by_path](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga12567a82abab6a54c396ea56cb895981)
	///
	/// Due to internal limitations (vararg call in C) this function supports a maximum of 10 elements
	/// in the `path` slice.
	pub fn get_child_by_path_mut(&mut self, path: &[&str]) -> Option<StanzaMutRef> {
		let res = internals::stanza_get_child_by_path(unsafe { self.inner.as_mut() }, path);
		unsafe { res.as_mut() }.map(|x| unsafe { Self::from_ref_mut(x) })
	}

	#[inline]
	pub fn children(&self) -> impl Iterator<Item = StanzaRef> {
		ChildIterator {
			cur: self.get_first_child().map(StanzaChildRef),
		}
	}

	#[inline]
	pub fn children_mut(&mut self) -> impl Iterator<Item = StanzaMutRef> {
		ChildIteratorMut {
			cur: self.get_first_child_mut().map(StanzaChildMutRef),
		}
	}

	#[inline]
	/// [xmpp_stanza_get_next](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga4eceb55b6a939767d473f7faacfcc6e2)
	pub fn get_next(&self) -> Option<StanzaRef> {
		unsafe { sys::xmpp_stanza_get_next(self.inner.as_ptr()).as_ref() }.map(|x| unsafe { Self::from_ref(x) })
	}

	#[inline]
	/// [xmpp_stanza_get_next](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga4eceb55b6a939767d473f7faacfcc6e2)
	pub fn get_next_mut(&mut self) -> Option<StanzaMutRef> {
		unsafe { sys::xmpp_stanza_get_next(self.inner.as_mut()).as_mut() }.map(|x| unsafe { Self::from_ref_mut(x) })
	}

	#[inline]
	/// [xmpp_stanza_add_child](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga9cfdeabcfc45409d2dfff4b364f84a0c)
	pub fn add_child(&mut self, child: Stanza) -> Result<()> {
		let mut child = child;
		unsafe { sys::xmpp_stanza_add_child(self.inner.as_mut(), child.inner.as_mut()) }.into_result()
	}

	#[inline]
	/// [xmpp_stanza_reply](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga32c20758b86bf9c46688e58878a284b5)
	pub fn reply(&self) -> Self {
		unsafe { Self::from_owned(sys::xmpp_stanza_reply(self.inner.as_ptr())) }
	}

	#[inline]
	#[cfg(feature = "libstrophe-0_10_0")]
	/// [xmpp_stanza_reply_error](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga62a222d584f3890b957ab507070664ff)
	pub fn reply_error(&self, error_type: impl AsRef<str>, condition: impl AsRef<str>, text: impl AsRef<str>) -> Self {
		let error_type = FFI(error_type.as_ref()).send();
		let condition = FFI(condition.as_ref()).send();
		let text = FFI(text.as_ref()).send();
		unsafe {
			Self::from_owned(sys::xmpp_stanza_reply_error(
				self.inner.as_ptr(),
				error_type.as_ptr(),
				condition.as_ptr(),
				text.as_ptr(),
			))
		}
	}

	#[inline]
	/// [xmpp_message_set_body](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gace4a07d21a6700692d22ea13200d13f5)
	pub fn set_body(&mut self, body: impl AsRef<str>) -> Result<()> {
		let body = FFI(body.as_ref()).send();
		unsafe { sys::xmpp_message_set_body(self.inner.as_mut(), body.as_ptr()) }.into_result()
	}

	#[inline]
	/// [xmpp_message_get_body](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga6659d4692fdddc6758f99c1f867fa3e2)
	pub fn body(&self) -> Option<String> {
		unsafe { FFI(sys::xmpp_message_get_body(self.inner.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x)) }
	}
}

#[inline]
#[allow(non_snake_case)]
/// Helper function for [Stanza::get_child_by_path]
/// [XMPP_STANZA_NAME_IN_NS](https://strophe.im/libstrophe/doc/0.12.2/strophe_8h.html#a3c83ba7062a2099e61d44c9226bdb286)
pub fn XMPP_STANZA_NAME_IN_NS(name: &str, ns: &str) -> String {
	format!("{}[@ns='{}']", name, ns)
}

#[cfg(feature = "libstrophe-0_10_0")]
impl std::str::FromStr for Stanza {
	type Err = ();

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from_str(s))
	}
}

impl Display for Stanza {
	/// [xmpp_stanza_to_text](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#ga2918484877ac34d483cc14cf5e957fad)
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		stanza_to_text(self.inner.as_ptr(), |buf| f.write_str(buf.to_str().map_err(|_| fmt::Error)?))
	}
}

impl Clone for Stanza {
	#[inline]
	fn clone(&self) -> Self {
		unsafe { Stanza::from_owned(sys::xmpp_stanza_copy(self.inner.as_ptr())) }
	}
}

impl PartialEq for Stanza {
	#[inline]
	fn eq(&self, other: &Stanza) -> bool {
		self.inner == other.inner
	}
}

impl Hash for Stanza {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
	}
}

impl Drop for Stanza {
	#[inline]
	/// [xmpp_stanza_release](https://strophe.im/libstrophe/doc/0.12.2/group___stanza.html#gaa231317e56af0b974d7a28793ebefb83)
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				sys::xmpp_stanza_release(self.inner.as_mut());
			}
		}
	}
}

impl Default for Stanza {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

unsafe impl Send for Stanza {}

impl From<Stanza> for StanzaRef<'_> {
	#[inline]
	fn from(s: Stanza) -> Self {
		StanzaRef(s, PhantomData)
	}
}

impl From<Stanza> for StanzaMutRef<'_> {
	#[inline]
	fn from(s: Stanza) -> Self {
		StanzaMutRef(s, PhantomData)
	}
}

/// Wrapper for constant reference to [`Stanza`], implements `Deref` to [`Stanza`]
///
/// You can obtain such objects by calling [`Stanza`] child search methods.
///
/// [`Stanza`]: struct.Stanza.html
#[derive(Debug, Eq)]
pub struct StanzaRef<'st>(Stanza, PhantomData<&'st Stanza>);

impl ops::Deref for StanzaRef<'_> {
	type Target = Stanza;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl PartialEq for StanzaRef<'_> {
	#[inline]
	fn eq(&self, other: &StanzaRef) -> bool {
		self.inner == other.inner
	}
}

impl Display for StanzaRef<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[derive(Debug)]
struct StanzaChildRef<'parent>(StanzaRef<'parent>);

impl<'parent> StanzaChildRef<'parent> {
	#[inline]
	pub fn get_next(&self) -> Option<StanzaChildRef<'parent>> {
		unsafe { sys::xmpp_stanza_get_next(self.0.inner.as_ptr()).as_ref() }.map(|x| StanzaChildRef(unsafe { Stanza::from_ref(x) }))
	}
}

/// Wrapper for mutable reference to [`Stanza`], implements `Deref` and `DerefMut` to [`Stanza`]
///
/// You can obtain such objects by calling [`Stanza`] child search methods.
///
/// [`Stanza`]: struct.Stanza.html
#[derive(Debug)]
pub struct StanzaMutRef<'st>(Stanza, PhantomData<&'st mut Stanza>);

impl ops::Deref for StanzaMutRef<'_> {
	type Target = Stanza;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ops::DerefMut for StanzaMutRef<'_> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Display for StanzaMutRef<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[derive(Debug)]
pub struct StanzaChildMutRef<'parent>(StanzaMutRef<'parent>);

impl<'parent> StanzaChildMutRef<'parent> {
	#[inline]
	pub fn get_next_mut(&mut self) -> Option<StanzaChildMutRef<'parent>> {
		unsafe { sys::xmpp_stanza_get_next(self.0.inner.as_ptr()).as_mut() }
			.map(|x| StanzaChildMutRef(unsafe { Stanza::from_ref_mut(x) }))
	}
}

struct ChildIterator<'st> {
	cur: Option<StanzaChildRef<'st>>,
}

impl<'st> Iterator for ChildIterator<'st> {
	type Item = StanzaRef<'st>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		self.cur.take().map(|cur| {
			self.cur = cur.get_next();
			cur.0
		})
	}
}

struct ChildIteratorMut<'st> {
	cur: Option<StanzaChildMutRef<'st>>,
}

impl<'st> Iterator for ChildIteratorMut<'st> {
	type Item = StanzaMutRef<'st>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		self.cur.take().map(|mut cur| {
			self.cur = cur.get_next_mut();
			cur.0
		})
	}
}

fn stanza_to_text<T, E>(stanza: *mut sys::xmpp_stanza_t, cb: impl FnOnce(&CStr) -> Result<T, E>) -> Result<T, E>
where
	E: From<Error>,
{
	let mut buf: *mut c_char = ptr::null_mut();
	let mut buflen: usize = 0;
	let res = unsafe { sys::xmpp_stanza_to_text(stanza, &mut buf, &mut buflen) };
	let _free_buf = scopeguard::guard(buf, |buf| {
		if !buf.is_null() {
			unsafe {
				ALLOC_CONTEXT.free(buf);
			}
		}
	});
	res.into_result().map_err(E::from).and_then(|_| {
		let text = unsafe { CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(buf as *const _, buflen + 1)) };
		cb(text)
	})
}
