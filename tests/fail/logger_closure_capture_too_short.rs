use libstrophe::{Context, Logger};
use std::sync::atomic::{AtomicU16, Ordering};

fn main() {
	{
		let val: AtomicU16 = AtomicU16::new(0);
		Context::new(Logger::new(|_, _, _| {
			val.fetch_add(1, Ordering::Relaxed);
		}))
	};
}
