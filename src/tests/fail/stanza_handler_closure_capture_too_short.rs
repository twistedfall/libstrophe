use libstrophe::{Context, Connection};

fn main() {
	{
		let mut val = 0;
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.handler_add(|_, _, _| {
			val = 1;
			false
		}, None, None, None);
		conn
	};
}
