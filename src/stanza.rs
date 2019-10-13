use std::{
	collections,
	ffi,
	fmt::{
		Display,
		Error as FmtError,
		Formatter,
		Result as FmtResult,
	},
	hash::{
		Hash,
		Hasher,
	},
	marker::PhantomData,
	mem::MaybeUninit,
	ops,
	os::raw,
	ptr,
	ptr::NonNull,
	slice,
};

use crate::{
	ALLOC_CONTEXT,
	error,
	FFI,
};

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
/// [docs]: http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html
/// [sources]: https://github.com/strophe/libstrophe/blob/0.9.2/src/stanza.c
/// [xmpp_stanza_to_text]: http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga49d188283a22e228ebf188aa06cf55b6
/// [xmpp_stanza_release]: http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga71a1b5d6974e435aa1dca60a547fd11a
/// [xmpp_stanza_copy]: http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaef536615ea184b55e461980a1a8dba02
#[derive(Debug)]
pub struct Stanza {
	inner: NonNull<sys::xmpp_stanza_t>,
	owned: bool,
}

impl Stanza {
	/// [xmpp_stanza_new](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga8e9261485e96c080de3103d8a42001df)
	///
	/// The newly created stanza is not really useful until you assign an internal type to it. To do
	/// that you must call [`set_text()`] to make it `XMPP_STANZA_TEXT` stanza or [`set_name()`] to make
	/// it `XMPP_STANZA_TAG` stanza.
	///
	/// [`set_text()`]: struct.Stanza.html#method.set_text
	/// [`set_name()`]: struct.Stanza.html#method.set_name
	pub fn new() -> Self {
		unsafe { Stanza::from_inner(sys::xmpp_stanza_new(ALLOC_CONTEXT.as_inner())) }
	}

	/// [xmpp_presence_new](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga5f3a4cde910ad181b8e569ff0431d7ac)
	pub fn new_presence() -> Self {
		unsafe { Stanza::from_inner(sys::xmpp_presence_new(ALLOC_CONTEXT.as_inner())) }
	}

	/// [xmpp_iq_new](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaf23007ddde78ec028a78ceec056544fd)
	pub fn new_iq(typ: Option<&str>, id: Option<&str>) -> Self
	{
		let typ = FFI(typ).send();
		let id = FFI(id).send();
		unsafe {
			Stanza::from_inner(
				sys::xmpp_iq_new(
					ALLOC_CONTEXT.as_inner(),
					typ.as_ptr(),
					id.as_ptr()
				)
			)
		}
	}

