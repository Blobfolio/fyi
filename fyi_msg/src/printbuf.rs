/*!
# FYI Message: Print Buffer

`PrintBuf` is a print-ready buffer for UTF-8 byte strings. The payload is
stored in a single `[u8]` buffer, but "part" indexes are independently stored
allowing for easy in-place manipulation — insertion, removal, replacement, etc.
— for each component.

The printing methods keep track of what was previously printed, allowing for
optional `cls`-type handling, and repaint throttling.

# Example

```no_run
use fyi_msg::PrintBuf;

let mut buf = PrintBuf::from("My favorite animal is: ");
let idx: usize = buf.add_part("dog");
assert_eq!(*buf, b"My favorite animal is: dog"[..]);

buf.replace_part(idx, "goat");
assert_eq!(*buf, b"My favorite animal is: goat"[..]);

buf.replace_part(0, "I want a ");
assert_eq!(*buf, b"I want a goat"[..]);
```
*/

use bytes::BytesMut;
use crate::utility::{
	slice_swap,
	term_width,
};
use memchr::Memchr;
use smallvec::{SmallVec, smallvec};
use std::{
	borrow::{
		Borrow,
		Cow,
	},
	fmt,
	io,
	ops::Deref,
};
use unicode_width::UnicodeWidthChar;


bitflags::bitflags! {
	/// Print flags.
	pub struct PrintFlags: u8 {
		/// Empty flag.
		const NONE = 0b00;
		/// Chop lines to terminal width.
		const CHOPPED = 0b01;
		/// Print on top of previous job.
		const REPRINT = 0b10;
	}
}

impl Default for PrintFlags {
	/// Default.
	fn default() -> Self {
		PrintFlags::NONE
	}
}



#[derive(Debug, Clone, Copy, Hash, PartialEq)]
/// Print destination.
pub enum PrinterKind {
	/// Sink.
	Sink,
	/// Stderr.
	Stderr,
	/// Stdout.
	Stdout,
}

impl Default for PrinterKind {
	/// Default.
	fn default() -> Self {
		PrinterKind::Stdout
	}
}

impl PrinterKind {
	/// Erase X Lines.
	pub fn erase(self, num: usize) {
		// Pre-compute line clearings. Ten'll do for most use cases.
		static CLEAR: [&[u8]; 10] = [
			b"\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
			b"\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K\x1B[1A\x1B[1000D\x1B[K",
		];

		// Precomputed!
		if num <= 9 {
			unsafe { self.print(CLEAR[num]); }
		}
		// We have to build it.
		else {
			unsafe {
				self.print(&[
					CLEAR[9],
					&CLEAR[1][10..].repeat(num - 9),
				].concat());
			}
		}
	}

	/// Print.
	///
	/// # Safety
	///
	/// This method assumes the data is valid UTF-8.
	pub unsafe fn print(self, buf: &[u8]) {
		use io::Write;

		match self {
			PrinterKind::Sink => {
				let mut handle = io::sink();
				if handle.write_all(buf).is_ok() {
					let _ = handle.flush().is_ok();
				}
			},
			PrinterKind::Stderr => {
				let writer = io::stderr();
				let mut handle = writer.lock();
				if handle.write_all(buf).is_ok() {
					let _ = handle.flush().is_ok();
				}
			},
			PrinterKind::Stdout => {
				let writer = io::stdout();
				let mut handle = writer.lock();
				if handle.write_all(buf).is_ok() {
					let _ = handle.flush().is_ok();
				}
			},
		}
	}

	/// Print With Trailing Line Break.
	///
	/// # Safety
	///
	/// This method assumes the data is valid UTF-8.
	pub unsafe fn println(self, buf: &[u8]) {
		use io::Write;

		match self {
			PrinterKind::Sink => {
				let mut handle = io::sink();
				let _ = writeln!(handle, "{}", std::str::from_utf8_unchecked(buf)).is_ok();
			},
			PrinterKind::Stderr => {
				let writer = io::stderr();
				let mut handle = writer.lock();
				let _ = writeln!(handle, "{}", std::str::from_utf8_unchecked(buf)).is_ok();
			},
			PrinterKind::Stdout => {
				let writer = io::stdout();
				let mut handle = writer.lock();
				let _ = writeln!(handle, "{}", std::str::from_utf8_unchecked(buf)).is_ok();
			},
		}
	}
}



