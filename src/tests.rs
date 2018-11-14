extern crate env_logger;
extern crate matches;

use std::{mem, rc::Rc, sync::Arc, time::Duration};
use super::*;

#[test]
fn examples() {
//	super::examples::bot_fn_safe::main();
//	super::examples::bot_closure_unsafe::main();
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
		let ctx = Arc::new(Context::new(Logger::new(|_, _, _| {
			i = i + 1;
		})));
		let mut conn = Connection::new(ctx.clone());
		conn.set_jid("test-JID@127.50.60.70");
		conn.connect_client(None, None, |_, _, _, _| {}).unwrap();
		ctx.run_once(Duration::from_secs(1));
	}
	assert_eq!(i, 5);
}

#[test]
fn conn_client_wo_jid() {
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx.clone());
	// no JID supplied
	assert_matches!(conn.connect_client(None, None, |_, _, _, _| {}), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
}

#[test]
fn conn_client() {
	let conn_handler = |conn: &mut Connection,
	                    event: ConnectionEvent,
	                    _error: i32,
	                    _stream_error: Option<&error::StreamError>, | {
		assert_eq!(event, ConnectionEvent::XMPP_CONN_DISCONNECT);
		conn.context().stop();
	};

	let ctx = Context::new_with_null_logger();
	// ref closure
	{
		let mut conn = Connection::new(ctx.clone());
		conn.set_jid("test-JID@127.50.60.70");
		conn.connect_client(None, None, &conn_handler).unwrap();
		ctx.run();
	}

	// own closure
	{
		let mut conn = Connection::new(ctx.clone());
		conn.set_jid("test-JID@127.50.60.70");
		conn.connect_client(None, None, conn_handler).unwrap();
		ctx.run();
	}
}

#[test]
fn conn_raw() {
	let conn_handler = |conn: &mut Connection,
	                    event: ConnectionEvent,
	                    _error: i32,
	                    _stream_error: Option<&error::StreamError>, | {
		assert_eq!(event, ConnectionEvent::XMPP_CONN_DISCONNECT);
		conn.context().stop();
	};

	let ctx = Context::new_with_null_logger();
	// ref closure
	{
		let mut conn = Connection::new(ctx.clone());
		conn.set_jid("test-JID@127.50.60.70");
		conn.connect_raw(None, None, &conn_handler).unwrap();
		ctx.run();
	}

	// own closure
	{
		let mut conn = Connection::new(ctx.clone());
		conn.set_jid("test-JID@127.50.60.70");
		conn.connect_raw(None, None, conn_handler).unwrap();
		ctx.run();
	}
}

#[test]
fn timed_handler() {
	let timed_handler = |_conn: &mut Connection| { false };
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let handle = conn.timed_handler_add(&timed_handler, Duration::from_secs(1)).unwrap();
	assert_matches!(conn.timed_handler_add(&timed_handler, Duration::from_secs(1)), None);
	conn.timed_handler_delete(handle);
}

#[test]
fn stanza_handler() {
	let stanza_handler = |_conn: &mut Connection,
	                      _stanza: &Stanza| { false };
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
	let id_handler = |_conn: &mut Connection,
	                  _stanza: &Stanza| { false };
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	let h = conn.id_handler_add(&id_handler, "test").unwrap();
	assert_matches!(conn.id_handler_add(&id_handler, "test"), None);
	conn.id_handler_delete(h);
}

#[test]
fn stanza_handler_in_con() {
	let stanza_handler = |_con: &mut Connection,
	                      _stanza: &Stanza| { false };
	let con_handler = move |conn: &mut Connection,
	                         _event: ConnectionEvent,
	                         _error: i32,
	                         _stream_error: Option<&error::StreamError>, | {
		conn.handler_add(stanza_handler, None, None, None);
	};
	let ctx = Context::new_with_null_logger();
	let mut conn = Connection::new(ctx);
	conn.set_jid("test-JID@127.50.60.70");
	conn.connect_client(None, None, &con_handler).unwrap();
}

