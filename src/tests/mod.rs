use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::{env, mem};

use matches::assert_matches;
use names::Generator;

use crate::*;

#[test]
fn examples() {
	//	super::examples::bot_fn::main();
	//	super::examples::bot_closure::main();
}

#[test]
fn default_context() {
	Context::new_with_default_logger();
}

#[test]
fn default_context_null() {
	Context::new_with_null_logger();
}

#[test]
fn default_logger() {
	Logger::default();
}

#[test]
fn null_logger() {
	Logger::new_null();
}

#[test]
fn custom_logger() {
	let i: AtomicU16 = AtomicU16::new(0);
	{
		let ctx = Context::new(Logger::new(|_, _, _| {
			i.fetch_add(1, Ordering::Relaxed);
		}));
		let mut conn = Connection::new(ctx);
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_client(None, Some(1234), |_, _, _| {}).unwrap();
		ctx.run_once(Duration::from_secs(1));
	}
	assert_eq!(i.load(Ordering::Relaxed), 5);
}

#[test]
fn conn_client_wo_jid() {
	let conn = Connection::new(Context::new_with_null_logger());
	// no JID supplied
	assert_matches!(
		conn.connect_client(None, None, |_, _, _| {}),
		Err(ConnectClientError {
			error: Error::InvalidOperation,
			..
		})
	);
}

#[test]
fn conn_client() {
	let conn_handler = |ctx: &Context, _: &mut Connection, event: ConnectionEvent| {
		assert_matches!(event, ConnectionEvent::Disconnect(_));
		ctx.stop();
	};

	// ref closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_client(None, None, conn_handler).unwrap();
		ctx.run();
	}

	// own closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_client(None, None, conn_handler).unwrap();
		ctx.run();
	}
}

#[test]
fn conn_raw() {
	let conn_handler = |ctx: &Context, _: &mut Connection, event: ConnectionEvent| {
		assert_matches!(event, ConnectionEvent::Disconnect(_));
		ctx.stop();
	};

	// ref closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_raw(None, Some(1234), conn_handler).unwrap();
		ctx.run();
	}

	// own closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_raw(None, Some(1234), conn_handler).unwrap();
		ctx.run();
	}
}

#[test]
fn timed_handler() {
	let timed_handler = |_: &Context, _: &mut Connection| HandlerResult::RemoveHandler;
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let handle = conn
		.timed_handler_add(&timed_handler, Duration::from_secs(1))
		.expect("Can't add timed handler");
	assert_matches!(conn.timed_handler_add(&timed_handler, Duration::from_secs(1)), None);
	conn.timed_handler_delete(handle);
}

#[test]
fn stanza_handler() {
	let stanza_handler = |_: &Context, _: &mut Connection, _: &Stanza| HandlerResult::RemoveHandler;
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let handle = conn
		.handler_add(&stanza_handler, Some("ns"), None, None)
		.expect("Can't add handler");
	assert_matches!(conn.handler_add(&stanza_handler, Some("ns"), None, None), None);
	conn.handler_delete(handle);
	let handle = conn
		.handler_add(stanza_handler, None, Some("name"), None)
		.expect("Can't add handler");
	conn.handler_delete(handle);
}

#[test]
fn id_handler() {
	let id_handler = |_: &Context, _: &mut Connection, _: &Stanza| HandlerResult::RemoveHandler;
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let h = conn.id_handler_add(&id_handler, "test").expect("Can't add id handler");
	assert_matches!(conn.id_handler_add(&id_handler, "test"), None);
	conn.id_handler_delete(h);
}

#[test]
fn stanza_handler_in_con() {
	let stanza_handler = |_: &Context, _: &mut Connection, _: &Stanza| HandlerResult::RemoveHandler;
	let con_handler = move |_: &Context, conn: &mut Connection, _: ConnectionEvent| {
		conn.handler_add(stanza_handler, None, None, None).expect("Can't add handler");
	};
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	conn.set_jid("test-JID@127.50.60.70");
	conn.connect_client(None, None, con_handler).unwrap();
}