#[derive(Debug, Clone, Default, Hash, PartialEq)]
/// Print Buffer
pub struct PrintBuf {
	/// The buffer.
	buf: BytesMut,
	/// The index of parts.
	parts: SmallVec<[(usize, usize); 16]>,
	/// Destination.
	printer: PrinterKind,
	/// The buffer hash that was last printed.
	last_hash: u64,
	/// The number of lines that were last printed.
	last_lines: usize,
	/// The CLI width during the last print.
	last_width: usize,
	/// The wrapping method last used, if any.
	last_wrap: bool,
}

impl AsRef<str> for PrintBuf {
	#[inline]
	fn as_ref(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

impl AsRef<[u8]> for PrintBuf {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl Borrow<str> for PrintBuf {
	#[inline]
	fn borrow(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self) }
	}
}

impl Borrow<[u8]> for PrintBuf {
	#[inline]
	fn borrow(&self) -> &[u8] {
		self
	}
}

impl Deref for PrintBuf {
	type Target = [u8];

	/// Deref.
	fn deref(&self) -> &Self::Target {
		&self.buf
	}
}

impl fmt::Display for PrintBuf {
	#[inline]
	/// Display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(unsafe { std::str::from_utf8_unchecked(self) })
	}
}

impl PrintBuf {
	#[must_use]
	/// From Part.
	///
	/// Create a new `PrintBuf` with content `part`. The seeding value will be
	/// assigned part index 0.
	pub fn from<T> (part: T) -> Self
	where T: Borrow<str> {
		unsafe { PrintBuf::from_unchecked(part.borrow().as_bytes()) }
	}

	#[must_use]
	/// From Part (Unchecked).
	///
	/// This method performs the same tasks as `from()`, without UTF-8 checks.
	///
	/// # Safety
	///
	/// This method accepts a raw `[u8]` that is assumed to be valid UTF-8.
	pub unsafe fn from_unchecked(part: &[u8]) -> Self {
		PrintBuf {
			buf: BytesMut::from(part),
			parts: smallvec![(0, part.len())],
			..PrintBuf::default()
		}
	}

	#[must_use]
	/// From Parts.
	///
	/// Create a new `PrintBuf` with content `part`. The seeding value will be
	/// assigned part index 0.
	pub fn from_parts<T> (parts: &[T]) -> Self
	where T: Borrow<str> {
		let mut buf: PrintBuf = PrintBuf::with_capacity(256);
		for p in parts {
			unsafe { buf.add_part_unchecked(p.borrow().as_bytes()); }
		}
		buf
	}

	#[must_use]
	/// With Capacity.
	///
	/// Initialize an empty `PrintBuf` with capacity.
	pub fn with_capacity(cap: usize) -> Self {
		PrintBuf {
			buf: BytesMut::with_capacity(cap),
			parts: SmallVec::new(),
			..PrintBuf::default()
		}
	}

	/// Add Line Break.
	///
	/// Add a `\n` to the end of the stream, increasing the part count by one.
	pub fn add_line(&mut self) -> usize {
		unsafe { self.add_part_unchecked(b"\n") }
	}

	/// Add Part.
	///
	/// Add a new part to the end of the stream, increasing the part count by
	/// one. The return value corresponds to the part index that was just
	/// added.
	pub fn add_part<T> (&mut self, part: T) -> usize
	where T: Borrow<str> {
		unsafe { self.add_part_unchecked(part.borrow().as_bytes()) }
	}

	/// Add Part (Unchecked).
	///
	/// This method performs the same tasks as `add_part()`, without UTF-8
	/// checks.
	///
	/// # Safety
	///
	/// This method accepts a raw `[u8]` that is assumed to be valid UTF-8.
	pub unsafe fn add_part_unchecked(&mut self, part: &[u8]) -> usize {
		let start: usize = self.buf.len();
		self.buf.extend_from_slice(part);
		self.parts.push((start, self.buf.len()));
		self.parts.len() - 1
	}

	/// Add Multiple Parts.
	///
	/// This method works just like `add_part()` except it adds multiple parts
	/// in one go. The return value is the last part index added.
	pub fn add_parts<T> (&mut self, parts: &[T]) -> usize
	where T: Borrow<str> {
		for part in parts {
			unsafe { self.add_part_unchecked(part.borrow().as_bytes()); }
		}
		self.parts.len() - 1
	}

