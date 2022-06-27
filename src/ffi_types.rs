use std::ffi::{c_char, CStr, CString};
use std::ptr;

#[allow(clippy::upper_case_acronyms)]
pub struct FFI<T>(pub T);

impl FFI<&str> {
	#[inline]
	pub fn send(self) -> CString {
		CString::new(self.0).expect("Cannot convert to CString")
	}
}

impl<T: num_traits::Zero + PartialEq> FFI<T> {
	#[inline]
	pub fn receive_bool(self) -> bool {
		self.0 != T::zero()
	}
}

impl FFI<*const c_char> {
	/// The lifetime of the returned reference is bound to "lifetime" of the pointer
	#[inline]
	pub unsafe fn receive<'s>(self) -> Option<&'s str> {
		self
			.0
			.as_ref()
			.map(|x| CStr::from_ptr(x).to_str().expect("Cannot convert non-null pointer into &str"))
	}
}

impl FFI<*mut c_char> {
	#[inline]
	pub unsafe fn receive_with_free(self, free: impl FnOnce(*mut c_char)) -> Option<String> {
		self.0.as_mut().map(|x| {
			let out = CStr::from_ptr(x)
				.to_owned()
				.into_string()
				.expect("Cannot convert non-null pointer into String");
			free(x);
			out
		})
	}
}

impl FFI<Option<&str>> {
	#[inline]
	pub fn send(self) -> Nullable<CString> {
		match self.0 {
			None => Nullable::Null,
			Some(v) => Nullable::Val(FFI(v).send()),
		}
	}
}

pub enum Nullable<T> {
	Null,
	Val(T),
}

impl Nullable<CString> {
	#[inline]
	pub fn as_ptr(&self) -> *const c_char {
		match *self {
			Nullable::Null => ptr::null(),
			Nullable::Val(ref v) => v.as_ptr(),
		}
	}
}

impl<T: num_traits::Zero> Nullable<T> {
	#[inline]
	pub fn val(self) -> T {
		match self {
			Nullable::Null => T::zero(),
			Nullable::Val(v) => v,
		}
	}
}

impl<T> From<Option<T>> for Nullable<T> {
	#[inline]
	fn from(f: Option<T>) -> Self {
		match f {
			None => Nullable::Null,
			Some(v) => Nullable::Val(v),
		}
	}
}

//impl<T> Into<Option<T>> for Nullable<T> {
//	#[inline]
//	fn into(self) -> Option<T> {
//		match self {
//			Nullable::Null => Option::None,
//			Nullable::Val(v) => Option::Some(v)
//		}
//	}
//}
