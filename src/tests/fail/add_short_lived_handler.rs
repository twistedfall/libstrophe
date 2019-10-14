use libstrophe::{Connection, Context, Stanza, StreamError};

fn main() {
	let handler = |_: &Context, conn: &mut Connection, _, _, _: Option<&StreamError>| {
		let handler = |_: &Context, _: &mut Connection, _: &Stanza| { false };
		conn.handler_add(&handler, None, None, None);
	};
	let conn = Connection::new(Context::new_with_null_logger());
	conn.connect_client(None, None, &handler);
}
