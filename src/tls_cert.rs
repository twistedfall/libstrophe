use core::fmt;
use core::ptr::NonNull;

use crate::{CertElement, Context, FFI};

pub struct TlsCert {
	inner: NonNull<sys::xmpp_tlscert_t>,
	owned: bool,
}

impl TlsCert {
	#[inline]
	unsafe fn with_inner(inner: *mut sys::xmpp_tlscert_t, owned: bool) -> Self {
		Self {
			inner: NonNull::new(inner).expect("Cannot allocate memory for TLS certificate"),
			owned,
		}
	}

	#[inline]
	pub(crate) unsafe fn from_owned(inner: *mut sys::xmpp_tlscert_t) -> Self {
		unsafe { Self::with_inner(inner, true) }
	}

	#[inline]
	pub(crate) unsafe fn from_ref(inner: *const sys::xmpp_tlscert_t) -> Self {
		unsafe { Self::with_inner(inner.cast_mut(), false) }
	}

	pub(crate) fn as_ptr(&self) -> *mut sys::xmpp_tlscert_t {
		self.inner.as_ptr()
	}

	#[inline]
	/// [xmpp_tlscert_get_ctx](https://strophe.im/libstrophe/doc/0.13.0/group___t_l_s.html#gaae2a196318df4cc7155d1051e99ecf0c)
	pub fn context(&self) -> Context {
		unsafe { Context::from_ref_mut(sys::xmpp_tlscert_get_ctx(self.as_ptr())) }
	}

	#[inline]
	/// [xmpp_tlscert_get_pem](https://strophe.im/libstrophe/doc/0.13.0/group___t_l_s.html#ga5eef98297b6b1779c2101f12551a6595)
	pub fn pem(&self) -> Option<&str> {
		unsafe { FFI(sys::xmpp_tlscert_get_pem(self.as_ptr())).receive() }
	}

	#[inline]
	/// [xmpp_tlscert_get_dnsname](https://strophe.im/libstrophe/doc/0.13.0/group___t_l_s.html#ga586b6294d680cf13b2390c4ee5d6c3ce)
	pub fn get_dns_name(&self, n: usize) -> Option<&str> {
		unsafe { FFI(sys::xmpp_tlscert_get_dnsname(self.as_ptr(), n)).receive() }
	}

	#[inline]
	/// [xmpp_tlscert_get_string](https://strophe.im/libstrophe/doc/0.13.0/group___t_l_s.html#ga1b9715dbf4c363f587a8d48c072e78b9)
	pub fn get_string(&self, element: CertElement) -> Option<&str> {
		unsafe { FFI(sys::xmpp_tlscert_get_string(self.as_ptr(), element)).receive() }
	}

	#[inline]
	/// [xmpp_tlscert_get_description](https://strophe.im/libstrophe/doc/0.13.0/group___t_l_s.html#ga3373a412085f6c3a3db9adb2c49d9d07)
	pub fn get_element_description(element: CertElement) -> Option<&'static str> {
		unsafe { FFI(sys::xmpp_tlscert_get_description(element)).receive() }
	}
}

impl Drop for TlsCert {
	#[inline]
	/// [xmpp_tlscert_free](https://strophe.im/libstrophe/doc/0.13.0/group___t_l_s.html#ga6d01550c3a62c21cf4536c83eca97b1e)
	fn drop(&mut self) {
		if self.owned {
			unsafe {
				sys::xmpp_tlscert_free(self.inner.as_mut());
			}
		}
	}
}

impl fmt::Debug for TlsCert {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut debug = f.debug_struct("TlsCert");
		let elems = [
			CertElement::XMPP_CERT_VERSION,
			CertElement::XMPP_CERT_SERIALNUMBER,
			CertElement::XMPP_CERT_SUBJECT,
			CertElement::XMPP_CERT_ISSUER,
			CertElement::XMPP_CERT_NOTBEFORE,
			CertElement::XMPP_CERT_NOTAFTER,
			CertElement::XMPP_CERT_KEYALG,
			CertElement::XMPP_CERT_SIGALG,
			CertElement::XMPP_CERT_FINGERPRINT_SHA1,
			CertElement::XMPP_CERT_FINGERPRINT_SHA256,
		];
		for elem in elems {
			if let Some(name) = TlsCert::get_element_description(elem) {
				debug.field(name, &self.get_string(elem).unwrap_or("<unreadable value or non-utf8>"));
			}
		}
		let mut n = 0;
		while let Some(dns) = self.get_dns_name(n) {
			debug.field("DNS Name", &dns);
			n += 1;
		}
		debug.finish()
	}
}
