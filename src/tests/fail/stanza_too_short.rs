use libstrophe::Stanza;

fn main() {
	let _a = {
		let stanza = Stanza::new();
		stanza.get_first_child()
	};
}
