/*!
# FYI Message: Print
*/

use std::io::{
	self,
	Write,
};

/// Simplify Object Printing
pub trait PrintyPlease {
	/// Print to STDOUT.
	fn fyi_print(&self);

	/// Print to STDOUT with trailing line.
	fn fyi_println(&self);

	/// Locked/flushed print to STDOUT.
	fn fyi_print_flush(&self);

	/// Locked/Flushed print to STDOUT with trailing line.
	fn fyi_println_flush(&self);
}

/// Simplify Object Error Printing
pub trait EPrintyPlease {
	/// Print to STDERR.
	fn fyi_eprint(&self);

	/// Print to STDERR with trailing line.
	fn fyi_eprintln(&self);

	/// Locked/flushed print to STDERR.
	fn fyi_eprint_flush(&self);

	/// Locked/Flushed print to STDERR with trailing line.
	fn fyi_eprintln_flush(&self);
}

impl PrintyPlease for Vec<u8> {
	/// Print to STDOUT.
	fn fyi_print(&self) {
		io::stdout().write_all(self).unwrap();
	}

	/// Print to STDOUT with trailing line.
	fn fyi_println(&self) {
		io::stdout().write_all(&self.iter().chain(&[10]).copied().collect::<Self>()).unwrap();
	}

	/// Locked/flushed print to STDOUT.
	fn fyi_print_flush(&self) {
		let writer = io::stdout();
		let mut handle = writer.lock();
		handle.write_all(self).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDOUT with trailing line.
	fn fyi_println_flush(&self) {
		let writer = io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.iter().chain(&[10]).copied().collect::<Self>()).unwrap();
		handle.flush().unwrap();
	}
}

impl EPrintyPlease for Vec<u8> {
	/// Print to STDERR.
	fn fyi_eprint(&self) {
		io::stderr().write_all(self).unwrap();
	}

	/// Print to STDERR with trailing line.
	fn fyi_eprintln(&self) {
		io::stderr().write_all(&self.iter().chain(&[10]).copied().collect::<Self>()).unwrap();
	}

	/// Locked/flushed print to STDERR.
	fn fyi_eprint_flush(&self) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		handle.write_all(self).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDERR with trailing line.
	fn fyi_eprintln_flush(&self) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.iter().chain(&[10]).copied().collect::<Self>()).unwrap();
		handle.flush().unwrap();
	}
}

impl PrintyPlease for &[u8] {
	/// Print to STDOUT.
	fn fyi_print(&self) {
		io::stdout().write_all(self).unwrap();
	}

	/// Print to STDOUT with trailing line.
	fn fyi_println(&self) {
		io::stdout().write_all(&self.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
	}

	/// Locked/flushed print to STDOUT.
	fn fyi_print_flush(&self) {
		let writer = io::stdout();
		let mut handle = writer.lock();
		handle.write_all(self).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDOUT with trailing line.
	fn fyi_println_flush(&self) {
		let writer = io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
		handle.flush().unwrap();
	}
}

impl EPrintyPlease for &[u8] {
	/// Print to STDERR.
	fn fyi_eprint(&self) {
		io::stderr().write_all(self).unwrap();
	}

	/// Print to STDERR with trailing line.
	fn fyi_eprintln(&self) {
		io::stderr().write_all(&self.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
	}

	/// Locked/flushed print to STDERR.
	fn fyi_eprint_flush(&self) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		handle.write_all(self).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDERR with trailing line.
	fn fyi_eprintln_flush(&self) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
		handle.flush().unwrap();
	}
}

impl PrintyPlease for &str {
	/// Print to STDOUT.
	fn fyi_print(&self) {
		io::stdout().write_all(self.as_bytes()).unwrap();
	}

	/// Print to STDOUT with trailing line.
	fn fyi_println(&self) {
		io::stdout().write_all(&self.as_bytes().iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
	}

	/// Locked/flushed print to STDOUT.
	fn fyi_print_flush(&self) {
		let writer = io::stdout();
		let mut handle = writer.lock();
		handle.write_all(self.as_bytes()).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDOUT with trailing line.
	fn fyi_println_flush(&self) {
		let writer = io::stdout();
		let mut handle = writer.lock();
		handle.write_all(&self.as_bytes().iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
		handle.flush().unwrap();
	}
}

impl EPrintyPlease for &str {
	/// Print to STDERR.
	fn fyi_eprint(&self) {
		io::stderr().write_all(self.as_bytes()).unwrap();
	}

	/// Print to STDERR with trailing line.
	fn fyi_eprintln(&self) {
		io::stderr().write_all(&self.as_bytes().iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
	}

	/// Locked/flushed print to STDERR.
	fn fyi_eprint_flush(&self) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		handle.write_all(self.as_bytes()).unwrap();
		handle.flush().unwrap();
	}

	/// Locked/Flushed print to STDERR with trailing line.
	fn fyi_eprintln_flush(&self) {
		let writer = io::stderr();
		let mut handle = writer.lock();
		handle.write_all(&self.as_bytes().iter().chain(&[10]).copied().collect::<Vec<u8>>()).unwrap();
		handle.flush().unwrap();
	}
}
