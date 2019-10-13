// hack to emulate `extern crate libstrophe`
mod libstrophe {
	pub use crate::*;
}

/// Port of the [bot.c](https://github.com/strophe/libstrophe/blob/0.9.2/examples/bot.c) code
#[allow(dead_code)]
pub fn main() {
	env_logger::init();

	let jid = "test@example.com";
	let pass = "<password>";

	let version_handler = |_ctx: &libstrophe::Context,
	                       conn: &mut libstrophe::Connection,
	                       stanza: &libstrophe::Stanza| {
		eprintln!("Received version request from {}", stanza.from().expect("Empty from"));

		let mut reply = stanza.reply();
		reply.set_stanza_type("result").expect("Cannot set stanza type");

		let mut query = libstrophe::Stanza::new();
		query.set_name("query").expect("Cannot set stanza name");
		if let Some(ns) = stanza.get_first_child().expect("No children").ns() {
			query.set_ns(ns).expect("Cannot set stanza ns");
		}

		let mut name = libstrophe::Stanza::new();
		name.set_name("name").expect("Cannot set stanza name");

		let mut text = libstrophe::Stanza::new();
		text.set_text("libstrophe example bot").expect("Cannot set stanza text");
		name.add_child(text).expect("Cannot add child");

		query.add_child(name).expect("Cannot add child");

		let mut version = libstrophe::Stanza::new();
		version.set_name("version").expect("Cannot set stanza name");

		let mut text = libstrophe::Stanza::new();
		text.set_text("1.0").expect("Cannot set stanza text");
		version.add_child(text).expect("Cannot add child");

		query.add_child(version).expect("Cannot add child");

		reply.add_child(query).expect("Cannot add child");

		conn.send(&reply);
		true
	};

	let message_handler = |_ctx: &libstrophe::Context,
	                       conn: &mut libstrophe::Connection,
	                       stanza: &libstrophe::Stanza| {
		let body = match stanza.get_child_by_name("body") {
			Some(body) => body,
			None => return true,
		};

		match stanza.stanza_type() {
			Some(typ) => if typ == "error"{
				return true
			},
			None => return true,
		};

		let intext = body.text().expect("Cannot get body");

		eprintln!("Incoming message from {}: {}", stanza.from().expect("Cannot get from"), intext);

		let mut reply = stanza.reply();
		if reply.stanza_type().is_none() {
			reply.set_stanza_type("chat").expect("Cannot set type");
		}

		let (quit, replytext) = if intext == "quit" {
			(true, "bye!".to_owned())
		} else {
			(false, format!("{} to you too!", intext))
		};
		reply.set_body(replytext).expect("Cannot set body");

		conn.send(&reply);

		if quit {
			conn.disconnect();
		}

		true
	};

	let conn_handler = move |ctx: &libstrophe::Context,
	                         conn: &mut libstrophe::Connection,
	                         evt: libstrophe::ConnectionEvent,
	                         _error: i32,
	                         _stream_error: Option<&libstrophe::error::StreamError>| {
		if evt == libstrophe::ConnectionEvent::XMPP_CONN_CONNECT {
			eprintln!("Connected");
			conn.handler_add(version_handler, Some("jabber:iq:version"), Some("iq"), None);
			conn.handler_add(message_handler, None, Some("message"), None);
			let pres = libstrophe::Stanza::new_presence();
			conn.send(&pres);
		} else {
			eprintln!("Disconnected");
			ctx.stop();
		}
	};

	let mut conn = libstrophe::Connection::new(libstrophe::Context::new_with_default_logger());
	conn.set_jid(jid);
	conn.set_pass(pass);
	let ctx = conn.connect_client(None, None, &conn_handler).expect("Cannot connect to XMPP server");
	ctx.run();
	libstrophe::shutdown();
}