#[test]
fn jid_test() {
	assert_eq!(
		Some("node@domain.com/test".to_string()),
		jid::jid_new(Some("node"), "domain.com", Some("test"))
	);
	assert_eq!(
		Some("domain.com/test".to_string()),
		jid::jid_new(None, "domain.com", Some("test"))
	);
	assert_eq!(Some("domain.com".to_string()), jid::jid_new(None, "domain.com", None));

	let jid = jid::jid_new(Some("node"), "domain.com", Some("test")).unwrap();
	let jid_domain = jid::jid_new(None, "domain.com", None).unwrap();
	assert_eq!(Some("node@domain.com".to_string()), jid::jid_bare(&jid));

	assert_eq!(Some("node".to_string()), jid::jid_node(&jid));
	assert_eq!(None, jid::jid_node(&jid_domain));

	assert_eq!(Some("domain.com".to_string()), jid::jid_domain(&jid));
	assert_eq!(Some("domain.com".to_string()), jid::jid_domain(&jid_domain));

	assert_eq!(Some("test".to_string()), jid::jid_resource(&jid));
	assert_eq!(None, jid::jid_resource(&jid_domain));
}

#[test]
fn stanza_err() {
	let mut stanza = Stanza::new();
	assert_matches!(stanza.to_text(), Err(ToTextError::StropheError(Error::InvalidOperation)));
	stanza.set_name("test").unwrap();
	assert_matches!(stanza.set_body("body"), Err(Error::InvalidOperation));
}

#[test]
fn stanza_display() {
	let mut stanza = Stanza::new();
	stanza.set_name("message").unwrap();
	stanza.set_id("stanza_id").unwrap();
	stanza.add_child(Stanza::new_iq(Some("test"), None)).unwrap();
	assert_eq!(stanza.to_string(), stanza.to_text().unwrap());
}

#[test]
fn stanza_hier() {
	let mut stanza = Stanza::new();
	stanza.set_name("test").unwrap();
	stanza.add_child(Stanza::new_presence()).unwrap();
	stanza.add_child(Stanza::new_iq(Some("test"), None)).unwrap();
	let mut msg = Stanza::new_message(Some("chat"), Some("id"), Some("to"));
	msg.set_body("Test body").unwrap();
	stanza.add_child(msg).unwrap();

	{
		let child = stanza.get_first_child().unwrap();
		assert_eq!(child.name().unwrap(), "presence");
		let child = child.get_next().unwrap();
		assert_eq!(child.name().unwrap(), "iq");
	}

	{
		let mut child = stanza.get_first_child_mut().unwrap();
		let mut child = child.get_next_mut().unwrap();
		let child = child.get_next_mut().unwrap();
		assert_eq!(child.name().unwrap(), "message");
		assert_eq!(child.body().unwrap(), "Test body");
	}

	{
		for (i, child) in stanza.children().enumerate() {
			assert_eq!(stanza.get_first_child().unwrap().name().unwrap(), "presence"); // simultaneous borrow test
			match i {
				0 => assert_eq!(child.name().unwrap(), "presence"),
				1 => assert_eq!(child.name().unwrap(), "iq"),
				2 => {
					assert_eq!(child.name().unwrap(), "message");
					assert_eq!(child.body().unwrap(), "Test body");
				}
				_ => panic!("Too many items: {}", child),
			}
		}
	}

	{
		for (i, mut child) in stanza.children_mut().enumerate() {
			match i {
				0 => {
					assert_eq!(child.name().unwrap(), "presence");
					child.set_name("presence1").unwrap();
				}
				1 => assert_eq!(child.name().unwrap(), "iq"),
				2 => {
					assert_eq!(child.name().unwrap(), "message");
					assert_eq!(child.body().unwrap(), "Test body");
				}
				_ => panic!("Too many items: {}", child),
			}
		}
		assert_eq!(stanza.get_first_child().unwrap().name().unwrap(), "presence1");
	}
}

