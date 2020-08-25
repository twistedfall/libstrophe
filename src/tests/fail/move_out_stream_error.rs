use libstrophe::{Connection, Context, StreamError};
use std::sync::Mutex;

fn main() {
	let stream_error = Mutex::new(Option::<StreamError>::None);
	let mut _handler = {
		move |_: &Context, _: &mut Connection, _: i32, _: i32, err: Option<StreamError>| {
			*stream_error.lock().unwrap() = err;
		}
	};
//		let conn = Connection::new(Context::new_with_null_logger());
//		conn.connect_client(None, None, &mut handler);
}
