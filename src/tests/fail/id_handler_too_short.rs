use libstrophe::{Connection, Context, Stanza, StanzaResult};

fn main() {
	let mut conn = Connection::new(Context::new_with_null_logger());
	{
		let handler = |_: &Context, _: &mut Connection, _: &Stanza| StanzaResult::Remove;
		conn.id_handler_add(&handler, "id");
		conn
	};
}
