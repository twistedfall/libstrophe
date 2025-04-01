use crate::{ALLOC_CONTEXT, FFI};

#[inline]
/// [xmpp_jid_new](https://strophe.im/libstrophe/doc/0.13.0/jid_8c.html#af7473f6e7a3338ec42d1c874ae563a47)
pub fn jid_new(node: Option<&str>, domain: impl AsRef<str>, resource: Option<&str>) -> Option<String> {
	let node = FFI(node).send();
	let domain = FFI(domain.as_ref()).send();
	let resource = FFI(resource).send();
	unsafe {
		FFI(sys::xmpp_jid_new(
			ALLOC_CONTEXT.as_ptr(),
			node.as_ptr(),
			domain.as_ptr(),
			resource.as_ptr(),
		))
		.receive_with_free(|x| ALLOC_CONTEXT.free(x))
	}
}

#[inline]
/// [xmpp_jid_bare](https://strophe.im/libstrophe/doc/0.13.0/jid_8c.html#ab4e8f0b359e076e08003e5c8559da768)
pub fn jid_bare(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe { FFI(sys::xmpp_jid_bare(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x)) }
}

#[inline]
/// [xmpp_jid_node](https://strophe.im/libstrophe/doc/0.13.0/jid_8c.html#afc32b798cbd4dd86d44036f0b0b4ede5)
pub fn jid_node(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe { FFI(sys::xmpp_jid_node(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x)) }
}

#[inline]
/// [xmpp_jid_domain](https://strophe.im/libstrophe/doc/0.13.0/jid_8c.html#a22fcada690c1dcf712feeb22128d2de7)
pub fn jid_domain(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe { FFI(sys::xmpp_jid_domain(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x)) }
}

#[inline]
/// [xmpp_jid_resource](https://strophe.im/libstrophe/doc/0.13.0/jid_8c.html#ad9a6f65b0943d09b5159ec5eea379781)
pub fn jid_resource(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe { FFI(sys::xmpp_jid_resource(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x)) }
}