	/// Add Multiple Parts (Unchecked).
	///
	/// This method performs the same tasks as `add_parts()`, without UTF-8
	/// checks.
	///
	/// # Safety
	///
	/// This method accepts a raw `[u8]` that is assumed to be valid UTF-8.
	pub unsafe fn add_parts_unchecked(&mut self, parts: &[&[u8]]) -> usize {
		for part in parts {
			self.add_part_unchecked(part);
		}
		self.parts.len() - 1
	}

	/// Clear.
	///
	/// This clears the buffer and parts, leaving you with an empty buffer.
	/// This does not re-allocate or shrink; to clear memory, drop the buffer.
	pub fn clear(&mut self) {
		self.buf.clear();
		self.parts.clear();
	}

	#[must_use]
	/// Get Part.
	///
	/// Return a part of the buffer (or an empty string if the index is out of
	/// range).
	pub fn get_part(&self, idx: usize) -> Cow<[u8]> {
		if idx < self.parts.len() {
			unsafe { self.get_part_unchecked(idx) }
		}
		else {
			Cow::Borrowed(&[])
		}
	}

	#[must_use]
	/// Get Part (Unchecked).
	///
	/// This method performs the same tasks as `get_part()`, without checking
	/// the index is in range.
	///
	/// # Safety
	///
	/// This method assumes `idx` is in range.
	pub unsafe fn get_part_unchecked(&self, idx: usize) -> Cow<[u8]> {
		Cow::Borrowed(&self.buf[self.parts[idx].0..self.parts[idx].1])
	}

	/// Insert Line Break.
	///
	/// Insert a `\n` as a new part at index `idx`, shifting subsequent parts
	/// to the right.
	pub fn insert_line(&mut self, idx: usize) {
		if idx < self.parts.len() {
			unsafe { self.insert_part_unchecked(idx, b"\n"); }
		}
		else {
			unsafe { self.add_part_unchecked(b"\n"); }
		}
	}

	/// Insert Part.
	///
	/// Insert a new part at index `idx`, shifting subsequent parts to the
	/// right.
	pub fn insert_part<T> (&mut self, idx: usize, part: T)
	where T: Borrow<str> {
		if idx < self.parts.len() {
			unsafe { self.insert_part_unchecked(idx, part.borrow().as_bytes()); }
		}
		else {
			unsafe { self.add_part_unchecked(part.borrow().as_bytes()); }
		}
	}

	/// Insert Part (Unchecked).
	///
	/// This method performs the same tasks as `insert_part()`, without
	/// checking the index is in range or validating the UTF-8 content of the
	/// part.
	///
	/// # Safety
	///
	/// This method assumes `idx` is in range, and that `part` is valid UTF-8.
	pub unsafe fn insert_part_unchecked(&mut self, idx: usize, part: &[u8]) {
		let adj: usize = part.len();

		// Add to the beginning.
		if idx == 0 {
			// Inject the slice.
			let b = self.buf.split_off(0);
			self.buf.extend_from_slice(part);
			self.buf.unsplit(b);

			// Shift the indexes.
			self.parts.insert(0, (0, adj));
			self.shift_parts_right(1, adj);
		}
		// Somewhere in the middle.
		else {
			// Inject the slice.
			let b = self.buf.split_off(self.parts[idx].0);
			self.buf.extend_from_slice(part);
			self.buf.unsplit(b);

			self.parts.insert(idx, (self.parts[idx].0, self.parts[idx].0 + adj));
			self.shift_parts_right(idx + 1, adj);
		}
	}

	/// Partition by Line.
	pub fn partition_by_lines(&mut self) -> Vec<usize> {
		if self.is_empty() {
			self.parts.clear();
			vec![0]
		}
		else {
			let mut out: Vec<usize> = Vec::new();
			let mut last_idx: usize = 0;
			self.parts.clear();
			for idx in Memchr::new(10_u8, &self.buf) {
				if idx != 0 {
					self.parts.push((last_idx, idx));
					out.push(self.parts.len() - 1);
				}
				last_idx = idx + 1;
				self.parts.push((idx, last_idx));
			}
			if last_idx < self.buf.len() {
				self.parts.push((last_idx, self.buf.len()));
				out.push(self.parts.len() - 1);
			}

			out
		}
	}

