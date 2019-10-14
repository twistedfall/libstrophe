use libstrophe::{Connection, Context, Stanza};

fn main() {
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	{
		let handler = |_: &Context, _: &mut Connection, _: &Stanza| { false };
		conn.handler_add(&handler, None, None, None);
		conn
	};
}
