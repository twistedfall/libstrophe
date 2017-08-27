extern crate num;

use std::{ffi, ptr};
use std::os::raw;

pub struct FFI<T>(pub T);

impl<T> FFI<T> {}

impl<'s> FFI<&'s str> {
	#[inline]
	pub fn send(self) -> ffi::CString {
		ffi::CString::new(self.0).expect("Cannot convert to CString")
	}
}

impl<T: num::Zero + PartialEq> FFI<T> {
	#[inline]
	pub fn receive_bool(self) -> bool {
		self.0 != T::zero()
	}
}

impl<'s> FFI<*const raw::c_char> {
	/// The lifetime of the returned reference is bound to "lifetime" of the pointer
	#[inline]
	pub unsafe fn receive(self) -> Option<&'s str> {
		if self.0.is_null() {
			None
		} else {
			Some(ffi::CStr::from_ptr(self.0).to_str().expect("Cannot convert non-null pointer into CStr"))
		}
	}
}

impl<'s> FFI<*mut raw::c_char> {
	/// The lifetime of the returned reference is bound to "lifetime" of the pointer
	#[inline]
	pub unsafe fn receive(self) -> Option<&'s str> {
		if self.0.is_null() {
			None
		} else {
			Some(ffi::CStr::from_ptr(self.0).to_str().expect("Cannot convert non-null pointer into CStr"))
		}
	}
}

impl<'s> FFI<Option<&'s str>> {
	#[inline]
	pub fn send(self) -> Nullable<ffi::CString> {
		match self.0 {
			None => Nullable::Null,
			Some(v) => Nullable::Val(FFI(v).send())
		}
	}
}


pub enum Nullable<T> {
	Null,
	Val(T),
}

impl<'s> Nullable<ffi::CString> {
	#[inline]
	pub fn as_ptr(&self) -> *const raw::c_char {
		match *self {
			Nullable::Null => ptr::null(),
			Nullable::Val(ref v) => v.as_ptr()
		}
	}
}

impl<T: num::Zero> Nullable<T> {
	#[inline]
	pub fn val(self) -> T {
		match self {
			Nullable::Null => T::zero(),
			Nullable::Val(v) => v
		}
	}
}

impl<T> From<Option<T>> for Nullable<T> {
	#[inline]
	fn from(f: Option<T>) -> Self {
		match f {
			None => Nullable::Null,
			Some(v) => Nullable::Val(v)
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