	/// Print to Writer.
	///
	/// (Re)print the buffer to the specified writer.
	///
	/// If `reprint` is `true`, the last print job — if any — will first be
	/// erased so that the new job can be printed "over" it. If the two jobs
	/// are identical and nothing in the environment has changed, no action
	/// will be taken.
	pub fn print(&mut self, flags: PrintFlags) {
		let hash: u64 = seahash::hash(&self.buf);
		let chopped: bool = flags.contains(PrintFlags::CHOPPED);
		let width: usize = if chopped {
			term_width()
		}
		else { 0 };

		// Reprint?
		if flags.contains(PrintFlags::REPRINT) {
			// We don't have to do anything!
			if self.last_hash == hash &&
				self.last_width == width &&
				self.last_wrap == chopped {
				return;
			}
			// We need to clear some lines.
			else if 0 != self.last_lines {
				self.printer.erase(self.last_lines);
			}
		}

		// Update the lasts.
		self.last_hash = hash;
		self.last_width = width;
		self.last_wrap = chopped;

		// Nothing?
		if self.buf.is_empty() {
			self.last_lines = 0;
			return;
		}

		// Get a line count.
		self.last_lines = bytecount::count(&self.buf, b'\n') + 1;

		if chopped {
			unsafe { self.printer.println(self.chop(width).as_ref()) }
		}
		else {
			unsafe { self.printer.println(self) }
		}
	}

	/// Remove Part.
	///
	/// Remove a part, shifting all subsequent parts to the left.
	pub fn remove_part(&mut self, idx: usize) {
		if idx < self.parts.len() {
			unsafe { self.remove_part_unchecked(idx) }
		}
	}

	/// Remove Part (Unchecked).
	///
	/// This method performs the same tasks as `remove_part()`, without
	/// checking the index is in range.
	///
	/// # Safety
	///
	/// This method assumes `idx` is in range.
	pub unsafe fn remove_part_unchecked(&mut self, idx: usize) {
		let old_len: usize = self.parts.len();

		// The last item is easy to remove.
		if idx + 1 == old_len {
			// If this is the only item, just clear everything.
			if 0 == idx {
				self.parts.clear();
				self.buf.clear();
			}
			// Otherwise drop the position index and truncate the buffer
			// accordingly.
			else {
				let pos = self.parts.remove(idx);
				self.buf.truncate(pos.0);
			}
		}
		// Put on the surgeon gloves!
		else {
			// Drop the index.
			let pos = self.parts.remove(idx);

			// Split, chop, and glue the buffer.
			let b = self.buf.split_off(pos.1);
			self.buf.truncate(pos.0);
			self.buf.unsplit(b);

			// Now we need to shift all the remaining positions.
			self.shift_parts_left(idx, pos.1 - pos.0);
		}
	}

	/// Replace Part.
	///
	/// Replace the content of a part. Only changed bytes are written to the
	/// buffer, and subsequent parts are nudged to the left or the right as
	/// needed.
	pub fn replace_part<T> (&mut self, idx: usize, part: T)
	where T: Borrow<str> {
		if idx < self.parts.len() {
			unsafe { self.replace_part_unchecked(idx, part.borrow().as_bytes()) }
		}
	}