	/// [xmpp_message_new](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga3042d09bbe4aba9018ae617ca07f31a8)
	pub fn new_message(typ: Option<&str>, id: Option<&str>, to: Option<&str>) -> Self
	{
		let typ = FFI(typ).send();
		let to = FFI(to).send();
		let id = FFI(id).send();
		unsafe {
			Stanza::from_inner(
				sys::xmpp_message_new(
					ALLOC_CONTEXT.as_inner(),
					typ.as_ptr(),
					to.as_ptr(),
					id.as_ptr(),
				)
			)
		}
	}

	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_stanza_t, owned: bool) -> Self {
		Stanza { inner: NonNull::new(inner).expect("Cannot allocate memory for Stanza"), owned }
	}

	/// Create an owning stanza from the raw pointer, for internal use
	pub unsafe fn from_inner(inner: *mut sys::xmpp_stanza_t) -> Self {
		Stanza::with_inner(inner, true)
	}

	/// Create a borrowing stanza from the constant raw pointer, for internal use
	pub unsafe fn from_inner_ref<'st>(inner: *const sys::xmpp_stanza_t) -> StanzaRef<'st> {
		Stanza::with_inner(inner as _, false).into()
	}

	/// Create a borrowing stanza from the mutable raw pointer, for internal use
	pub unsafe fn from_inner_ref_mut<'st>(inner: *mut sys::xmpp_stanza_t) -> StanzaMutRef<'st> {
		Stanza::with_inner(inner, false).into()
	}

	/// Return internal raw pointer to stanza, for internal use
	pub fn as_inner(&self) -> *mut sys::xmpp_stanza_t { self.inner.as_ptr() }

	/// [xmpp_stanza_is_text](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gafe82902f19a387da45ce08a23b1cded6)
	pub fn is_text(&self) -> bool {
		FFI(unsafe { sys::xmpp_stanza_is_text(self.inner.as_ptr()) }).receive_bool()
	}

	/// [xmpp_stanza_is_tag](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga90cda8b1581546b0b93f6aefdeb288ad)
	pub fn is_tag(&self) -> bool {
		FFI(unsafe { sys::xmpp_stanza_is_tag(self.inner.as_ptr()) }).receive_bool()
	}

	/// [xmpp_stanza_to_text](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga49d188283a22e228ebf188aa06cf55b6)
	pub fn to_text(&self) -> error::Result<String> {
		let mut buf: MaybeUninit<*mut raw::c_char> = MaybeUninit::uninit();
		let mut buflen: MaybeUninit<usize> = MaybeUninit::uninit();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_to_text(self.inner.as_ptr(), buf.as_mut_ptr(), buflen.as_mut_ptr())
		}).and_then(|_| {
			let buf = unsafe { ffi::CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(buf.assume_init() as _, buflen.assume_init() + 1)) };
			let out = buf.to_str()?.to_owned();
			unsafe {
				ALLOC_CONTEXT.free(buf.as_ptr() as *mut raw::c_char);
			}
			Ok(out)
		})
	}

	/// [xmpp_stanza_set_name](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gabf2e9e4c2e5b638296a97d687ec36b70)
	///
	/// Be aware that calling this method changes the internal type of stanza to `XMPP_STANZA_TAG`.
	pub fn set_name(&mut self, name: impl AsRef<str>) -> error::EmptyResult {
		let name = FFI(name.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_name(self.inner.as_mut(), name.as_ptr())
		})
	}

	/// [xmpp_stanza_get_name](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gad94f7ab260305ef72be043e3ad327102)
	pub fn name(&self) -> Option<&str> {
		unsafe {
			FFI(sys::xmpp_stanza_get_name(self.inner.as_ptr())).receive()
		}
	}

	/// [xmpp_stanza_get_attribute_count](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga4f129d65e5222394903522d77e554649)
	pub fn attribute_count(&self) -> i32 {
		unsafe {
			sys::xmpp_stanza_get_attribute_count(self.inner.as_ptr())
		}
	}

	/// [xmpp_stanza_set_attribute](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaa444244670d08acaef1d518912614d83)
	pub fn set_attribute(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> error::EmptyResult {
		let name = FFI(name.as_ref()).send();
		let value = FFI(value.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_attribute(self.inner.as_mut(), name.as_ptr(), value.as_ptr())
		})
	}

	/// [xmpp_stanza_get_attribute](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga955cdbcdd570c9fd164b6703040c5781)
	pub fn get_attribute(&self, name: impl AsRef<str>) -> Option<&str> {
		let name = FFI(name.as_ref()).send();
		unsafe {
			FFI(sys::xmpp_stanza_get_attribute(self.inner.as_ptr(), name.as_ptr())).receive()
		}
	}

	/// [xmpp_stanza_get_attributes](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gafe698eb2346933a8a9109753a0bbab5a)
	///
	/// This method returns data as `HashMap` unlike underlying function.
	pub fn attributes(&self) -> collections::HashMap<&str, &str> {
		let count = self.attribute_count();
		let mut out = collections::HashMap::with_capacity(count as _);
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

	/// [xmpp_stanza_del_attribute](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga80f657159081216671ca8f1f33dad5fe)
	pub fn del_attribute(&mut self, name: impl AsRef<str>) -> error::EmptyResult {
		let name = FFI(name.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_del_attribute(self.inner.as_mut(), name.as_ptr())
		})
	}

	/// [xmpp_stanza_set_text_with_size](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gabd17e35d29c582eadc9bf7b14af94fd0)
	///
	/// Be aware that calling this method changes the internal type of stanza to `XMPP_STANZA_TEXT`.
	pub fn set_text(&mut self, text: impl AsRef<str>) -> error::EmptyResult {
		let text = text.as_ref();
		error::code_to_result(unsafe { sys::xmpp_stanza_set_text_with_size(self.inner.as_mut(), text.as_ptr() as _, text.len()) })
	}

	/// [xmpp_stanza_get_text](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaceb6c04c44e2387f5918a2edf5853a8c)
	pub fn text(&self) -> Option<String> {
		unsafe {
			FFI(sys::xmpp_stanza_get_text(self.inner.as_ptr())).receive_with_free(|x| {
				ALLOC_CONTEXT.free(x)
			})
		}
	}


	/// [xmpp_stanza_set_id](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga95e5daab560a571cd3f0faf8e15f0ad1)
	pub fn set_id(&mut self, id: impl AsRef<str>) -> error::EmptyResult {
		let id = FFI(id.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_id(self.inner.as_mut(), id.as_ptr())
		})
	}

	/// [xmpp_stanza_get_id](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gae5ca6ad0e6ca61d3291dcf7b268f69fd)
	pub fn id(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_id(self.inner.as_ptr())).receive() } }

	/// [xmpp_stanza_set_ns](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gadc05616c1b7f95fc7fea910c6960def7)
	pub fn set_ns(&mut self, ns: impl AsRef<str>) -> error::EmptyResult {
		let ns = FFI(ns.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_ns(self.inner.as_mut(), ns.as_ptr())
		})
	}

	/// [xmpp_stanza_get_ns](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaa2515e705415cf5735416969b671f9d5)
	pub fn ns(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_ns(self.inner.as_ptr())).receive() } }

	/// [xmpp_stanza_set_type](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga2251d36609c20f2c367d3595dac307da)
	pub fn set_stanza_type(&mut self, typ: impl AsRef<str>) -> error::EmptyResult {
		let typ = FFI(typ.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_type(self.inner.as_mut(), typ.as_ptr())
		})
	}

	/// [xmpp_stanza_get_type](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga6a1a17033a76fb63d2b173a279bf57fa)
	pub fn stanza_type(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_type(self.inner.as_ptr())).receive() } }

	/// [xmpp_stanza_set_to](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga37ac2d958f43842038fc70d03fc7114a)
	pub fn set_to(&mut self, to: impl AsRef<str>) -> error::EmptyResult {
		let to = FFI(to.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_to(self.inner.as_mut(), to.as_ptr())
		})
	}

	/// [xmpp_stanza_get_to](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gacd7b09bb46f5f3950e81edb946a49a50)
	pub fn to(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_to(self.inner.as_ptr())).receive() } }

	/// [xmpp_stanza_set_from](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga1f80e0f685c3adf71741a702d36b3bae)
	pub fn set_from(&mut self, from: impl AsRef<str>) -> error::EmptyResult {
		let from = FFI(from.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_set_from(self.inner.as_mut(), from.as_ptr())
		})
	}

	/// [xmpp_stanza_get_from](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga88854ce441147e68835e7215ee53139a)
	pub fn from(&self) -> Option<&str> { unsafe { FFI(sys::xmpp_stanza_get_from(self.inner.as_ptr())).receive() } }

	/// [xmpp_stanza_get_children](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga95e73cb5be9ba97560416e251da5d8e1)
	pub fn get_first_child(&self) -> Option<StanzaRef> {
		unsafe {
			sys::xmpp_stanza_get_children(self.inner.as_ptr()).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_children](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga95e73cb5be9ba97560416e251da5d8e1)
	pub fn get_first_child_mut(&mut self) -> Option<StanzaMutRef> {
		unsafe {
			sys::xmpp_stanza_get_children(self.inner.as_mut()).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	/// [xmpp_stanza_get_child_by_ns](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga762593f54a559f32f0445792f7a7614d)
	pub fn get_child_by_ns(&self, ns: impl AsRef<str>) -> Option<StanzaRef> {
		let ns = FFI(ns.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_ns(self.inner.as_ptr(), ns.as_ptr()).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_child_by_ns](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga762593f54a559f32f0445792f7a7614d)
	pub fn get_child_by_ns_mut(&mut self, ns: impl AsRef<str>) -> Option<StanzaMutRef> {
		let ns = FFI(ns.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_ns(self.inner.as_mut(), ns.as_ptr()).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	/// [xmpp_stanza_get_child_by_name](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga49522c46a0a0c2c6fe9479073e307b05)
	pub fn get_child_by_name(&self, name: impl AsRef<str>) -> Option<StanzaRef> {
		let name = FFI(name.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_name(self.inner.as_ptr(), name.as_ptr()).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_child_by_name](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga49522c46a0a0c2c6fe9479073e307b05)
	pub fn get_child_by_name_mut(&mut self, name: impl AsRef<str>) -> Option<StanzaMutRef> {
		let name = FFI(name.as_ref()).send();
		unsafe {
			sys::xmpp_stanza_get_child_by_name(self.inner.as_mut(), name.as_ptr()).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	pub fn children(&self) -> impl Iterator<Item=StanzaRef> {
		ChildIterator {
			cur: self.get_first_child().map(|x| x.as_inner()),
			_ref: PhantomData,
		}
	}

	pub fn children_mut(&mut self) -> impl Iterator<Item=StanzaMutRef> {
		ChildIteratorMut {
			cur: self.get_first_child_mut().map(|x| x.as_inner()),
			_ref: PhantomData,
		}
	}

	/// [xmpp_stanza_get_next](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaa9a115b89f245605279120c05d698853)
	pub fn get_next(&self) -> Option<StanzaRef> {
		unsafe {
			sys::xmpp_stanza_get_next(self.inner.as_ptr()).as_ref()
		}.map(|x| unsafe { Self::from_inner_ref(x) })
	}

	/// [xmpp_stanza_get_next](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gaa9a115b89f245605279120c05d698853)
	pub fn get_next_mut(&mut self) -> Option<StanzaMutRef> {
		unsafe {
			sys::xmpp_stanza_get_next(self.inner.as_mut()).as_mut()
		}.map(|x| unsafe { Self::from_inner_ref_mut(x) })
	}

	/// [xmpp_stanza_add_child](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga9cfdeabcfc45409d2dfff4b364f84a0c)
	pub fn add_child(&mut self, child: Stanza) -> error::EmptyResult {
		let mut child = child;
		error::code_to_result(unsafe {
			sys::xmpp_stanza_add_child(self.inner.as_mut(), child.inner.as_mut())
		})
	}

	/// [xmpp_stanza_reply](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#gab7d75fc56a0558edd8bf45368d50fb15)
	pub fn reply(&self) -> Self {
		unsafe {
			Self::from_inner(sys::xmpp_stanza_reply(self.inner.as_ptr()))
		}
	}

	/// [xmpp_message_set_body](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga2fdc6b475a1906a4b22f6e7568b36f98)
	pub fn set_body(&mut self, body: impl AsRef<str>) -> error::EmptyResult {
		let body = FFI(body.as_ref()).send();
		error::code_to_result(unsafe {
			sys::xmpp_message_set_body(self.inner.as_mut(), body.as_ptr())
		})
	}

	/// [xmpp_message_get_body](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga3ad5f7e64be52d04ed6f6680d80303fb)
	pub fn body(&self) -> Option<String> {
		unsafe {
			FFI(sys::xmpp_message_get_body(self.inner.as_ptr())).receive_with_free(|x| {
				ALLOC_CONTEXT.free(x)
			})
		}
	}
}

impl Display for Stanza {
	/// [xmpp_stanza_to_text](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga49d188283a22e228ebf188aa06cf55b6)
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		let mut buf: MaybeUninit<*mut raw::c_char> = MaybeUninit::uninit();
		let mut buflen: MaybeUninit<usize> = MaybeUninit::uninit();
		error::code_to_result(unsafe {
			sys::xmpp_stanza_to_text(self.inner.as_ptr(), buf.as_mut_ptr(), buflen.as_mut_ptr())
		}).map_err(|_| {
			FmtError
		}).and_then(|_| {
			let buf = unsafe { ffi::CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(buf.assume_init() as _, buflen.assume_init() + 1)) };
			let out = write!(f, "{}", buf.to_str().map_err(|_| FmtError)?);
			unsafe {
				ALLOC_CONTEXT.free(buf.as_ptr() as *mut raw::c_char);
			}
			out
		})
	}
}

impl Clone for Stanza {
	fn clone(&self) -> Self {
		unsafe { Stanza::from_inner(sys::xmpp_stanza_copy(self.inner.as_ptr())) }
	}
}

impl PartialEq for Stanza {
	fn eq(&self, other: &Stanza) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Stanza {}

impl Hash for Stanza {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
	}
}

impl Drop for Stanza {
	/// [xmpp_stanza_release](http://strophe.im/libstrophe/doc/0.9.2/group___stanza.html#ga71a1b5d6974e435aa1dca60a547fd11a)
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				sys::xmpp_stanza_release(self.inner.as_mut());
			}
		}
	}
}

impl Default for Stanza {
	fn default() -> Self {
		Self::new()
	}
}

unsafe impl Send for Stanza {}

impl<'st> Into<StanzaRef<'st>> for Stanza {
	fn into(self) -> StanzaRef<'st> {
		StanzaRef(self, PhantomData)
	}
}

impl<'st> Into<StanzaMutRef<'st>> for Stanza {
	fn into(self) -> StanzaMutRef<'st> {
		StanzaMutRef(self, PhantomData)
	}
}

/// Wrapper for constant reference to [`Stanza`], implements `Deref` to [`Stanza`]
///
/// You can obtain such objects by calling [`Stanza`] child search methods.
///
/// [`Stanza`]: struct.Stanza.html
#[derive(Debug)]
pub struct StanzaRef<'st>(Stanza, PhantomData<&'st Stanza>);