#[test]
fn stanza_get_child() {
	let mut root = Stanza::new();
	root.set_name("test").unwrap();
	let presence = Stanza::new_presence();
	root.add_child(presence.clone()).unwrap();
	let mut iq_ns = Stanza::new_iq(Some("test"), None);
	iq_ns.set_ns("iq_namespace").unwrap();
	root.add_child(iq_ns.clone()).unwrap();
	let mut msg = Stanza::new_message(Some("chat"), Some("id"), Some("to"));
	msg.set_body("Test body").unwrap();
	root.add_child(msg.clone()).unwrap();

	assert_eq!(presence.to_string(), root.get_child_by_name("presence").unwrap().to_string());
	assert_eq!(iq_ns.to_string(), root.get_child_by_ns("iq_namespace").unwrap().to_string());
	#[cfg(feature = "libstrophe-0_10_0")]
	{
		assert_eq!(
			iq_ns.to_string(),
			root.get_child_by_name_and_ns("iq", "iq_namespace").unwrap().to_string()
		);
		assert_eq!(None, root.get_child_by_name_and_ns("presence", "iq_namespace"));
	}
	#[cfg(feature = "libstrophe-0_12_0")]
	{
		assert_eq!(root.to_string(), root.get_child_by_path(&["test"]).unwrap().to_string());
		assert_eq!(
			iq_ns.to_string(),
			root
				.get_child_by_path(&["test", &XMPP_STANZA_NAME_IN_NS("iq", "iq_namespace")])
				.unwrap()
				.to_string()
		);
		assert_eq!(
			msg.clone().to_string(), // additional clone is needed because clone() changes the order of the attributes so the comparison below will fail
			root.get_child_by_path(&["test", "message"]).unwrap().to_string()
		);
		assert_eq!(
			"<body>Test body</body>",
			root.get_child_by_path(&["test", "message", "body"]).unwrap().to_string()
		);

		root
			.get_child_by_path_mut(&["test", "message"])
			.unwrap()
			.set_to("New to")
			.unwrap();
		assert_eq!("New to", root.get_child_by_name("message").unwrap().to().unwrap());
	}
}

#[test]
fn stanza() {
	let mut stanza = Stanza::new();
	stanza.set_name("message").unwrap();
	stanza.set_id("stanza_id").unwrap();

	let stanza2 = Stanza::new_message(None, Some("stanza_id"), None);
	assert_eq!(stanza.name(), stanza2.name());
	assert_eq!(stanza.stanza_type(), stanza2.stanza_type());
	assert_eq!(stanza.id(), stanza2.id());
	assert_eq!(stanza.to(), stanza2.to());
	assert_eq!(stanza.body(), stanza2.body());

	stanza.set_name("presence").unwrap();
	let stanza2 = Stanza::new_presence();
	assert_eq!(stanza.name(), stanza2.name());

	let mut stanza3 = stanza.clone();
	stanza3.set_id("iq").unwrap();
	assert_ne!(stanza.id(), stanza3.id());
}

#[test]
fn stanza_clone() {
	let stanza = {
		let mut stanza = Stanza::new();
		stanza.set_name("message").unwrap();
		stanza.set_id("stanza_id").unwrap();
		#[allow(clippy::redundant_clone)]
		stanza.clone()
	};
	assert_eq!("<message id=\"stanza_id\"/>", stanza.to_text().unwrap());
}

