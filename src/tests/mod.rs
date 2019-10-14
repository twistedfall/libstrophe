use std::{
	collections::HashMap,
	time::Duration,
};

use matches::assert_matches;

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
	let mut i = 0;
	{
		let ctx = Context::new(Logger::new(|_, _, _| {
			i += 1;
		}));
		let mut conn = Connection::new(ctx);
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_client(None, None, |_, _, _, _, _| {}).unwrap();
		ctx.run_once(Duration::from_secs(1));
	}
	assert_eq!(i, 5);
}

#[test]
fn conn_client_wo_jid() {
	let conn = Connection::new(Context::new_with_null_logger());
	// no JID supplied
	assert_matches!(conn.connect_client(None, None, |_, _, _, _, _| {}), Err(ConnectError { error: Error::InvalidOperation, .. }));
}

#[test]
fn conn_client() {
	let conn_handler = |ctx: &Context,
	                    _: &mut Connection,
	                    event: ConnectionEvent,
	                    _: i32,
	                    _: Option<&error::StreamError>, | {
		assert_eq!(event, ConnectionEvent::XMPP_CONN_DISCONNECT);
		ctx.stop();
	};

	// ref closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_client(None, None, &conn_handler).unwrap();
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
	let conn_handler = |ctx: &Context,
	                    _: &mut Connection,
	                    event: ConnectionEvent,
	                    _: i32,
	                    _: Option<&error::StreamError>, | {
		assert_eq!(event, ConnectionEvent::XMPP_CONN_DISCONNECT);
		ctx.stop();
	};

	// ref closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_raw(None, None, &conn_handler).unwrap();
		ctx.run();
	}

	// own closure
	{
		let mut conn = Connection::new(Context::new_with_null_logger());
		conn.set_jid("test-JID@127.50.60.70");
		let ctx = conn.connect_raw(None, None, conn_handler).unwrap();
		ctx.run();
	}
}

#[test]
fn timed_handler() {
	let timed_handler = |_: &Context, _: &mut Connection| { false };
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let handle = conn.timed_handler_add(&timed_handler, Duration::from_secs(1)).unwrap();
	assert_matches!(conn.timed_handler_add(&timed_handler, Duration::from_secs(1)), None);
	conn.timed_handler_delete(handle);
}

#[test]
fn stanza_handler() {
	let stanza_handler = |_: &Context, _: &mut Connection, _: &Stanza| { false };
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let handle = conn.handler_add(&stanza_handler, Some("ns"), None, None).unwrap();
	assert_matches!(conn.handler_add(&stanza_handler, Some("ns"), None, None), None);
	conn.handler_delete(handle);
	let handle = conn.handler_add(stanza_handler, None, Some(&"name".to_owned()), None).unwrap();
	conn.handler_delete(handle);
}

#[test]
fn id_handler() {
	let id_handler = |_: &Context, _: &mut Connection, _: &Stanza| { false };
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let h = conn.id_handler_add(&id_handler, "test").unwrap();
	assert_matches!(conn.id_handler_add(&id_handler, "test"), None);
	conn.id_handler_delete(h);
}

#[test]
fn stanza_handler_in_con() {
	let stanza_handler = |_: &Context, _: &mut Connection, _: &Stanza| { false };
	let con_handler = move |_: &Context,
	                        conn: &mut Connection,
	                        _: ConnectionEvent,
	                        _: i32,
	                        _: Option<&error::StreamError>, | {
		conn.handler_add(stanza_handler, None, None, None);
	};
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	conn.set_jid("test-JID@127.50.60.70");
	conn.connect_client(None, None, &con_handler).unwrap();
}

#[test]
fn jid_test() {
	assert_eq!(Some("node@domain.com/test".to_string()), jid::jid_new(Some("node"), "domain.com", Some("test")));
	assert_eq!(Some("domain.com/test".to_string()), jid::jid_new(None, "domain.com", Some("test")));
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
				_ => panic!("Too many items: {}", child)
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
				_ => panic!("Too many items: {}", child)
			}
		}
		assert_eq!(stanza.get_first_child().unwrap().name().unwrap(), "presence1");
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