#[test]
fn jid() {
	let ctx = Context::new_with_null_logger();
	assert_eq!(Some("node@domain.com/test".to_string()), ctx.jid_new(Some("node"), "domain.com", Some("test")));
	assert_eq!(Some("domain.com/test".to_string()), ctx.jid_new(None, "domain.com", Some("test")));
	assert_eq!(Some("domain.com".to_string()), ctx.jid_new(None, "domain.com", None));

	let jid = ctx.jid_new(Some("node"), "domain.com", Some("test")).unwrap();
	let jid_domain = ctx.jid_new(None, "domain.com", None).unwrap();
	assert_eq!(Some("node@domain.com".to_string()), ctx.jid_bare(&jid));

	assert_eq!(Some("node".to_string()), ctx.jid_node(&jid));
	assert_eq!(None, ctx.jid_node(&jid_domain));

	assert_eq!(Some("domain.com".to_string()), ctx.jid_domain(&jid));
	assert_eq!(Some("domain.com".to_string()), ctx.jid_domain(&jid_domain));

	assert_eq!(Some("test".to_string()), ctx.jid_resource(&jid));
	assert_eq!(None, ctx.jid_resource(&jid_domain));
}

#[test]
fn context_eq_test() {
	let ctx = Context::new_with_null_logger();

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
	let ctx = Context::new_with_null_logger();

	let mut stanza = Stanza::new(&ctx);
	assert_matches!(stanza.to_text(), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
	stanza.set_name("test").unwrap();
	assert_matches!(stanza.set_body("body"), Err(error::Error(error::ErrorKind::InvalidOperation, ..)));
}

#[test]
fn stanza_display() {
	let ctx = Context::new_with_null_logger();

	let mut stanza = Stanza::new(&ctx);
	stanza.set_name("message").unwrap();
	stanza.set_id("stanza_id").unwrap();
	stanza.add_child(Stanza::new_iq(&ctx, Some("test"), None)).unwrap();
	assert_eq!(stanza.to_string(), stanza.to_text().unwrap());
}

#[test]
fn stanza_hier() {
	let ctx = Context::new_with_null_logger();

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
	let ctx = Context::new_with_null_logger();

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

#[cfg(feature = "creds-test")]
mod with_credentials {
	use std::{cell::{Cell, RefCell}};
	use super::*;

	// testing is done on vanilla local ejabberd-17.04
	const JID: &'static str = "";
	const PASS: &'static str = "";

	fn make_conn<'c>() -> Connection<'c> {
		let mut conn = Connection::new(Context::new_with_default_logger());
		conn.set_jid(JID);
		conn.set_pass(PASS);
		conn
	}

	#[test]
	fn zero_sized_handlers() {
		let i = Rc::new(Cell::new(0));

		{
			let i_incrementer = {
				let i = i.clone();
				move |_: &mut Connection, _: &Stanza| {
					i.set(i.get() + 1);
					false
				}
			};

			// zero sized handlers are called
			{
				let zero_sized = |conn: &mut Connection, _: &Stanza| {
					let ctx = conn.context();
					let pres = Stanza::new_presence(&ctx);
					conn.send(&pres);
					return false;
				};
				assert_eq!(mem::size_of_val(&zero_sized), 0);

				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.handler_add(zero_sized, None, None, None);
								conn.handler_add(i_incrementer.clone(), None, Some("presence"), None);
								conn.timed_handler_add(|conn| {
									conn.disconnect();
									false
								}, Duration::from_secs(1));
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => conn.context().stop(),
							_ => ()
						}
					}
				}).unwrap();
				conn.context().run();
			}
			assert_eq!(i.get(), 1);

			// non zero sized handlers are called
			i.set(0);
			{
				assert_ne!(mem::size_of_val(&i_incrementer), 0);

				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.handler_add(i_incrementer.clone(), None, Some("presence"), None);
								let ctx = conn.context();
								let pres = Stanza::new_presence(&ctx);
								conn.send(&pres);
								conn.timed_handler_add(|conn| {
									conn.disconnect();
									false
								}, Duration::from_secs(1));
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => conn.context().stop(),
							_ => ()
						}
					}
				}).unwrap();
				conn.context().run();
			}
			assert_eq!(i.get(), 1);

			// handlers_clear clears zs and non-zs handlers
			i.set(0);
			{
				let zero_sized = |conn: &mut Connection, _: &Stanza| {
					let ctx = conn.context();
					let pres = Stanza::new_presence(&ctx);
					conn.send(&pres);
					return false;
				};
				assert_eq!(mem::size_of_val(&zero_sized), 0);

				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.handler_add(zero_sized.clone(), None, None, None);
								conn.handler_add(zero_sized.clone(), None, None, None);
								conn.handler_add(i_incrementer.clone(), None, None, None);
								conn.handler_add(i_incrementer.clone(), None, None, None);
								conn.handlers_clear();
								conn.handler_add(i_incrementer.clone(), None, Some("presence"), None);
								conn.timed_handler_add(|conn| {
									conn.disconnect();
									false
								}, Duration::from_secs(1));
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => conn.context().stop(),
							_ => ()
						}
					}
				}).unwrap();
				conn.context().run();
			}
			assert_eq!(i.get(), 0);
		}

		assert_eq!(Rc::try_unwrap(i).expect("There are hanging references to Rc value").into_inner(), 0);
	}

	#[test]
	fn connection_handler() {
		let flags = Rc::new(RefCell::new((0, 0, 0, 0)));
		let ctx = Context::new_with_default_logger();
		{
			let mut conn = Connection::new(ctx);
			conn.set_jid(JID);
			conn.set_pass(PASS);
			conn.connect_client(None, None, {
				let flags = flags.clone();
				move |conn, evt, _, _| {
					match evt {
						ConnectionEvent::XMPP_CONN_CONNECT => {
							flags.borrow_mut().0 += 1;
							conn.disconnect();
						}
						ConnectionEvent::XMPP_CONN_RAW_CONNECT => {
							flags.borrow_mut().1 += 1;
						}
						ConnectionEvent::XMPP_CONN_DISCONNECT => {
							flags.borrow_mut().2 += 1;
							conn.context().stop();
						}
						ConnectionEvent::XMPP_CONN_FAIL => {
							flags.borrow_mut().3 += 1;
						}
					}
				}
			}).unwrap();

			// can't connect_client twice until disconnect
			let res = conn.connect_client(None, None, |_, _, _, _| {});
			assert_matches!(res, Err(error::Error(error::ErrorKind::InvalidOperation, ..)));

			conn.context().run();

			// can connect again after disconnect
			conn.connect_client(None, None, |conn, evt, _, _| {
				match evt {
					ConnectionEvent::XMPP_CONN_CONNECT => conn.disconnect(),
					ConnectionEvent::XMPP_CONN_DISCONNECT => conn.context().stop(),
					_ => {}
				}
			}).unwrap();

			conn.context().run();
		}
		assert_eq!(Rc::try_unwrap(flags).unwrap().into_inner(), (1, 0, 1, 0));
	}

	#[test]
	fn must_fail() {
		#![allow(unreachable_code)]
		return;
		use std::thread;
		let internal = || {
			let ctx = Context::new_with_default_logger();
			let mut conn = Connection::new(ctx);
			let h = conn.handler_add(|_: &mut Connection, _: &Stanza| false, None, None, None);
			h
		};
		let _h = internal();
		env_logger::init();

		// e.g. it's possible for Connection to consume context on ::new() and release it on ::connect_client()
		let ctx = Context::new_with_default_logger();
		let mut conn = Connection::new(ctx.clone());
		conn.set_jid(JID);
		conn.set_pass(PASS);
		conn.connect_client(None, None, |con, evt, _, _| {
			match evt {
				ConnectionEvent::XMPP_CONN_CONNECT => {
					con.timed_handler_add(|c| {
						eprintln!("!!!!!!!!!!!!!!!!");
						let ctx = c.context();
						let mut pres = Stanza::new(&ctx);
						pres.set_name("message").unwrap();
						pres.set_stanza_type("normal").unwrap();
						c.send(&pres);
						true
					}, Duration::from_millis(100));
				}
				ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => con.context().stop(),
				_ => {},
			}

		}).unwrap();
		thread::spawn({
			move || {
				let _a = conn;
				thread::sleep(Duration::from_secs(120));
			}
		});
		ctx.run();
	}

	#[test]
	fn timed_handler() {
		let i = Rc::new(Cell::new(0));

		let do_common_stuff = |conn: &mut Connection| {
			conn.timed_handler_add(|conn| {
				conn.disconnect();
				false
			}, Duration::from_secs(1));
		};

		{
			let i_incrementer = {
				let i = i.clone();
				move |_: &mut Connection| {
					i.set(i.get() + 1);
					true
				}
			};

			// timed trigger, inside
			{
				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1));
								do_common_stuff(conn);
							}
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => conn.context().stop(),
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert!(i.get() > 0);
				assert!(i.get() < 1000);
			}

			// outside
			i.set(0);
			{
				let mut conn = make_conn();
				conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1));
				conn.connect_client(None, None, {
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert!(i.get() > 0);
				assert!(i.get() < 1000);
			}

			// delete
			i.set(0);
			{
				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								let handler = conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1)).unwrap();
								conn.timed_handler_delete(handler);
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert_eq!(i.get(), 0);
			}

			// clear
			i.set(0);
			{
				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.timed_handler_add(i_incrementer.clone(), Duration::from_millis(1));
								conn.timed_handlers_clear();
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert_eq!(i.get(), 0);
			}
		}
		assert_eq!(Rc::try_unwrap(i).expect("There are hanging references to Rc value").into_inner(), 0);
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
		let i = Rc::new(Cell::new(0));

		let do_common_stuff = |conn: &mut Connection| {
			let ctx = conn.context();
			let mut iq = Stanza::new_iq(&ctx, Some("get"), Some("get_roster"));
			let mut query = Stanza::new(&ctx);
			query.set_name("query").unwrap();
			query.set_ns("jabber:iq:roster").unwrap();
			iq.add_child(query).unwrap();
			conn.send(&iq);

			conn.timed_handler_add(|conn| {
				conn.disconnect();
				false
			}, Duration::from_secs(1));
		};

		{
			let i_incrementer = {
				let i = i.clone();
				move |_: &mut Connection, _: &Stanza| {
					i.set(i.get() + 1);
					true
				}
			};

			// iq trigger, inside
			{
				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.id_handler_add(i_incrementer.clone(), "get_roster");

								let ctx = conn.context();
								let mut iq = Stanza::new_iq(&ctx, Some("get"), Some("get_roster1"));
								let mut query = Stanza::new(&ctx);
								query.set_name("query").unwrap();
								query.set_ns("jabber:iq:roster").unwrap();
								iq.add_child(query).unwrap();
								conn.send(&iq);

								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert_eq!(i.get(), 1);
			}

			// outside
			i.set(0);
			{
				let mut conn = make_conn();
				conn.id_handler_add(i_incrementer.clone(), "get_roster");
				conn.connect_client(None, None, {
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								let ctx = conn.context();
								let mut iq = Stanza::new_iq(&ctx, Some("get"), Some("get_roster1"));
								let mut query = Stanza::new(&ctx);
								query.set_name("query").unwrap();
								query.set_ns("jabber:iq:roster").unwrap();
								iq.add_child(query).unwrap();
								conn.send(&iq);

								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert_eq!(i.get(), 1);
			}

			// delete
			i.set(0);
			{
				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								let handler = conn.id_handler_add(i_incrementer.clone(), "get_roster").unwrap();
								conn.id_handler_delete(handler);

								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert_eq!(i.get(), 0);
			}

			// clear
			i.set(0);
			{
				let mut conn = make_conn();
				conn.connect_client(None, None, {
					let i_incrementer = i_incrementer.clone();
					let do_common_stuff = do_common_stuff.clone();
					move |conn, evt, _, _| {
						match evt {
							ConnectionEvent::XMPP_CONN_CONNECT => {
								conn.id_handler_add(i_incrementer.clone(), "get_roster").unwrap();
								conn.id_handlers_clear();
								do_common_stuff(conn);
							},
							ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
								conn.context().stop();
							},
							_ => {},
						}
					}
				}).unwrap();
				conn.context().run();
				assert_eq!(i.get(), 0);
			}
		}
		assert_eq!(Rc::try_unwrap(i).expect("There are hanging references to Rc value").into_inner(), 0);
	}

	#[test]
	fn handler() {
		let i = Rc::new(Cell::new(0));

		let default_con_handler = |conn: &mut Connection, evt: ConnectionEvent, _: i32, _: Option<&error::StreamError>| {
			match evt {
				ConnectionEvent::XMPP_CONN_CONNECT => {
					conn.disconnect();
				},
				ConnectionEvent::XMPP_CONN_DISCONNECT | ConnectionEvent::XMPP_CONN_FAIL => {
					conn.context().stop();
				},
				_ => {},
			}
		};

		let i_incrementer = {
			let i = i.clone();
			move |_: &mut Connection, _: &Stanza| {
				i.set(i.get() + 1);
				true
			}
		};

		// handler call stanza name filter
		{
			let mut conn = make_conn();
			conn.connect_client(None, None, default_con_handler).unwrap();
			conn.handler_add(i_incrementer.clone(), None, Some("iq"), None);
			conn.context().run();
			assert_eq!(i.get(), 1);
		}

		// handler call stanza name not existent
		i.set(0);
		{
			let mut conn = make_conn();
			conn.connect_client(None, None, default_con_handler).unwrap();
			conn.handler_add(i_incrementer.clone(), None, Some("non-existent"), None);
			conn.context().run();
			assert_eq!(i.get(), 0);
		}

		// handler delete
		i.set(0);
		{
			let mut conn = make_conn();
			conn.connect_client(None, None, default_con_handler).unwrap();
			let handler = conn.handler_add(i_incrementer.clone(), None, None, None).unwrap();
			conn.handler_delete(handler);
			conn.context().run();
			assert_eq!(i.get(), 0);
		}

		// handler clear
		i.set(0);
		{
			let mut conn = make_conn();
			conn.connect_client(None, None, default_con_handler).unwrap();
			conn.handler_add(i_incrementer.clone(), None, None, None);
			conn.handlers_clear();
			conn.context().run();
			assert_eq!(i.get(), 0);
		}

		// same handler twice
		i.set(0);
		{
			let mut conn = make_conn();
			conn.connect_client(None, None, default_con_handler).unwrap();
			assert!(matches!(conn.handler_add(&i_incrementer, None, Some("iq"), None,), Some(..)));
			assert!(matches!(conn.handler_add(&i_incrementer, None, Some("iq"), None), None));
			conn.context().run();
			assert_eq!(i.get(), 1);
		}

		// cloned handler twice, not sure if this behaviour is right
		i.set(0);
		{
			let mut conn = make_conn();
			conn.connect_client(None, None, default_con_handler).unwrap();
			assert!(matches!(conn.handler_add(i_incrementer.clone(), None, Some("iq"), None,), Some(..)));
			assert!(matches!(conn.handler_add(i_incrementer.clone(), None, Some("iq"), None), None));
			conn.context().run();
			assert_eq!(i.get(), 1);
		}
	}
}

// All of those and only those functions must fail compilation due to lifetime problems
#[cfg(feature = "fail-test")]
mod fail {
	use super::*;
	use super::super::LogLevel;

	#[test]
	fn conn_handler_too_short() {
		let conn = Connection::new(Context::new_with_null_logger());
		let not_long_enough1 = |_: &Context, _: &mut Connection, _: ConnectionEvent, _: i32, _: Option<&error::StreamError>| {};
		conn.connect_client(None, None, &not_long_enough1).unwrap();
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
		let ctx = Context::new_with_null_logger();
		{
			let not_long_enough5 = Stanza::new(&ctx);
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

}