#[test]
fn stanza_attributes() {
	let mut stanza = Stanza::new();

	assert_matches!(stanza.set_id("stanza_id"), Err(Error::InvalidOperation));
	assert_eq!(stanza.attribute_count(), 0);

	stanza.set_name("message").unwrap();
	stanza.set_id("stanza_id").unwrap();
	stanza.set_stanza_type("type").unwrap();
	stanza.set_ns("myns").unwrap();

	assert_eq!(stanza.attribute_count(), 3);
	assert_matches!(stanza.get_attribute("type"), Some("type"));
	assert_matches!(stanza.get_attribute("non-existent"), None);

	stanza.set_attribute("xmlns", "myotherns").unwrap();
	assert_matches!(stanza.ns(), Some("myotherns"));

	let mut compare = HashMap::new();
	compare.insert("xmlns", "myotherns");
	compare.insert("type", "type");
	compare.insert("id", "stanza_id");
	assert_eq!(stanza.attributes(), compare);

	stanza.del_attribute("type").unwrap();
	assert_eq!(stanza.attribute_count(), 2);
	assert_matches!(stanza.get_attribute("type"), None);
	compare.remove("type");
	assert_eq!(stanza.attributes(), compare);
}

#[test]
#[cfg(feature = "libstrophe-0_10_0")]
fn stanza_from_str() {
	let s = Stanza::from_str("<test><child1/><child2/></test>");
	let mut children = s.children();
	assert_eq!(Some("<child1/>".to_string()), children.next().map(|c| c.to_string()));
	assert_eq!(Some("<child2/>".to_string()), children.next().map(|c| c.to_string()));
	assert_eq!(None, children.next().as_deref());
}

#[test]
fn zero_sized_handlers() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};
	let i = Arc::new(RwLock::new(0));

	{
		let i_incrementer = {
			let i = i.clone();
			move |_: &Context, _: &mut Connection, _: &Stanza| {
				*i.write().unwrap() += 1;
				HandlerResult::RemoveHandler
			}
		};
		assert_ne!(mem::size_of_val(&i_incrementer), 0);

		// zero sized handlers are called
		{
			let zero_sized = |_: &Context, conn: &mut Connection, _: &Stanza| {
				let pres = Stanza::new_presence();
				conn.send(&pres);
				HandlerResult::RemoveHandler
			};
			assert_eq!(mem::size_of_val(&zero_sized), 0);

			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn.handler_add(zero_sized, None, None, None).expect("Can't add handler");
							conn
								.handler_add(i_incrementer.clone(), None, Some("presence"), None)
								.expect("Can't add handler");
							conn
								.timed_handler_add(
									|_, conn| {
										conn.disconnect();
										HandlerResult::RemoveHandler
									},
									Duration::from_secs(1),
								)
								.expect("Can't add timed handler");
						}
						ConnectionEvent::Disconnect(_) => ctx.stop(),
						_ => (),
					}
				})
				.unwrap();
			ctx.run();
		}
		assert_eq!(*i.read().unwrap(), 1);

		// non zero sized handlers are called
		*i.write().unwrap() = 0;
		{
			assert_ne!(mem::size_of_val(&i_incrementer), 0);

			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn
								.handler_add(i_incrementer.clone(), None, Some("presence"), None)
								.expect("Can't add handler");
							let pres = Stanza::new_presence();
							conn.send(&pres);
							conn
								.timed_handler_add(
									|_, conn| {
										conn.disconnect();
										HandlerResult::RemoveHandler
									},
									Duration::from_secs(1),
								)
								.expect("Can't add timed handler");
						}
						ConnectionEvent::Disconnect(_) => ctx.stop(),
						_ => (),
					}
				})
				.unwrap();
			ctx.run();
		}
		assert_eq!(*i.read().unwrap(), 1);

		// handlers_clear clears zs and non-zs handlers
		*i.write().unwrap() = 0;
		{
			let zero_sized = |_: &Context, conn: &mut Connection, _: &Stanza| {
				let pres = Stanza::new_presence();
				conn.send(&pres);
				HandlerResult::RemoveHandler
			};
			assert_eq!(mem::size_of_val(&zero_sized), 0);

			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn.handler_add(zero_sized, None, None, None).expect("Can't add handler");
							if conn.handler_add(zero_sized, None, None, None).is_some() {
								panic!("Must not be able to add handler");
							}
							conn
								.handler_add(i_incrementer.clone(), None, None, None)
								.expect("Can't add handler");
							if conn.handler_add(i_incrementer.clone(), None, None, None).is_some() {
								panic!("Must not be able to add handler");
							}
							conn.handlers_clear();
							conn
								.handler_add(i_incrementer.clone(), None, Some("presence"), None)
								.expect("Can't add handler");
							conn
								.timed_handler_add(
									|_, conn| {
										conn.disconnect();
										HandlerResult::RemoveHandler
									},
									Duration::from_secs(1),
								)
								.expect("Can't add timed handler");
						}
						ConnectionEvent::Disconnect(_) => ctx.stop(),
						_ => (),
					}
				})
				.unwrap();
			ctx.run();
		}
		assert_eq!(*i.read().unwrap(), 0);
	}

	assert_eq!(
		Arc::try_unwrap(i)
			.expect("There are hanging references to Rc value")
			.into_inner()
			.unwrap(),
		0
	);
}