#[cfg(feature = "creds-test")]
mod with_credentials {
	use std::{
		mem,
		sync::{Arc, RwLock},
	};

	use super::*;

	// testing is done on vanilla local ejabberd-17.04
	const JID: &str = include_str!("../../jid.txt");
	const PASS: &str = include_str!("../../password.txt");

	fn make_conn<'cn>() -> Connection<'cn, 'static> {
		let mut conn = Connection::new(Context::new_with_default_logger());
		conn.set_jid(JID);
		conn.set_pass(PASS);
		conn
	}

	#[test]
	fn zero_sized_handlers() {
		let i = Arc::new(RwLock::new(0));

		{
			let i_incrementer = {
				let i = i.clone();
				move |_: &Context, _: &mut Connection, _: &Stanza| {
					*i.write().unwrap() += 1;
					false
				}
			};

			// zero sized handlers are called
			{
				let zero_sized = |_: &Context, conn: &mut Connection, _: &Stanza| {
					let pres = Stanza::new_presence();
					conn.send(&pres);
					false
				};
				assert_eq!(mem::size_of_val(&zero_sized), 0);

				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.handler_add(zero_sized, None, None, None);
								conn.handler_add(i_incrementer.clone(), None, Some("presence"), None);
								conn.timed_handler_add(|_, conn| {
									conn.disconnect();
									false
								}, Duration::from_secs(1));
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => ctx.stop(),
							_ => ()
						}
					}
				}).unwrap();
				ctx.run();
			}
			assert_eq!(*i.read().unwrap(), 1);

			// non zero sized handlers are called
			*i.write().unwrap() = 0;
			{
				assert_ne!(mem::size_of_val(&i_incrementer), 0);

				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.handler_add(i_incrementer.clone(), None, Some("presence"), None);
								let pres = Stanza::new_presence();
								conn.send(&pres);
								conn.timed_handler_add(|_, conn| {
									conn.disconnect();
									false
								}, Duration::from_secs(1));
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => ctx.stop(),
							_ => ()
						}
					}
				}).unwrap();
				ctx.run();
			}
			assert_eq!(*i.read().unwrap(), 1);

			// handlers_clear clears zs and non-zs handlers
			*i.write().unwrap() = 0;
			{
				let zero_sized = |_: &Context, conn: &mut Connection, _: &Stanza| {
					let pres = Stanza::new_presence();
					conn.send(&pres);
					false
				};
				assert_eq!(mem::size_of_val(&zero_sized), 0);

				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.handler_add(zero_sized.clone(), None, None, None);
								conn.handler_add(zero_sized.clone(), None, None, None);
								conn.handler_add(i_incrementer.clone(), None, None, None);
								conn.handler_add(i_incrementer.clone(), None, None, None);
								conn.handlers_clear();
								conn.handler_add(i_incrementer.clone(), None, Some("presence"), None);
								conn.timed_handler_add(|_, conn| {
									conn.disconnect();
									false
								}, Duration::from_secs(1));
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => ctx.stop(),
							_ => ()
						}
					}
				}).unwrap();
				ctx.run();
			}
			assert_eq!(*i.read().unwrap(), 0);
		}

		assert_eq!(Arc::try_unwrap(i).expect("There are hanging references to Rc value").into_inner().unwrap(), 0);
	}

	#[test]
	fn connection_handler() {
		let flags = Arc::new(RwLock::new((0, 0, 0, 0)));
		let ctx = Context::new_with_default_logger();
		{
			let mut conn = Connection::new(ctx);
			conn.set_jid(JID);
			conn.set_pass(PASS);
			let ctx = conn.connect_client(None, None, {
				let flags = flags.clone();
				move |ctx, conn, evt, _, _| {
					match evt {
						ConnectionEvent::XMPP_CONN_CONNECT => {
							flags.write().unwrap().0 += 1;
							conn.disconnect();
						}
						ConnectionEvent::XMPP_CONN_RAW_CONNECT => {
							flags.write().unwrap().1 += 1;
						}
						ConnectionEvent::XMPP_CONN_DISCONNECT => {
							flags.write().unwrap().2 += 1;
							ctx.stop();
						}
						ConnectionEvent::XMPP_CONN_FAIL => {
							flags.write().unwrap().3 += 1;
						}
					}
				}
			}).unwrap();

			// can't connect_client twice until disconnect
//			let res = conn.connect_client(None, None, |_, _, _, _| {});
//			assert_matches!(res.map_err(|e| e.downcast().unwrap()), Err(error::Error::InvalidOperation));

			ctx.run();

			// can connect again after disconnect
//			let ctx = conn.connect_client(None, None, |conn, evt, _, _| {
//				match evt {
//					ConnectionEvent::XMPP_CONN_CONNECT => conn.disconnect(),
//					ConnectionEvent::XMPP_CONN_DISCONNECT => conn.context().stop(),
//					_ => {}
//				}
//			}).unwrap();
//
//			ctx.run();
		}
		assert_eq!(Arc::try_unwrap(flags).expect("There are hanging references to Rc value").into_inner().unwrap(), (1, 0, 1, 0));
	}

	#[test]
	fn timed_handler() {
		let i = Arc::new(RwLock::new(0));

		let do_common_stuff = |conn: &mut Connection| {
			conn.timed_handler_add(|_, conn| {
				conn.disconnect();
				false
			}, Duration::from_secs(1));
		};

		{
			let i_incrementer = {
				let i = i.clone();
				move |_: &Context, _: &mut Connection| {
					*i.write().unwrap() += 1;
					true
				}
			};

			// timed trigger, inside
			{
				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1));
								do_common_stuff(conn);
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => ctx.stop(),
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert!(*i.read().unwrap() > 0);
				assert!(*i.read().unwrap() < 1000);
			}

			// outside
			*i.write().unwrap() = 0;
			{
				let mut conn = make_conn();
				conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1));
				let ctx = conn.connect_client(None, None, {
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert!(*i.read().unwrap() > 0);
				assert!(*i.read().unwrap() < 1000);
			}

			// delete
			*i.write().unwrap() = 0;
			{
				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								let handler = conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1)).unwrap();
								conn.timed_handler_delete(handler);
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert_eq!(*i.read().unwrap(), 0);
			}

			// clear
			*i.write().unwrap() = 0;
			{
				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1));
								conn.timed_handlers_clear();
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert_eq!(*i.read().unwrap(), 0);
			}
		}
		assert_eq!(Arc::try_unwrap(i).expect("There are hanging references to Rc value").into_inner().unwrap(), 0);
	}

	/*#[test]
	fn ctx_con_lifetime_issues() {
		let ctx = Context::new_with_default_logger();

		// if you exchange the next 2 lines then it works, probably not related to code lifetimes, but to Rust
		let make_conn = || { Connection::new(ctx.clone()) };

		let default_con_handler = |conn: &mut Connection, evt: ConnectionEvent, _: i32, _: Option<&error::StreamError>| {};

		let mut conn = make_conn();
		conn.connect_client(None, None, &default_con_handler).unwrap();
	}*/


	#[test]
	fn id_handler() {
		let i = Arc::new(RwLock::new(0));

		let do_common_stuff = |_: &Context, conn: &mut Connection| {
			let mut iq = Stanza::new_iq(Some("get"), Some("get_roster"));
			let mut query = Stanza::new();
			query.set_name("query").unwrap();
			query.set_ns("jabber:iq:roster").unwrap();
			iq.add_child(query).unwrap();
			conn.send(&iq);

			conn.timed_handler_add(|_, conn| {
				conn.disconnect();
				false
			}, Duration::from_secs(1));
		};

		{
			let i_incrementer = {
				let i = i.clone();
				move |_: &Context, _: &mut Connection, _: &Stanza| {
					*i.write().unwrap() += 1;
					true
				}
			};

			// iq trigger, inside
			{
				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.id_handler_add(i_incrementer.clone(), "get_roster");

								let mut iq = Stanza::new_iq(Some("get"), Some("get_roster1"));
								let mut query = Stanza::new();
								query.set_name("query").unwrap();
								query.set_ns("jabber:iq:roster").unwrap();
								iq.add_child(query).unwrap();
								conn.send(&iq);

								do_common_stuff(ctx, conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert_eq!(*i.read().unwrap(), 1);
			}

			// outside
			*i.write().unwrap() = 0;
			{
				let mut conn = make_conn();
				conn.id_handler_add(i_incrementer.clone(), "get_roster");
				let ctx = conn.connect_client(None, None, {
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								let mut iq = Stanza::new_iq(Some("get"), Some("get_roster1"));
								let mut query = Stanza::new();
								query.set_name("query").unwrap();
								query.set_ns("jabber:iq:roster").unwrap();
								iq.add_child(query).unwrap();
								conn.send(&iq);

								do_common_stuff(ctx, conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert_eq!(*i.read().unwrap(), 1);
			}

			// delete
			*i.write().unwrap() = 0;
			{
				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								let handler = conn.id_handler_add(i_incrementer.clone(), "get_roster").unwrap();
								conn.id_handler_delete(handler);

								do_common_stuff(ctx, conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert_eq!(*i.read().unwrap(), 0);
			}

			// clear
			*i.write().unwrap() = 0;
			{
				let conn = make_conn();
				let ctx = conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |ctx, conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.id_handler_add(i_incrementer.clone(), "get_roster").unwrap();
								conn.id_handlers_clear();
								do_common_stuff(ctx, conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								ctx.stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				ctx.run();
				assert_eq!(*i.read().unwrap(), 0);
			}
		}
		assert_eq!(Arc::try_unwrap(i).expect("There are hanging references to Rc value").into_inner().unwrap(), 0);
	}

	#[test]
	fn handler() {
		let i = Arc::new(RwLock::new(0));

		let default_con_handler = |ctx: &Context, conn: &mut Connection, evt: ConnectionEvent, _: i32, _: Option<&error::StreamError>| {
			match evt {
				ConnectionEvent::XMPP_CONN_CONNECT => {
					conn.disconnect();
				},
				ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
					ctx.stop();
				},
				_ => {},
			}
		};

		let i_incrementer = {
			let i = i.clone();
			move |_: &Context, _: &mut Connection, _: &Stanza| {
				*i.write().unwrap() += 1;
				true
			}
		};

		// handler call stanza name filter
		{
			let mut conn = make_conn();
			conn.handler_add(i_incrementer.clone(), None, Some("iq"), None);
			let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 1);
		}

		// handler call stanza name not existent
		*i.write().unwrap() = 0;
		{
			let mut conn = make_conn();
			conn.handler_add(i_incrementer.clone(), None, Some("non-existent"), None);
			let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}

		// handler delete
		*i.write().unwrap() = 0;
		{
			let mut conn = make_conn();
			let handler = conn.handler_add(i_incrementer.clone(), None, None, None).unwrap();
			conn.handler_delete(handler);
			let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}

		// handler clear
		*i.write().unwrap() = 0;
		{
			let mut conn = make_conn();
			conn.handler_add(i_incrementer.clone(), None, None, None);
			conn.handlers_clear();
			let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 0);
		}

		// same handler twice
		*i.write().unwrap() = 0;
		{
			let mut conn = make_conn();
			assert_matches!(conn.handler_add(&i_incrementer, None, Some("iq"), None,), Some(..));
			assert_matches!(conn.handler_add(&i_incrementer, None, Some("iq"), None), None);
			let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 1);
		}

		// cloned handler twice, not sure if this behaviour is right
		*i.write().unwrap() = 0;
		{
			let mut conn = make_conn();
			assert_matches!(conn.handler_add(i_incrementer.clone(), None, Some("iq"), None,), Some(..));
			assert_matches!(conn.handler_add(i_incrementer.clone(), None, Some("iq"), None), None);
			let ctx = conn.connect_client(None, None, default_con_handler).unwrap();
			ctx.run();
			assert_eq!(*i.read().unwrap(), 1);
		}
	}
}

// All of those and only those functions must fail compilation due to lifetime problems
#[cfg(feature = "fail-test")]
mod fail {
	use crate::LogLevel;

	use super::*;

	#[test]
	fn conn_handler_too_short() {
		{
			let conn = Connection::new(Context::new_with_null_logger());
			let not_long_enough1 = |_: &Context, _: &mut Connection, _: ConnectionEvent, _: i32, _: Option<&error::StreamError>| {};
			conn.connect_client(None, None, &not_long_enough1).unwrap()
		};
	}

	#[test]
	fn stanza_handler_too_short() {
		let ctx = Context::new_with_null_logger();
		let mut conn = Connection::new(ctx);
		{
			let not_long_enough2 = |_: &Context, _: &mut Connection, _: &Stanza| { false };
			conn.handler_add(&not_long_enough2, None, None, None);
			conn
		};
	}

	#[test]
	fn timed_handler_too_short() {
		let mut conn = Connection::new(Context::new_with_null_logger());
		{
			let not_long_enough3 = |_: &Context, _: &mut Connection| { false };
			conn.timed_handler_add(&not_long_enough3, Duration::from_secs(1));
			conn
		};
	}

	#[test]
	fn id_handler_too_short() {
		let mut conn = Connection::new(Context::new_with_null_logger());
		{
			let not_long_enough4 = |_: &Context, _: &mut Connection, _: &Stanza| { false };
			conn.id_handler_add(&not_long_enough4, "id");
			conn
		};
	}

	#[test]
	fn stanza_too_short() {
		let _a = {
			let not_long_enough5 = Stanza::new();
			not_long_enough5.get_first_child()
		};
	}

	#[test]
	fn logger_too_short() {
		{
			let not_long_enough6 = |_: LogLevel, _: &str, _: &str| {};
			let logger = Logger::new(&not_long_enough6);
			Context::new(logger)
		};
	}

	#[test]
	fn add_short_lived_handler() {
		let handler = |_: &Context, conn: &mut Connection, _, _, _: Option<&error::StreamError>| {
			let not_long_enough7 = |_: &Context, _: &mut Connection, _: &Stanza| { false };
			conn.handler_add(&not_long_enough7, None, None, None);
		};
		let conn = Connection::new(Context::new_with_null_logger());
		conn.connect_client(None, None, &handler);
	}

	#[test]
	fn logger_closure_capture_too_short() {
		{
			let mut not_long_enough8 = 0;
			Context::new(Logger::new(|_, _, _| {
				not_long_enough8 = 1;
			}))
		};
	}

	#[test]
	fn conn_handler_closure_capture_too_short() {
		{
			let mut not_long_enough9 = 0;
			let conn = Connection::new(Context::new_with_null_logger());
			conn.connect_client(None, None, |_, _, _, _, _| {
				not_long_enough9 = 1;
			}).unwrap()
		};
	}

	#[test]
	fn stanza_handler_closure_capture_too_short() {
		{
			let mut not_long_enough10 = 0;
			let mut conn = Connection::new(Context::new_with_null_logger());
			conn.handler_add(|_, _, _| {
				not_long_enough10 = 1;
				false
			}, None, None, None);
			conn
		};
	}

	#[test]
	fn timed_handler_closure_capture_too_short() {
		{
			let mut not_long_enough11 = 0;
			let mut conn = Connection::new(Context::new_with_null_logger());
			conn.timed_handler_add(|_, _| {
				not_long_enough11 = 1;
				false
			}, Duration::from_secs(1));
			conn
		};
	}

	#[test]
	fn id_handler_closure_capture_too_short() {
		{
			let mut not_long_enough12 = 0;
			let mut conn = Connection::new(Context::new_with_null_logger());
			conn.id_handler_add(|_, _, _| {
				not_long_enough12 = 1;
				false
			}, "");
			conn
		};
	}

	#[test]
	fn stanza_iterator_borrow() {
		let mut already_borrowed13 = Stanza::new();
		let _a = already_borrowed13.children();
		already_borrowed13.get_first_child_mut();
	}

	#[test]
	fn stanza_iterator_mut_borrow() {
		let mut already_borrowed14 = Stanza::new();
		let _a = already_borrowed14.children_mut();
		already_borrowed14.get_first_child();
	}
}
