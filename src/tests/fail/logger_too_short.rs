use libstrophe::{Context, Logger, LogLevel};

fn main() {
	{
		let handler = |_: LogLevel, _: &str, _: &str| {};
		let logger = Logger::new(&handler);
		Context::new(logger)
	};
}
