/*!
# FYI Msg - Progless Tasks
*/

use crate::{
	fitted,
	iter::NoAnsi,
};
use std::{
	borrow::Borrow,
	cmp::Ordering,
	num::NonZeroU16,
};
use unicode_width::UnicodeWidthChar;



#[derive(Debug, Clone)]
/// # A Task.
///
/// This holds a (valid UTF-8) task name as a byte slice, pre-formatted for
/// `Progless` display.
pub(super) enum ProglessTask {
	/// # Regular ASCII.
	Ascii(Box<[u8]>),

	/// # Unicode.
	Unicode(Box<[u8]>, NonZeroU16),
}

impl Borrow<[u8]> for ProglessTask {
	#[inline]
	fn borrow(&self) -> &[u8] { self.as_slice() }
}

impl Eq for ProglessTask {}

impl Ord for ProglessTask {
	#[inline]
	fn cmp(&self, other: &Self) -> Ordering { self.as_slice().cmp(other.as_slice()) }
}

impl PartialEq for ProglessTask {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Ascii(s1), Self::Ascii(s2)) |
			(Self::Unicode(s1, _), Self::Unicode(s2, _)) => s1 == s2,
			_ => false,
		}
	}
}

impl PartialEq<str> for ProglessTask {
	#[inline]
	fn eq(&self, other: &str) -> bool { self.as_slice() == other.as_bytes() }
}

impl PartialOrd for ProglessTask {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl ProglessTask {
	/// # New.
	///
	/// Create a new task given a string slice, (possibly) reformatting for
	/// display.
	///
	/// If the source is empty or too big, `None` will be returned instead.
	pub(super) fn new(src: &str) -> Option<Self> {
		let src = src.trim_end();
		if src.is_empty() { return None; }

		// Preallocate the storage.
		let mut out = Vec::with_capacity(src.len());

		// Let's assume the source is ASCII because working with bytes is
		// (relatively) cheap.
		let mut ascii = true;
		for b in NoAnsi::<u8, _>::new(src.bytes()) {
			if b.is_ascii() {
				// Ignore control chars.
				if b.is_ascii_control() {
					// But keep whitespace (as a regular space).
					if b.is_ascii_whitespace() { out.push(b' '); }
				}
				// Keep the rest!
				else { out.push(b); }
			}
			else {
				ascii = false;
				break;
			}
		}

		// If our shortcut worked, we're done!
		if ascii {
			if out.trim_ascii().is_empty() { return None; }
			return Some(Self::Ascii(out.into_boxed_slice()));
		}

		// If not, we have to start over and loop char-by-char.
		out.truncate(0);

		let mut width = 0_usize;
		let mut buf = [0_u8; 4];
		for c in NoAnsi::<char, _>::new(src.chars()) {
			// Ignore control chars.
			if c.is_control() {
				// But keep whitespace (as a regular space).
				if c.is_whitespace() {
					width += 1;
					out.push(b' ');
				}
			}
			// Keep the rest!
			else if c.is_ascii() {
				width += 1;
				out.push(c as u8);
			}
			else {
				width += UnicodeWidthChar::width(c).unwrap_or(1);
				out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
			}
		}

		// Return it if we got it.
		if out.trim_ascii().is_empty() { None }
		else {
			u16::try_from(width).ok()
				.and_then(NonZeroU16::new)
				.map(|w| Self::Unicode(out.into_boxed_slice(), w))
		}
	}

	#[inline]
	/// # As Slice.
	///
	/// Return the inner slice, regardless of type.
	const fn as_slice(&self) -> &[u8] {
		match self { Self::Ascii(s) | Self::Unicode(s, _) => s }
	}

	#[inline]
	/// # Fitted.
	///
	/// Return the portion of the task that fits the given width, if any.
	pub(super) fn fitted(&self, width: usize) -> Option<&[u8]> {
		match self {
			// Length and width are equivalent.
			Self::Ascii(s) =>
				if s.len() <= width { Some(s) }
				else { Some(&s[..width]) },
			// Width-based truncation will be more complicated if we need it.
			Self::Unicode(s, w) =>
				if usize::from(w.get()) <= width { Some(s) }
				else {
					let end = fitted::length_width(s, width);
					if end == 0 { None }
					else { Some(&s[..end]) }
				},
		}
	}
}



#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn t_task() {
		// Quick ASCII tests.
		for (raw, expected) in [
			("\x1b[0mHello\x1b\nWorld!", Some("Hello World!")),
			("\n\x1b[0mHello\x1b\nWorld!", Some(" Hello World!")),
			("\x1b[0m ", None),
		] {
			if let Some(expected) = expected {
				let Some(found) = ProglessTask::new(raw) else {
					panic!("Task failed {raw:?}.");
				};
				assert!(matches!(found, ProglessTask::Ascii(_)));
				assert!(found == *expected);
			}
			else {
				assert!(
					ProglessTask::new(raw).is_none(),
					"Task {raw:?} should not have parsed.",
				);
			}
		}

		// Some Unicode.
		for (raw, expected) in [
			("\x1b[1mBjörk ", Some(("Björk", 5_u16))),
			("\x1b[1mGuðmundsdóttir", Some(("Guðmundsdóttir", 14_u16))),
			("\x1b[0m\u{2029} ", None),
		] {
			if let Some((ex_s, ex_w)) = expected {
				let Some(ProglessTask::Unicode(s, w)) = ProglessTask::new(raw) else {
					panic!("Task failed {raw:?}.");
				};

				assert_eq!(&*s, ex_s.as_bytes());
				assert_eq!(w.get(), ex_w);
			}
			else {
				assert!(
					ProglessTask::new(raw).is_none(),
					"Task {raw:?} should not have parsed.",
				);
			}
		}
	}

	#[test]
	fn t_task_eq() {
		// Equality should be type-dependent, but otherwise text-only.
		let text: &[u8] = b"hello world";
		assert_eq!(
			ProglessTask::Ascii(Box::from(text)),
			ProglessTask::Ascii(Box::from(text)),
		);
		assert_eq!(
			ProglessTask::Unicode(Box::from(text), NonZeroU16::MIN),
			ProglessTask::Unicode(Box::from(text), NonZeroU16::MAX),
		);
		assert_ne!(
			ProglessTask::Ascii(Box::from(text)),
			ProglessTask::Unicode(Box::from(text), NonZeroU16::MIN),
		);
	}

	#[test]
	fn t_task_fitted() {
		let a = ProglessTask::new("Hello World").unwrap();
		assert!(matches!(a, ProglessTask::Ascii(_)));
		assert_eq!(a.fitted(35), Some(&b"Hello World"[..]));
		assert_eq!(a.fitted(5), Some(&b"Hello"[..]));
		assert_eq!(a.fitted(0), Some(&b""[..]));

		let b = ProglessTask::new("Björk Guðmundsdóttir").unwrap();
		assert!(matches!(b, ProglessTask::Unicode(_, _)));
		assert_eq!(
			b.fitted(35).and_then(|s| std::str::from_utf8(s).ok()),
			Some("Björk Guðmundsdóttir"),
			"Split b35 failed.",
		);
		assert_eq!(
			b.fitted(5).and_then(|s| std::str::from_utf8(s).ok()),
			Some("Björk"),
			"Split b5 failed.",
		);
		assert_eq!(
			b.fitted(3).and_then(|s| std::str::from_utf8(s).ok()),
			Some("Bjö"),
			"Split b3 failed.",
		);
		assert_eq!(
			b.fitted(0).and_then(|s| std::str::from_utf8(s).ok()),
			None,
			"Split b0 failed.",
		);
	}
}
