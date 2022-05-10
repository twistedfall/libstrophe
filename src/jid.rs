use crate::{ALLOC_CONTEXT, FFI};

#[inline]
/// [xmpp_jid_new](https://github.com/strophe/libstrophe/blob/0.10.0/src/jid.c#L21-L30)
pub fn jid_new(node: Option<&str>, domain: impl AsRef<str>, resource: Option<&str>) -> Option<String> {
	let node = FFI(node).send();
	let domain = FFI(domain.as_ref()).send();
	let resource = FFI(resource).send();
	unsafe {
		FFI(sys::xmpp_jid_new(ALLOC_CONTEXT.as_ptr(), node.as_ptr(), domain.as_ptr(), resource.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x))
	}
}

#[inline]
/// [xmpp_jid_bare](https://github.com/strophe/libstrophe/blob/0.10.0/src/jid.c#L67-L73)
pub fn jid_bare(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe {
		FFI(sys::xmpp_jid_bare(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x))
	}
}

#[inline]
/// [xmpp_jid_node](https://github.com/strophe/libstrophe/blob/0.10.0/src/jid.c#L89-L96)
pub fn jid_node(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe {
		FFI(sys::xmpp_jid_node(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x))
	}
}

#[inline]
/// [xmpp_jid_domain](https://github.com/strophe/libstrophe/blob/0.10.0/src/jid.c#L114-L120)
pub fn jid_domain(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe {
		FFI(sys::xmpp_jid_domain(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x))
	}
}

#[inline]
/// [xmpp_jid_resource](https://github.com/strophe/libstrophe/blob/0.10.0/src/jid.c#L145-L152)
pub fn jid_resource(jid: impl AsRef<str>) -> Option<String> {
	let jid = FFI(jid.as_ref()).send();
	unsafe {
		FFI(sys::xmpp_jid_resource(ALLOC_CONTEXT.as_ptr(), jid.as_ptr())).receive_with_free(|x| ALLOC_CONTEXT.free(x))
	}
}