#[test]
fn connection_handler() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};

	let flags = Arc::new(RwLock::new((0, 0, 0)));
	{
		let conn = creds.make_conn();
		let ctx = conn
			.connect_client(None, None, {
				let flags = Arc::clone(&flags);
				move |ctx, conn, evt| match evt {
					ConnectionEvent::Connect => {
						flags.write().unwrap().0 += 1;
						conn.disconnect();
					}
					ConnectionEvent::RawConnect => {
						flags.write().unwrap().1 += 1;
					}
					ConnectionEvent::Disconnect(_) => {
						flags.write().unwrap().2 += 1;
						ctx.stop();
					}
				}
			})
			.unwrap();
		ctx.run();
	}
	assert_eq!(
		Arc::try_unwrap(flags)
			.expect("There are hanging references to Rc value")
			.into_inner()
			.unwrap(),
		(1, 0, 1)
	);
}

#[test]
fn timed_handler_creds() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};

	let i = Arc::new(RwLock::new(0));

	let do_common_stuff = |conn: &mut Connection| {
		conn
			.timed_handler_add(
				|_, conn| {
					conn.disconnect();
					HandlerResult::RemoveHandler
				},
				Duration::from_secs(1),
			)
			.expect("Can't add timed handler");
	};

	{
		let i_incrementer = {
			let i = i.clone();
			move |_: &Context, _: &mut Connection| {
				*i.write().unwrap() += 1;
				HandlerResult::KeepHandler
			}
		};

		// timed trigger, inside
		{
			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn
								.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1))
								.expect("Can't add timed handler");
							do_common_stuff(conn);
						}
						ConnectionEvent::Disconnect(_) => ctx.stop(),
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert!(*i.read().unwrap() > 0);
			assert!(*i.read().unwrap() < 1000);
		}

		// outside
		*i.write().unwrap() = 0;
		{
			let mut conn = creds.make_conn();
			conn
				.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1))
				.expect("Can't add timed handler");
			let ctx = conn
				.connect_client(None, None, {
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							do_common_stuff(conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert!(*i.read().unwrap() > 0);
			assert!(*i.read().unwrap() < 1000);
		}

		// delete
		*i.write().unwrap() = 0;
		{
			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							let handler = conn
								.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1))
								.expect("Can't add timed handler");
							conn.timed_handler_delete(handler);
							do_common_stuff(conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}

		// clear
		*i.write().unwrap() = 0;
		{
			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn
								.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1))
								.expect("Can't add timed handler");
							conn.timed_handlers_clear();
							do_common_stuff(conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}
	}
	assert_eq!(
		Arc::try_unwrap(i)
			.expect("There are hanging references to Rc value")
			.into_inner()
			.unwrap(),
		0
	);
}

