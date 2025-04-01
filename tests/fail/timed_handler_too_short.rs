use libstrophe::{Connection, Context, HandlerResult};
use std::time::Duration;

fn main() {
	let mut conn = Connection::new(Context::new_with_null_logger());
	{
		let handler = |_: &Context, _: &mut Connection| HandlerResult::RemoveHandler;
		conn.timed_handler_add(&handler, Duration::from_secs(1));
		conn
	};
}
