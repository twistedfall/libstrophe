//! Port of https://github.com/strophe/libstrophe/blob/0.14.0/examples/bot.c

use libstrophe::{Connection, ConnectionEvent, Context, HandlerResult, Stanza};

pub fn main() {
	env_logger::init();

	let jid = "test@example.com";
	let pass = "<password>";

	let version_handler = |_ctx: &Context, conn: &mut Connection, stanza: &Stanza| {
		eprintln!("Received version request from {}", stanza.from().expect("Empty from"));

		let mut reply = stanza.reply();
		reply.set_stanza_type("result").expect("Cannot set stanza type");

		let mut query = Stanza::new();
		query.set_name("query").expect("Cannot set stanza name");
		if let Some(ns) = stanza.get_first_child().expect("No children").ns() {
			query.set_ns(ns).expect("Cannot set stanza ns");
		}

		let mut name = Stanza::new();
		name.set_name("name").expect("Cannot set stanza name");

		let mut text = Stanza::new();
		text.set_text("libstrophe example bot").expect("Cannot set stanza text");
		name.add_child(text).expect("Cannot add child");

		query.add_child(name).expect("Cannot add child");

		let mut version = Stanza::new();
		version.set_name("version").expect("Cannot set stanza name");

		let mut text = Stanza::new();
		text.set_text("1.0").expect("Cannot set stanza text");
		version.add_child(text).expect("Cannot add child");

		query.add_child(version).expect("Cannot add child");

		reply.add_child(query).expect("Cannot add child");

		conn.send(&reply);
		HandlerResult::KeepHandler
	};

	let message_handler = |_ctx: &Context, conn: &mut Connection, stanza: &Stanza| {
		let body = match stanza.get_child_by_name("body") {
			Some(body) => body,
			None => return HandlerResult::KeepHandler,
		};

		match stanza.stanza_type() {
			Some(typ) => {
				if typ == "error" {
					return HandlerResult::KeepHandler;
				}
			}
			None => return HandlerResult::KeepHandler,
		};

		let intext = body.text().expect("Cannot get body");

		eprintln!("Incoming message from {}: {intext}", stanza.from().expect("Cannot get from"));

		let mut reply = stanza.reply();
		if reply.stanza_type().is_none() {
			reply.set_stanza_type("chat").expect("Cannot set type");
		}

		let (quit, replytext) = if intext == "quit" {
			(true, "bye!".to_owned())
		} else {
			(false, format!("{intext} to you too!"))
		};
		reply.set_body(replytext).expect("Cannot set body");

		conn.send(&reply);

		if quit {
			conn.disconnect();
		}

		HandlerResult::KeepHandler
	};

	let conn_handler = move |ctx: &Context, conn: &mut Connection, evt: ConnectionEvent| {
		if let ConnectionEvent::Connect = evt {
			eprintln!("Connected");
			conn.handler_add(version_handler, Some("jabber:iq:version"), Some("iq"), None);
			conn.handler_add(message_handler, None, Some("message"), None);
			let pres = Stanza::new_presence();
			conn.send(&pres);
		} else {
			eprintln!("Disconnected");
			ctx.stop();
		}
	};

	let mut conn = Connection::new(Context::new_with_default_logger());
	conn.set_jid(jid);
	conn.set_pass(pass);
	let mut ctx = conn
		.connect_client(None, None, conn_handler)
		.expect("Cannot connect to XMPP server");
	ctx.run();
	libstrophe::shutdown();
}
