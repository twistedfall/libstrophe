use libstrophe::{Connection, ConnectionEvent, Context, StreamError};

fn main() {
	{
		let conn = Connection::new(Context::new_with_null_logger());
		let handler = |_: &Context, _: &mut Connection, _: ConnectionEvent| {};
		conn.connect_client(None, None, &handler).unwrap()
	};
}