/*#[test]
fn ctx_con_lifetime_issues() {
	let ctx = Context::new_with_default_logger();

	// if you exchange the next 2 lines then it works, probably not related to code lifetimes, but to Rust
	let make_conn = || { Connection::new(ctx.clone()) };

	let default_con_handler = |conn: &mut Connection, evt: ConnectionEvent, _: i32, _: Option<&StreamError>| {};

	let mut conn = creds.make_conn();
	conn.connect_client(None, None, &default_con_handler).unwrap();
}*/

#[test]
fn id_handler_creds() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};

	let i = Arc::new(RwLock::new(0));

	let do_common_stuff = |_: &Context, conn: &mut Connection| {
		let mut iq = Stanza::new_iq(Some("get"), Some("get_roster"));
		let mut query = Stanza::new();
		query.set_name("query").unwrap();
		query.set_ns("jabber:iq:roster").unwrap();
		iq.add_child(query).unwrap();
		conn.send(&iq);

		conn
			.timed_handler_add(
				|_, conn| {
					conn.disconnect();
					HandlerResult::RemoveHandler
				},
				Duration::from_secs(1),
			)
			.expect("Can't add id handler");
	};

	{
		let i_incrementer = {
			let i = i.clone();
			move |_: &Context, _: &mut Connection, _: &Stanza| {
				*i.write().unwrap() += 1;
				HandlerResult::KeepHandler
			}
		};

		// iq trigger, inside
		{
			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn
								.id_handler_add(i_incrementer.clone(), "get_roster")
								.expect("Can't add id handler");

							let mut iq = Stanza::new_iq(Some("get"), Some("get_roster1"));
							let mut query = Stanza::new();
							query.set_name("query").unwrap();
							query.set_ns("jabber:iq:roster").unwrap();
							iq.add_child(query).unwrap();
							conn.send(&iq);

							do_common_stuff(ctx, conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 1);
		}

		// outside
		*i.write().unwrap() = 0;
		{
			let mut conn = creds.make_conn();
			conn
				.id_handler_add(i_incrementer.clone(), "get_roster")
				.expect("Can't add id handler");
			let ctx = conn
				.connect_client(None, None, {
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							let mut iq = Stanza::new_iq(Some("get"), Some("get_roster1"));
							let mut query = Stanza::new();
							query.set_name("query").unwrap();
							query.set_ns("jabber:iq:roster").unwrap();
							iq.add_child(query).unwrap();
							conn.send(&iq);

							do_common_stuff(ctx, conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 1);
		}

		// delete
		*i.write().unwrap() = 0;
		{
			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							let handler = conn
								.id_handler_add(i_incrementer.clone(), "get_roster")
								.expect("Can't id timed handler");
							conn.id_handler_delete(handler);

							do_common_stuff(ctx, conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}

		// clear
		*i.write().unwrap() = 0;
		{
			let conn = creds.make_conn();
			let ctx = conn
				.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							conn
								.id_handler_add(i_incrementer.clone(), "get_roster")
								.expect("Can't add id handler");
							conn.id_handlers_clear();
							do_common_stuff(ctx, conn);
						}
						ConnectionEvent::Disconnect(_) => {
							ctx.stop();
						}
						_ => {}
					}
				})
				.unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}
	}
	assert_eq!(
		Arc::try_unwrap(i)
			.expect("There are hanging references to Rc value")
			.into_inner()
			.unwrap(),
		0
	);
}

