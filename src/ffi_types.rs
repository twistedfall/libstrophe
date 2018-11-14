extern crate num;

use std::{ffi, ptr};
use std::os::raw;

pub struct FFI<T>(pub T);

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
		self.0.as_ref().map(|x| {
			ffi::CStr::from_ptr(x).to_str().expect("Cannot convert non-null pointer into &str")
		})
	}
}

impl<'s> FFI<*mut raw::c_char> {
	#[inline]
	pub unsafe fn receive_with_free<CB>(self, free: CB) -> Option<String>
		where
			CB: FnOnce(*mut raw::c_char)
	{
		self.0.as_mut().map(|x| {
			let out = ffi::CStr::from_ptr(x).to_owned().into_string().expect("Cannot convert non-null pointer into String");
			free(x);
			out
		})
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
