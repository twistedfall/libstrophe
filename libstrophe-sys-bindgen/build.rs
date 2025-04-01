#[cfg(feature = "buildtime_bindgen")]
fn build_wrapper() {
	use std::env;
	use std::path::PathBuf;

	use bindgen::callbacks::{IntKind, MacroParsingBehavior, ParseCallbacks};

	#[derive(Debug)]
	struct PCallbacks;

	impl ParseCallbacks for PCallbacks {
		fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
			if name.starts_with("XMPP_") {
				MacroParsingBehavior::Default
			} else {
				MacroParsingBehavior::Ignore
			}
		}

		fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
			if name.starts_with("XMPP_E") {
				Some(IntKind::Int)
			} else if name.starts_with("XMPP_CONN_FLAG_") {
				Some(IntKind::Long)
			} else {
				None
			}
		}
	}

	let mut builder = bindgen::builder().header("wrapper.h").parse_callbacks(Box::new(PCallbacks));

	const BLOCKLIST_TYPES: &[&str] = &["max_align_t", "wchar_t", "__fsid_t"];
	for blocklist_type in BLOCKLIST_TYPES {
		builder = builder.blocklist_type(blocklist_type);
	}

	const RUSTIFIED_ENUMS: &[&str] = &[
		"xmpp_log_level_t",
		"xmpp_conn_type_t",
		"xmpp_conn_event_t",
		"xmpp_error_type_t",
		"xmpp_cert_element_t",
		"xmpp_queue_element_t",
	];
	for rustified_enum in RUSTIFIED_ENUMS {
		builder = builder.rustified_enum(rustified_enum);
	}

	// Write the bindings to the src/ffi.rs file.
	let mut out_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Can't read CARGO_MANIFEST_DIR env var"));
	out_path.push("src/ffi.rs");
	builder
		.generate()
		.expect("Unable to generate bindings")
		.write_to_file(&out_path)
		.unwrap_or_else(|e| panic!("Couldn't write bindings to: {}, error: {e}", out_path.display()));
}

fn main() {
	println!("cargo:rustc-link-lib=strophe");
	#[cfg(feature = "buildtime_bindgen")]
	build_wrapper();
}
