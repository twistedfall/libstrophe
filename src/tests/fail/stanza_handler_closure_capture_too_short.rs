use libstrophe::{Connection, Context, HandlerResult};

fn main() {
	{
		let mut val = 0;
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.handler_add(
			|_, _, _| {
				val = 1;
				HandlerResult::RemoveHandler
			},
			None,
			None,
			None,
		);
		conn
	};
}
