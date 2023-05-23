#[cfg(feature = "buildtime_bindgen")]
fn build_wrapper() {
	use bindgen::callbacks;
	use std::{env, path::PathBuf};

	#[derive(Debug)]
	struct PCallbacks;

	impl callbacks::ParseCallbacks for PCallbacks {
		fn int_macro(&self, name: &str, _value: i64) -> Option<callbacks::IntKind> {
			if name == "XMPP_EOK" {
				Some(callbacks::IntKind::I32)
			} else {
				None
			}
		}
	}

	let builder = bindgen::builder()
		.header("wrapper.h")
		.size_t_is_usize(true)
		.parse_callbacks(Box::new(PCallbacks))
		.blocklist_type("max_align_t")
		.blocklist_type("wchar_t")
		.rustified_enum("xmpp_log_level_t")
		.rustified_enum("xmpp_conn_type_t")
		.rustified_enum("xmpp_conn_event_t")
		.rustified_enum("xmpp_error_type_t")
		.rustified_enum("xmpp_cert_element_t")
		.rustified_enum("xmpp_queue_element_t");

	let bindings = builder.generate().expect("Unable to generate bindings");

	// Write the bindings to the src/ffi.rs file.
	let mut out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	out_path.push("src/ffi.rs");
	bindings
		.write_to_file(&out_path)
		.expect(&format!("Couldn't write bindings to: {}", out_path.display()));
}

fn main() {
	println!("cargo:rustc-link-lib=strophe");
	#[cfg(feature = "buildtime_bindgen")]
	build_wrapper();
}
