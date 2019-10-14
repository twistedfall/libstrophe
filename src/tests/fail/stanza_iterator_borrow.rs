use libstrophe::Stanza;

fn main() {
	let mut stanza = Stanza::new();
	let _a = stanza.children();
	stanza.get_first_child_mut();
}
