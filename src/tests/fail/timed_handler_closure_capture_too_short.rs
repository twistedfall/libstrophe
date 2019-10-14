use libstrophe::{Connection, Context};
use std::time::Duration;

fn main() {
	{
		let mut val = 0;
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.timed_handler_add(|_, _| {
			val = 1;
			false
		}, Duration::from_secs(1));
		conn
	};
}
