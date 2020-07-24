/*!
# FYI Msg: Traits
*/

/// Chain Bools!
///
/// This is a workaround while waiting for [#64260](https://github.com/rust-lang/rust/issues/64260) to reach stability.
/// There are just way too many if/else blocks breaking up our tidy chains.
pub trait FYIBoolChain {
	/// True Some
	///
	/// If `true`, return `Some(default)`, otherwise `None`.
	fn true_some<U>(self, default: U) -> Option<U>;

	/// True Then
	///
	/// If `true`, run the callback and return the result, otherwise `None`.
	fn true_then_some<U, F: FnOnce() -> Option<U>>(self, f: F) -> Option<U>;

	/// True Map
	///
	/// If `true`, run the callback and return the result, otherwise return
	/// `default`.
	fn true_map<U, F: FnOnce() -> U>(self, default: U, f: F) -> U;

	/// False Some
	///
	/// If `false`, return `Some(default)`, otherwise `None`.
	fn false_some<U>(self, default: U) -> Option<U>;

	/// False Then
	///
	/// If `false`, run the callback and return the result, otherwise `None`.
	fn false_then_some<U, F: FnOnce() -> Option<U>>(self, f: F) -> Option<U>;

	/// False Map
	///
	/// If `false`, run the callback and return the result, otherwise return
	/// `default`.
	fn false_map<U, F: FnOnce() -> U>(self, default: U, f: F) -> U;
}

impl FYIBoolChain for bool {
	/// True Some
	///
	/// If `true`, return `Some(default)`, otherwise `None`.
	fn true_some<U>(self, default: U) -> Option<U> {
		if self { Some(default) }
		else { None }
	}

	/// True Then
	///
	/// If `true`, run the callback and return the result, otherwise `None`.
	fn true_then_some<U, F: FnOnce() -> Option<U>>(self, f: F) -> Option<U> {
		if self { f() }
		else { None }
	}

	/// True Map
	///
	/// If `true`, run the callback and return the result, otherwise return
	/// `default`.
	fn true_map<U, F: FnOnce() -> U>(self, default: U, f: F) -> U {
		if self { f() }
		else { default }
	}

	/// False Some
	///
	/// If `false`, return `Some(default)`, otherwise `None`.
	fn false_some<U>(self, default: U) -> Option<U> {
		if self { None }
		else { Some(default) }
	}

	/// False Then
	///
	/// If `false`, run the callback and return the result, otherwise `None`.
	fn false_then_some<U, F: FnOnce() -> Option<U>>(self, f: F) -> Option<U> {
		if self { None }
		else { f() }
	}

	/// False Map
	///
	/// If `false`, run the callback and return the result, otherwise return
	/// `default`.
	fn false_map<U, F: FnOnce() -> U>(self, default: U, f: F) -> U {
		if self { default }
		else { f() }
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn true_some() {
		assert_eq!(true.true_some("hey"), Some("hey"));
		assert_eq!(false.true_some("hey"), None);
	}

	#[test]
	fn true_then_some() {
		assert_eq!(true.true_then_some(|| Some("hey")), Some("hey"));
		assert_eq!(true.true_then_some(|| None::<usize>), None);
		assert_eq!(false.true_then_some(|| Some("hey")), None);
	}

	#[test]
	fn true_map() {
		assert_eq!(true.true_map(0, || 1), 1);
		assert_eq!(false.true_map(0, || 1), 0);
	}

	#[test]
	fn false_some() {
		assert_eq!(false.false_some("hey"), Some("hey"));
		assert_eq!(true.false_some("hey"), None);
	}

	#[test]
	fn false_then_some() {
		assert_eq!(false.false_then_some(|| Some("hey")), Some("hey"));
		assert_eq!(false.false_then_some(|| None::<usize>), None);
		assert_eq!(true.false_then_some(|| Some("hey")), None);
	}

	#[test]
	fn false_map() {
		assert_eq!(false.false_map(0, || 1), 1);
		assert_eq!(true.false_map(0, || 1), 0);
	}
}
