use libstrophe::{Connection, Context, StanzaResult};

fn main() {
	{
		let mut val = 0;
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.handler_add(
			|_, _, _| {
				val = 1;
				StanzaResult::Remove
			},
			None,
			None,
			None,
		);
		conn
	};
}
