use libstrophe::{Context, Connection};

fn main() {
	{
		let mut val = 0;
		let conn = Connection::new(Context::new_with_null_logger());
		conn.connect_client(None, None, |_, _, _, _, _| {
			val = 1;
		}).unwrap()
	};
}