#[test]
fn handler() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};

	let i = Arc::new(RwLock::new(0));

	let default_con_handler = |ctx: &Context, conn: &mut Connection, evt: ConnectionEvent| match evt {
		ConnectionEvent::Connect => {
			conn.disconnect();
		}
		ConnectionEvent::Disconnect(_) => {
			ctx.stop();
		}
		_ => {}
	};

	let i_incrementer = {
		let i = i.clone();
		move |_: &Context, _: &mut Connection, _: &Stanza| {
			*i.write().unwrap() += 1;
			HandlerResult::KeepHandler
		}
	};

	// handler call stanza name filter
	{
		let mut conn = creds.make_conn();
		conn
			.handler_add(i_incrementer.clone(), None, Some("iq"), None)
			.expect("Can't add handler");
		let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
		ctx.run();
		assert_eq!(*i.read().unwrap(), 1);
	}

	// handler call stanza name not existent
	*i.write().unwrap() = 0;
	{
		let mut conn = creds.make_conn();
		conn
			.handler_add(i_incrementer.clone(), None, Some("non-existent"), None)
			.expect("Can't add handler");
		let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
		ctx.run();
		assert_eq!(*i.read().unwrap(), 0);
	}

	// handler delete
	*i.write().unwrap() = 0;
	{
		let mut conn = creds.make_conn();
		let handler = conn
			.handler_add(i_incrementer.clone(), None, None, None)
			.expect("Can't add handler");
		conn.handler_delete(handler);
		let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
		ctx.run();
		assert_eq!(*i.read().unwrap(), 0);
	}

	// handler clear
	*i.write().unwrap() = 0;
	{
		let mut conn = creds.make_conn();
		conn
			.handler_add(i_incrementer.clone(), None, None, None)
			.expect("Can't add handler");
		conn.handlers_clear();
		let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
		ctx.run();
		assert_eq!(*i.read().unwrap(), 0);
	}

	// same handler twice
	*i.write().unwrap() = 0;
	{
		let mut conn = creds.make_conn();
		assert!(conn.handler_add(&i_incrementer, None, Some("iq"), None,).is_some());
		assert!(conn.handler_add(&i_incrementer, None, Some("iq"), None).is_none());
		let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
		ctx.run();
		assert_eq!(*i.read().unwrap(), 1);
	}

	// cloned handler twice, not sure if this behaviour is right
	*i.write().unwrap() = 0;
	{
		let mut conn = creds.make_conn();
		assert!(conn.handler_add(i_incrementer.clone(), None, Some("iq"), None,).is_some());
		assert!(conn.handler_add(i_incrementer.clone(), None, Some("iq"), None).is_none());
		let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
		ctx.run();
		assert_eq!(*i.read().unwrap(), 1);
	}
}

#[test]
fn stanza_global_context() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};

	let stz = Arc::new(Mutex::new(None));
	{
		let mut conn = creds.make_conn();
		conn
			.handler_add(
				{
					let stz = stz.clone();
					move |_, _, stanza| {
						*stz.lock().unwrap() = Some(stanza.clone());
						HandlerResult::RemoveHandler
					}
				},
				None,
				Some("iq"),
				None,
			)
			.expect("Can't add handler");
		let ctx = conn
			.connect_client(None, None, |ctx, conn, evt| match evt {
				ConnectionEvent::Connect => conn.disconnect(),
				ConnectionEvent::Disconnect(_) => ctx.stop(),
				_ => (),
			})
			.unwrap();
		ctx.run();
	}
	let stanza = Arc::try_unwrap(stz).unwrap().into_inner().unwrap().unwrap();
	// without forcing ALLOC_CONTEXT it will segfault
	assert_eq!(stanza.to_text().unwrap(), stanza.to_string());
}

