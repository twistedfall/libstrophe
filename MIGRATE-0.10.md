There are many API changes from 0.9 to 0.10 and most probably you'll need to migrate your code.
This is the summary of breaking changes:

 * the crate uses `edition = "2018"` thus requiring `rust-1.31`
 * `Connection` consumes `Context` on creation and releases it on `connect_*()`, allows to avoid using `Arc` and provides more safety
 * type signatures for `Connection`, `Context` have an additional lifetime arg
 * `Connection::context()` is removed, context is now passed as the first argument to a callback
 * `Connection` and `Logger` own the callbacks supplied to them, passing closure references is still possible, all callbacks must be `Send`
 * `Connection::*handler_add()` return special handles used to remove callbacks later
 * `Connection::*handler_delete()` take special handles returned from `Connection::*handler_add()` and not callback references
 * `Context` implements memory allocation through Rust allocator
 * `Context::set_timeout()` takes &mut self instead of &self
 * the crate uses failure instead of error-chain
 * `StreamError` no longer implements `std::error::Error`
 * `Stanza::context()` is no longer accessible publicly
 * `Stanza::get_first_child_mut()` takes &mut self instead of &self
