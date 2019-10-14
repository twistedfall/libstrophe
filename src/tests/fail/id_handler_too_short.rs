use libstrophe::{Connection, Context, Stanza};

fn main() {
	let mut conn = Connection::new(Context::new_with_null_logger());
	{
		let handler = |_: &Context, _: &mut Connection, _: &Stanza| { false };
		conn.id_handler_add(&handler, "id");
		conn
	};
}