impl ops::Deref for StanzaRef<'_> {
	type Target = Stanza;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Display for StanzaRef<'_> {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		self.0.fmt(f)
	}
}

/// Wrapper for mutable reference to [`Stanza`], implements `Deref` and `DerefMut` to [`Stanza`]
///
/// You can obtain such objects by calling [`Stanza`] child search methods.
///
/// [`Stanza`]: struct.Stanza.html
#[derive(Debug)]
pub struct StanzaMutRef<'st>(Stanza, PhantomData<&'st Stanza>);

impl ops::Deref for StanzaMutRef<'_> {
	type Target = Stanza;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ops::DerefMut for StanzaMutRef<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Display for StanzaMutRef<'_> {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		self.0.fmt(f)
	}
}

struct ChildIterator<'st> {
	cur: Option<*mut sys::xmpp_stanza_t>,
	_ref: PhantomData<&'st Stanza>,
}

impl<'st> Iterator for ChildIterator<'st> {
	type Item = StanzaRef<'st>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		if let Some(cur) = self.cur {
			let cur = unsafe { Stanza::from_inner_ref(cur) };
			self.cur = cur.get_next().map(|x| x.as_inner());
			Some(cur)
		} else {
			None
		}
	}
}

struct ChildIteratorMut<'st> {
	cur: Option<*mut sys::xmpp_stanza_t>,
	_ref: PhantomData<&'st Stanza>,
}

impl<'st> Iterator for ChildIteratorMut<'st> {
	type Item = StanzaMutRef<'st>;

	fn next(&mut self) -> Option<<Self as Iterator>::Item> {
		if let Some(cur) = self.cur {
			let cur = unsafe { Stanza::from_inner_ref_mut(cur) };
			self.cur = cur.get_next().map(|x| x.as_inner());
			Some(cur)
		} else {
			None
		}
	}
}
