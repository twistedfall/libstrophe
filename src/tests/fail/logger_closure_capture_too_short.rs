use libstrophe::{Context, Logger};

fn main() {
	{
		let mut val = 0;
		Context::new(Logger::new(|_, _, _| {
			val = 1;
		}))
	};
}
