use libstrophe::Stanza;

fn main() {
	// try to use iterator when stanza dropped
	let _a = {
		let stanza = Stanza::new();
		let child_iter = stanza.children();
		drop(stanza);
		child_iter
	};

	// try to use child stanza when iterator dropped
	let _a = {
		let stanza = Stanza::new();
		let mut child_iter = stanza.children();
		let child = child_iter.next().unwrap();
		drop(child_iter);
		drop(stanza);
		child
	};

	// try to use mutable iterator when stanza dropped
	let _a = {
		let mut stanza = Stanza::new();
		let child_iter = stanza.children_mut();
		drop(stanza);
		child_iter
	};

	// try to use child stanza when mutable iterator dropped
	let _a = {
		let stanza = Stanza::new();
		let mut child_iter = stanza.children_mut();
		let child = child_iter.next().unwrap();
		drop(child_iter);
		drop(stanza);
		child
	};
}