	/// Replace Part (Unchecked).
	///
	/// This method performs the same tasks as `replace_part()`, without
	/// checking the index is in range or validating the UTF-8 content of the
	/// part.
	///
	/// # Safety
	///
	/// This method assumes `idx` is in range, and that `part` is valid UTF-8.
	pub unsafe fn replace_part_unchecked(&mut self, idx: usize, part: &[u8]) {
		// Only bother doing stuff if stuff changed.
		if &self.buf[self.parts[idx].0..self.parts[idx].1] == part {
			return;
		}

		// We'll have work to do if the length changes.
		let old_len: usize = self.parts[idx].1 - self.parts[idx].0;
		let new_len: usize = part.len();

		// The simplest kind of conversion is an in-place one!
		if old_len == new_len {
			slice_swap(
				&mut self.buf[self.parts[idx].0..self.parts[idx].1],
				part
			);
		}
		// If there's only one part, we can handle length changes a bit more
		// directly.
		else if self.parts.len() == 1 {
			// The new slice is bigger so we have to do two writes.
			if new_len > old_len {
				slice_swap(
					&mut self.buf[..],
					&part[0..old_len]
				);
				self.buf.extend_from_slice(&part[old_len..]);
			}
			// If the new slice is smaller, we can shrink the old slice, then
			// copy in-place.
			else {
				self.buf.truncate(new_len);
				slice_swap(
					&mut self.buf[..],
					part
				);
			}

			// Nudge the index position accordingly.
			self.parts[0].1 = new_len;
		}
		// This will require a bit of surgery.
		else {
			// Chop the buffer so that `b` holds everything that comes after
			// the original slice ending.
			let b = self.buf.split_off(self.parts[idx].1);

			// The new slice is bigger so we have to do two writes.
			if new_len > old_len {
				slice_swap(
					&mut self.buf[self.parts[idx].0..],
					&part[0..old_len]
				);
				self.buf.extend_from_slice(&part[old_len..]);

				// Bump this and following index positions accordingly.
				let adj: usize = new_len - old_len;
				self.parts[idx].1 += adj;
				self.shift_parts_right(idx + 1, adj);
			}
			// If the new slice is smaller, we can shrink the old slice, then
			// copy in-place.
			else {
				let adj: usize = old_len - new_len;
				self.buf.truncate(self.buf.len() - adj);
				slice_swap(
					&mut self.buf[self.parts[idx].0..],
					part
				);

				// Drop this and following index positions accordingly.
				self.parts[idx].1 -= adj;
				self.shift_parts_left(idx + 1, adj);
			}

			// Glue the pieces back together.
			self.buf.unsplit(b);
		}
	}

	/// Set Printer.
	pub fn set_printer(&mut self, printer: PrinterKind) {
		if printer != self.printer {
			self.printer = printer;
			self.last_hash = 0;
			self.last_lines = 0;
			self.last_width = 0;
			self.last_wrap = false;
		}
	}

	/// Chop it.
	///
	/// # Safety
	///
	/// This method assumes the data is valid UTF-8.
	unsafe fn chop(&self, width: usize) -> Cow<[u8]> {
		let len: usize = self.buf.len();
		// Easy abort.
		if len <= width {
			Cow::Borrowed(&self.buf)
		}
		else {
			let mut buf2: PrintBuf = self.clone();
			let line_parts = buf2.partition_by_lines();
			for idx in line_parts {
				let slice: Vec<u8> = PrintBuf::chop_line(&buf2.get_part_unchecked(idx), width).to_vec();
				buf2.replace_part_unchecked(idx, &slice);
			}

			// If the lengths match, we can assume nothing changed.
			if self.buf.len() == buf2.len() {
				Cow::Borrowed(&self.buf)
			}
			// Pass the chopped value.
			else {
				Cow::Owned(buf2.to_vec())
			}
		}
	}

	/// Width At.
	///
	/// Chop the data once a certain width has been received.
	unsafe fn chop_line(buf: &[u8], width: usize) -> Cow<[u8]> {
		// Easy abort.
		let len: usize = buf.len();
		if len <= width {
			return Cow::Borrowed(buf);
		}

		let mut cur_len: usize = 0;
		let mut cur_width: usize = 0;
		let mut in_ansi: bool = false;

		for c in std::str::from_utf8_unchecked(buf).chars() {
			let char_len: usize = c.len_utf8();

			if in_ansi {
				if c == 'A' || c == 'K' || c == 'm' {
					in_ansi = false;
				}

				cur_len += char_len;
				continue;
			}
			else if c == '\x1b' {
				in_ansi = true;
				cur_len += char_len;
				continue;
			}

			let char_width: usize = UnicodeWidthChar::width(c).unwrap_or(0);
			cur_len += char_len;
			cur_width += char_width;

			#[allow(clippy::comparison_chain)]
			// We've gone over!
			if cur_width > width {
				cur_len -= char_len;
				break;
			}
			// We've hit it exactly!
			else if cur_width == width {
				break;
			}
		}

		// No length change... send it back unchanged!
		if cur_len == len {
			Cow::Borrowed(buf)
		}
		// Rebuild it better than it was!
		else {
			Cow::Owned([
				&buf[..cur_len],
				b"\x1B[0m",
			].concat())
		}
	}

