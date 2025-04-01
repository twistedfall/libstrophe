use libstrophe::{Connection, ConnectionEvent, Context, HandlerResult, Stanza};

fn main() {
	let handler = |_: &Context, conn: &mut Connection, _: ConnectionEvent| {
		let handler = |_: &Context, _: &mut Connection, _: &Stanza| HandlerResult::RemoveHandler;
		conn.handler_add(&handler, None, None, None);
	};
	let conn = Connection::new(Context::new_with_null_logger());
	conn.connect_client(None, None, &handler);
}