#[test]
#[cfg(feature = "libstrophe-0_11_0")]
fn connection_handler_tls() {
	let creds = if let Some(creds) = Creds::acquire() {
		creds
	} else {
		eprintln!("Can't acquire creds, skipping test");
		return;
	};

	{
		// self-signed certificate reject
		let flags = Arc::new(RwLock::new((0, 0, 0, 0)));
		{
			let mut conn = creds.make_tls_conn();
			// this handler will be replaced by another one
			conn.set_certfail_handler({
				let flags = Arc::clone(&flags);
				move |_cert, _err| {
					flags.write().unwrap().0 += 2;
					CertFailResult::TerminateConnection
				}
			});
			conn.set_certfail_handler({
				let flags = Arc::clone(&flags);
				move |_cert, _err| {
					flags.write().unwrap().0 += 1;
					CertFailResult::TerminateConnection
				}
			});
			let ctx = conn
				.connect_client(None, None, {
					let flags = Arc::clone(&flags);
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							flags.write().unwrap().1 += 1;
							conn.disconnect();
						}
						ConnectionEvent::RawConnect => {
							flags.write().unwrap().2 += 1;
						}
						ConnectionEvent::Disconnect(_) => {
							flags.write().unwrap().3 += 1;
							ctx.stop();
						}
					}
				})
				.unwrap();
			ctx.run();
		}
		assert_eq!(
			Arc::try_unwrap(flags)
				.expect("There are hanging references to Rc value")
				.into_inner()
				.unwrap(),
			(1, 0, 0, 1)
		);
	}

	{
		// self-signed certificate accept
		let flags = Arc::new(RwLock::new((0, 0, 0, 0)));
		{
			let mut conn = creds.make_tls_conn();
			conn.set_certfail_handler({
				let flags = Arc::clone(&flags);
				move |_cert, _err| {
					flags.write().unwrap().0 += 1;
					CertFailResult::EstablishConnection
				}
			});
			let ctx = conn
				.connect_client(None, None, {
					let flags = Arc::clone(&flags);
					move |ctx, conn, evt| match evt {
						ConnectionEvent::Connect => {
							flags.write().unwrap().1 += 1;
							if conn.peer_cert().is_some() {
								flags.write().unwrap().1 += 1;
							}
							conn.disconnect();
						}
						ConnectionEvent::RawConnect => {
							flags.write().unwrap().2 += 1;
						}
						ConnectionEvent::Disconnect(_) => {
							flags.write().unwrap().3 += 1;
							ctx.stop();
						}
					}
				})
				.unwrap();
			ctx.run();
		}
		let call_counts = Arc::try_unwrap(flags)
			.expect("There are hanging references to Rc value")
			.into_inner()
			.unwrap();
		// gives different results in CI and locally, probably difference in ejabberd versions
		assert!((2, 2, 0, 1) == call_counts || (3, 2, 0, 1) == call_counts);
	}
}

#[test]
fn fail() {
	let t = trybuild::TestCases::new();
	t.compile_fail("src/tests/fail/*.rs");
}

#[derive(Debug)]
struct Creds {
	jid: String,
	name: String,
	pass: String,
}

impl Creds {
	pub fn acquire() -> Option<Self> {
		let mut generator = Generator::default();
		let name = generator.next()?;
		let jid = format!("{name}@localhost");
		let pass = generator.next()?;
		let creds_acquire_script = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR")?).join("creds-acquire.sh");
		let result = Command::new(creds_acquire_script).arg(&name).arg(&pass).output().unwrap();
		if !result.status.success() {
			return None;
		}
		Some(Self { jid, name, pass })
	}

	pub fn make_conn(&self) -> Connection<'_, 'static> {
		let mut conn = Connection::new(Context::new_with_default_logger());
		conn.set_jid(&self.jid);
		conn.set_pass(&self.pass);
		conn
			.set_flags(ConnectionFlags::TRUST_TLS)
			.expect("Cannot set connection flags");
		conn
	}

	#[cfg(feature = "libstrophe-0_11_0")]
	pub fn make_tls_conn(&self) -> Connection<'_, 'static> {
		let mut conn = Connection::new(Context::new_with_default_logger());
		conn.set_jid(&self.jid);
		conn.set_pass(&self.pass);
		conn
			.set_flags(ConnectionFlags::MANDATORY_TLS)
			.expect("Cannot set connection flags");
		conn
	}
}

impl Drop for Creds {
	fn drop(&mut self) {
		if let Some(cargo_manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
			let creds_release_script = PathBuf::from(cargo_manifest_dir).join("creds-release.sh");
			if let Err(e) = Command::new(creds_release_script).arg(&self.name).output() {
				eprintln!("Failure releasing credentials: {e}");
			}
		}
	}
}