	/// Shift Parts Left.
	///
	/// This is an internal helper that shifts the indexes of all parts
	/// beginning at `idx` left by `num` amount.
	///
	/// Left shifts occur when a part of the buffer shrinks, due to a shorter
	/// replacement, outright removal, etc.
	fn shift_parts_left(&mut self, mut idx: usize, num: usize) {
		let len: usize = self.parts.len();
		while idx < len {
			self.parts[idx].0 -= num;
			self.parts[idx].1 -= num;
			idx += 1;
		}
	}

	/// Shift Parts Right.
	///
	/// This is an internal helper that shifts the indexes of all parts
	/// beginning at `idx` right by `num` amount.
	///
	/// Right shifts occur when a part of the buffer expands, due to a longer
	/// replacement, insertion, etc.
	fn shift_parts_right(&mut self, mut idx: usize, num: usize) {
		let len: usize = self.parts.len();
		while idx < len {
			self.parts[idx].0 += num;
			self.parts[idx].1 += num;
			idx += 1;
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_from() {
		let buf = PrintBuf::from("Your Mom");
		assert_eq!(*buf, b"Your Mom"[..]);
	}

	#[test]
	fn t_add_part() {
		let mut buf = PrintBuf::from("Your Mom");
		assert_eq!(*buf, b"Your Mom"[..]);

		let mut idx: usize = buf.add_part(" Thinks You're Silly.");
		assert_eq!(idx, 1);
		assert_eq!(*buf, b"Your Mom Thinks You're Silly."[..]);

		idx = buf.add_part(" But that's OK.");
		assert_eq!(idx, 2);
		assert_eq!(*buf, b"Your Mom Thinks You're Silly. But that's OK."[..]);

		buf.clear();
		assert_eq!(*buf, b""[..]);

		idx = buf.add_part("Hello World");
		assert_eq!(idx, 0);
		assert_eq!(*buf, b"Hello World"[..]);

		// And test adding a line break.
		idx = buf.add_line();
		assert_eq!(idx, 1);
		assert_eq!(*buf, b"Hello World\n"[..]);
	}

	#[test]
	fn t_add_parts() {
		let mut buf = PrintBuf::default();
		assert_eq!(*buf, b""[..]);

		buf.add_parts(&[
			"Your Mom",
			" Thinks You're Silly.",
		]);
		assert_eq!(*buf, b"Your Mom Thinks You're Silly."[..]);

		buf.add_parts(&[
			" But that's OK.",
			" You are!",
		]);
		assert_eq!(*buf, b"Your Mom Thinks You're Silly. But that's OK. You are!"[..]);
	}

	#[test]
	fn t_get_part() {
		let mut buf = PrintBuf::default();
		assert_eq!(*buf, b""[..]);

		let parts = [
			"One",
			"Two",
			"Three",
		];

		buf.add_parts(&parts);
		assert_eq!(*buf, b"OneTwoThree"[..]);

		for (k, v) in parts.iter().enumerate() {
			assert_eq!(buf.get_part(k).as_ref(), v.as_bytes());
		}
	}

	#[test]
	fn t_insert_part() {
		let mut buf = PrintBuf::default();
		assert_eq!(*buf, b""[..]);

		let parts = [
			"One",
			"Two",
			"Three",
		];

		buf.add_parts(&parts);
		assert_eq!(*buf, b"OneTwoThree"[..]);

		buf.insert_part(1, "Four");
		assert_eq!(*buf, b"OneFourTwoThree"[..]);

		assert_eq!(buf.get_part(0).as_ref(), parts[0].as_bytes());
		assert_eq!(buf.get_part(1).as_ref(), &b"Four"[..]);
		assert_eq!(buf.get_part(2).as_ref(), parts[1].as_bytes());
		assert_eq!(buf.get_part(3).as_ref(), parts[2].as_bytes());

		// Insert at zero.
		buf.insert_part(0, "Five");
		assert_eq!(*buf, b"FiveOneFourTwoThree"[..]);

		assert_eq!(buf.get_part(0).as_ref(), &b"Five"[..]);
		assert_eq!(buf.get_part(1).as_ref(), parts[0].as_bytes());
		assert_eq!(buf.get_part(2).as_ref(), &b"Four"[..]);
		assert_eq!(buf.get_part(3).as_ref(), parts[1].as_bytes());
		assert_eq!(buf.get_part(4).as_ref(), parts[2].as_bytes());

		// Insert at the end. This uses `add_part()`, so we we just need to
		// verify the main buffer bit; if `add_part()` is broken, its own test
		// will make that clear.
		buf.insert_part(5, "Six");
		assert_eq!(*buf, b"FiveOneFourTwoThreeSix"[..]);

		// And let's check that `insert_line()` works.
		buf.insert_line(5);
		assert_eq!(*buf, b"FiveOneFourTwoThree\nSix"[..]);
	}

	#[test]
	fn t_remove_part() {
		let mut buf = PrintBuf::from("Your Mom");
		assert_eq!(*buf, b"Your Mom"[..]);

		buf.remove_part(0);
		assert!(buf.is_empty());

		let parts = [
			"One",
			"Two",
			"Three",
		];

		buf.add_parts(&parts);
		assert_eq!(*buf, b"OneTwoThree"[..]);

		buf.remove_part(2);
		assert_eq!(*buf, b"OneTwo"[..]);
		assert_eq!(buf.get_part(0).as_ref(), parts[0].as_bytes());
		assert_eq!(buf.get_part(1).as_ref(), parts[1].as_bytes());

		buf.remove_part(0);
		assert_eq!(*buf, b"Two"[..]);
		assert_eq!(buf.get_part(0).as_ref(), parts[1].as_bytes());
	}

	#[test]
	fn t_replace_part() {
		let mut buf = PrintBuf::default();
		assert_eq!(*buf, b""[..]);

		buf.add_parts(&[
			"Apples",
			"Bananas",
			"Carrots",
		]);
		assert_eq!(*buf, b"ApplesBananasCarrots"[..]);

		// Test shrinking replace at each spot.
		buf.replace_part(0, "Pairs");
		assert_eq!(*buf, b"PairsBananasCarrots"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Pairs"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Bananas"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Carrots"[..]);

		buf.replace_part(1, "Dates");
		assert_eq!(*buf, b"PairsDatesCarrots"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Pairs"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Dates"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Carrots"[..]);

		buf.replace_part(2, "Roots");
		assert_eq!(*buf, b"PairsDatesRoots"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Pairs"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Dates"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Roots"[..]);

		// Now try replacing with elements of the same size.
		buf.replace_part(0, "Bears");
		assert_eq!(*buf, b"BearsDatesRoots"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Bears"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Dates"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Roots"[..]);

		buf.replace_part(1, "Bears");
		assert_eq!(*buf, b"BearsBearsRoots"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Bears"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Bears"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Roots"[..]);

		buf.replace_part(2, "Bears");
		assert_eq!(*buf, b"BearsBearsBears"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Bears"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Bears"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Bears"[..]);

		// Now test growing.
		buf.replace_part(0, "Polar Bears");
		assert_eq!(*buf, b"Polar BearsBearsBears"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Polar Bears"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Bears"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Bears"[..]);

		buf.replace_part(1, "Polar Bears");
		assert_eq!(*buf, b"Polar BearsPolar BearsBears"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Polar Bears"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Polar Bears"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Bears"[..]);

		buf.replace_part(2, "Polar Bears");
		assert_eq!(*buf, b"Polar BearsPolar BearsPolar Bears"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"Polar Bears"[..]);
		assert_eq!(buf.get_part(1).as_ref(), &b"Polar Bears"[..]);
		assert_eq!(buf.get_part(2).as_ref(), &b"Polar Bears"[..]);

		// Now let's do all of this again, but for a buffer where there's just
		// one entry.
		buf.clear();
		buf.add_part("dog");
		assert_eq!(*buf, b"dog"[..]);

		// Same size.
		buf.replace_part(0, "cat");
		assert_eq!(*buf, b"cat"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"cat"[..]);

		// Bigger.
		buf.replace_part(0, "gorilla");
		assert_eq!(*buf, b"gorilla"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"gorilla"[..]);

		// Smaller.
		buf.replace_part(0, "bat");
		assert_eq!(*buf, b"bat"[..]);
		assert_eq!(buf.get_part(0).as_ref(), &b"bat"[..]);
	}
}
