extern crate matches;

use std::time;

use super::{
	error,
	Connection,
	ConnectionEvent,
	Context,
	Logger,
	Stanza,
};

#[test]
fn examples() {
//	super::examples::bot_fn_safe::main();
//	super::examples::bot_closure_unsafe::main();
}

#[test]
fn default_context() {
	let _ = Context::new_with_default_logger();
}

#[test]
fn default_logger() {
	Logger::default();
}

#[test]
fn custom_logger() {
	let log_handler = |_level: super::LogLevel,
	                   _area: &str,
	                   _msg: &str| {};
	let logger = Logger::new(&log_handler);
	Context::new(logger);
}

#[test]
fn conn_client() {
	let conn_handler = |conn: &mut Connection,
	                    event: ConnectionEvent,
	                    _error: i32,
	                    _stream_error: Option<&error::StreamError>, | {
		assert_eq!(event, ConnectionEvent::XMPP_CONN_DISCONNECT);
		if event == ConnectionEvent::XMPP_CONN_CONNECT {
			let ctx = conn.context();
			conn.send(&Stanza::new_presence(&ctx));
			conn.disconnect();
		} else if event == ConnectionEvent::XMPP_CONN_DISCONNECT {
			conn.context().stop();
		}
	};
	let ctx = Context::new_with_default_logger();
	let mut con = Connection::new(ctx.clone());
	// no jid supplied
	assert_matches!(con.connect_client(None, None, &conn_handler), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
	con.set_jid("test-jid@127.50.60.70");
	con.connect_client(None, None, &conn_handler).unwrap();
	ctx.run();
}

#[test]
fn conn_raw() {
	let conn_handler = |conn: &mut Connection,
	                    event: ConnectionEvent,
	                    _error: i32,
	                    _stream_error: Option<&error::StreamError>, | {
		assert_eq!(event, ConnectionEvent::XMPP_CONN_DISCONNECT);
		if event == ConnectionEvent::XMPP_CONN_CONNECT {
			let ctx = conn.context();
			conn.send(&Stanza::new_presence(&ctx));
			conn.disconnect();
		} else if event == ConnectionEvent::XMPP_CONN_DISCONNECT {
			conn.context().stop();
		}
	};
	let ctx = Context::new_with_default_logger();
	let mut con = Connection::new(ctx.clone());
	// no jid supplied
	assert_matches!(con.connect_client(None, None, &conn_handler), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
	con.set_jid("test-jid@127.50.60.70");
	con.connect_raw(None, None, &conn_handler).unwrap();
	ctx.run();
}

#[test]
fn timed_handler() {
	let timed_handler = |_conn: &mut Connection| { false };
	let ctx = Context::new_with_default_logger();
	let mut con = Connection::new(ctx);
	con.timed_handler_add(&timed_handler, time::Duration::from_secs(1));
	con.timed_handler_delete(&timed_handler);
	unsafe { con.timed_handler_add_unsafe(&timed_handler, time::Duration::from_secs(1)) };
	con.timed_handler_delete(&timed_handler);
}

#[test]
fn stanza_handler() {
	let stanza_handler = |_conn: &mut Connection,
	                      _stanza: &Stanza| { false };
	let ctx = Context::new_with_default_logger();
	let mut con = Connection::new(ctx);
	con.handler_add(&stanza_handler, Some("ns"), None, None);
	con.handler_delete(&stanza_handler);
	unsafe { con.handler_add_unsafe(&stanza_handler, None, Some(&"name".to_owned()), None) };
	con.handler_delete(&stanza_handler);
}

#[test]
fn id_handler() {
	let id_handler = |_conn: &mut Connection,
	                  _stanza: &Stanza| { false };
	let ctx = Context::new_with_default_logger();
	let mut con = Connection::new(ctx);
	con.id_handler_add(&id_handler, "test");
	con.id_handler_delete(&id_handler, "test");
	unsafe { con.id_handler_add_unsafe(&id_handler, "test") };
	con.id_handler_delete(&id_handler, "test");
}

#[test]
fn stanza_handler_in_conn() {
	let stanza_handler = |_conn: &mut Connection,
	                      _stanza: &Stanza| { false };
	let conn_handler = move |conn: &mut Connection,
	                         _event: ConnectionEvent,
	                         _error: i32,
	                         _stream_error: Option<&error::StreamError>, | {
		unsafe { conn.handler_add_unsafe(&stanza_handler, None, None, None) };
	};
	let ctx = Context::new_with_default_logger();
	let mut con = Connection::new(ctx);
	con.set_jid("test-jid@127.50.60.70");
	con.connect_client(None, None, &conn_handler).unwrap();
}

#[test]
fn eq_test() {
	let ctx = Context::new_with_default_logger();

	{
		let stanza = Stanza::new(&ctx);
		assert_eq!(*ctx, *stanza.context());
	}

	{
		let conn = Connection::new(ctx.clone());
		assert_eq!(*ctx, *conn.context());
	}
}

#[test]
fn stanza_err() {
	let ctx = Context::new_with_default_logger();

	let mut stanza = Stanza::new(&ctx);
	assert_matches!(stanza.to_text(), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
	stanza.set_name("test").unwrap();
	assert_matches!(stanza.set_body("body"), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
}

#[test]
fn stanza_display() {
	let ctx = Context::new_with_default_logger();

	let mut stanza = Stanza::new(&ctx);
	stanza.set_name("message").unwrap();
	stanza.set_id("stanza_id").unwrap();
	stanza.add_child(Stanza::new_iq(&ctx, Some("test"), None)).unwrap();
	assert_eq!(stanza.to_string(), stanza.to_text().unwrap());
}

#[test]
fn stanza_hier() {
	let ctx = Context::new_with_default_logger();

	let mut stanza = Stanza::new(&ctx);
	stanza.set_name("test").unwrap();
	stanza.add_child(Stanza::new_presence(&ctx)).unwrap();
	stanza.add_child(Stanza::new_iq(&ctx, Some("test"), None)).unwrap();
	let mut msg = Stanza::new_message(&ctx, Some("chat"), Some("id"), Some("to"));
	msg.set_body("Test body").unwrap();
	stanza.add_child(msg).unwrap();

	let child = stanza.get_first_child().unwrap();
	assert_eq!(child.name().unwrap(), "presence");
	let child = child.get_next().unwrap();
	assert_eq!(child.name().unwrap(), "iq");
	let mut child = stanza.get_first_child_mut().unwrap();
	let mut child = child.get_next_mut().unwrap();
	let child = child.get_next_mut().unwrap();
	assert_eq!(child.name().unwrap(), "message");
	assert_eq!(child.body().unwrap(), "Test body");
}

#[test]
fn stanza() {
	let ctx = Context::new_with_default_logger();

	let mut stanza = Stanza::new(&ctx);
	stanza.set_name("message").unwrap();
	stanza.set_id("stanza_id").unwrap();

	let stanza2 = Stanza::new_message(&ctx, None, Some("stanza_id"), None);
	assert_eq!(stanza.name(), stanza2.name());
	assert_eq!(stanza.stanza_type(), stanza2.stanza_type());
	assert_eq!(stanza.id(), stanza2.id());
	assert_eq!(stanza.to(), stanza2.to());
	assert_eq!(stanza.body(), stanza2.body());

	stanza.set_name("presence").unwrap();
	let stanza2 = Stanza::new_presence(&ctx);
	assert_eq!(stanza.name(), stanza2.name());

	let mut stanza3 = stanza.clone();
	stanza3.set_id("iq").unwrap();
	assert_ne!(stanza.id(), stanza3.id());
}

// All of those and only those functions must fail compilation due to lifetime problems
#[cfg(feature = "fail-test")]
mod fail {
	use super::{
		error,
		time,
		Connection,
		ConnectionEvent,
		Context,
		Logger,
		Stanza,
	};
	use super::super::LogLevel;

	#[test]
	fn conn_handler_too_short() {
		let mut con = Connection::new(Context::new_with_default_logger());
		let not_long_enough1 = |_conn: &mut Connection,
		                        _event: ConnectionEvent,
		                        _error: i32,
		                        _stream_error: Option<&error::StreamError>, | {};
		con.connect_client(None, None, &not_long_enough1).unwrap();
	}

	#[test]
	fn stanza_handler_too_short() {
		let ctx = Context::new_with_default_logger();
		let mut con = Connection::new(ctx);
		{
			let not_long_enough2 = |_conn: &mut Connection,
			                        _stanza: &Stanza| { false };
			con.handler_add(&not_long_enough2, None, None, None);
			con
		};
	}

	#[test]
	fn timed_handler_too_short() {
		let mut con = Connection::new(Context::new_with_default_logger());
		{
			let not_long_enough3 = |_conn: &mut Connection| { false };
			con.timed_handler_add(&not_long_enough3, time::Duration::from_secs(1));
			con
		};
	}

	#[test]
	fn id_handler_too_short() {
		let mut con = Connection::new(Context::new_with_default_logger());
		{
			let not_long_enough4 = |_conn: &mut Connection,
			                        _stanza: &Stanza| { false };
			con.id_handler_add(&not_long_enough4, "id");
			con
		};
	}

	#[test]
	fn stanza_too_short() {
		let ctx = Context::new_with_default_logger();
		{
			let not_long_enough5 = Stanza::new(&ctx);
			not_long_enough5.get_first_child()
		};
	}

	#[test]
	fn logger_too_short() {
		{
			let not_long_enough6 = |_level: LogLevel,
			                   _area: &str,
			                   _msg: &str| {};
			let logger = Logger::new(&not_long_enough6);
			Context::new(logger)
		};
	}
}
