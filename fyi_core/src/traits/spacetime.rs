use bytes::{
	BytesMut,
	BufMut
};
use crate::{
	Error,
	Result,
};
use std::{
	borrow::Cow,
	fs,
	os::unix::fs::PermissionsExt,
	path::{
		Path,
		PathBuf,
	},
};



lazy_static::lazy_static! {
	static ref ELAPSED_ONE: [&'static [u8]; 4] = [
		" day".as_bytes(),
		" hour".as_bytes(),
		" minute".as_bytes(),
		" second".as_bytes(),
	];
	static ref ELAPSED_MANY: [&'static [u8]; 4] = [
		" days".as_bytes(),
		" hours".as_bytes(),
		" minutes".as_bytes(),
		" seconds".as_bytes(),
	];
}



/// Canon or death.
pub trait AbsPath {
	/// Parent.
	fn parent_abs(&self) -> Result<PathBuf>;

	/// Absolute PathBuf.
	fn to_path_buf_abs(&self) -> PathBuf;
}

impl<T> AbsPath for T
where T: AsRef<Path> {
	/// Parent.
	fn parent_abs(&self) -> Result<PathBuf> {
		if let Some(dir) = self.as_ref().parent() {
			if dir.is_dir() {
				return Ok(dir.to_path_buf_abs());
			}
		}

		Err(Error::PathInvalid(self.as_ref().to_path_buf(), "has no parent"))
	}

	/// Absolute PathBuf.
	fn to_path_buf_abs(&self) -> PathBuf {
		match fs::canonicalize(self.as_ref()) {
			Ok(path) => path,
			_ => self.as_ref().to_path_buf(),
		}
	}
}



/// Human-Readable Elapsed.
pub trait Elapsed {
	/// Output type for chunks.
	type Chunk;

	/// Elapsed Chunks
	///
	/// Return a fixed array containing the number of days, hours,
	/// minutes, and seconds.
	fn elapsed_chunks(&self) -> Self::Chunk;

	/// Human-Readable Elapsed Time
	///
	/// This turns seconds into a human list like 1 hour and 2 seconds.
	fn elapsed (&self) -> Cow<'static, str>;

	/// Elapsed Time (Compact)
	///
	/// This turns seconds into a 00:00:00-style display. Days are included
	/// only if positive.
	fn elapsed_short(&self) -> Cow<'static, str>;
}



macro_rules! impl_elapsed {
	($type:ty) => {
		impl Elapsed for $type {
			type Chunk = [$type; 4];

			/// Elapsed Chunks
			///
			/// Return a fixed array containing the number of days, hours,
			/// minutes, and seconds.
			fn elapsed_chunks(&self) -> Self::Chunk {
				let mut out: [$type; 4] = [0, 0, 0, *self];

				// Days.
				if out[3] >= 86400 {
					out[0] = out[3] / 86400;
					out[3] -= out[0] * 86400;
				}

				// Hours.
				if out[3] >= 3600 {
					out[1] = out[3] / 3600;
					out[3] -= out[1] * 3600;
				}

				// Minutes.
				if out[3] >= 60 {
					out[2] = out[3] / 60;
					out[3] -= out[2] * 60;
				}

				out
			}

			/// Human-Readable Elapsed Time
			///
			/// This turns seconds into a human list like 1 hour and 2 seconds.
			fn elapsed (&self) -> Cow<'static, str> {
				if 0 == *self {
					"0 seconds".into()
				}
				else if 1 == *self {
					"1 second".into()
				}
				else if *self < 60 {
					let mut buf = BytesMut::with_capacity(32);
					let mut cache = [0u8; 20];
					let n = itoa::write(&mut cache[..], *self).unwrap();
					buf.put(&cache[0..n]);
					buf.put(ELAPSED_MANY[3].as_ref());
					unsafe { String::from_utf8_unchecked(buf.to_vec()).into() }
				}
				else {
					let c = self.elapsed_chunks();
					let len: usize = c.iter().filter(|&n| *n != 0).count();
					assert!(len > 0);

					let comma = ", ".as_bytes();
					let mut buf = BytesMut::with_capacity(32);
					let mut cache = [0u8; 20];
					let mut i: usize = 0;
					let mut j: usize = 0;
					loop {
						// Skip empties.
						if c[i] == 0 {
							i += 1;
							continue;
						}

						let n = itoa::write(&mut cache[..], c[i]).unwrap();
						buf.put(&cache[0..n]);
						match c[i] {
							1 => buf.put(ELAPSED_ONE[i].as_ref()),
							_ => buf.put(ELAPSED_MANY[i].as_ref()),
						}

						i += 1;
						j += 1;

						if j == len {
							break;
						}
						else if len - j == 1 {
							if len > 2 {
								buf.put(", and ".as_bytes());
							}
							else {
								buf.put(" and ".as_bytes());
							}
						}
						else {
							buf.put(comma);
						}
					}

					unsafe { String::from_utf8_unchecked(buf.to_vec()).into() }
				}
			}

			/// Elapsed Time (Compact)
			///
			/// This turns seconds into a 00:00:00-style display. Days are included
			/// only if positive.
			fn elapsed_short(&self) -> Cow<'static, str> {
				if 0 == *self {
					"00:00:00".into()
				}
				else {
					let c = self.elapsed_chunks();
					if 0 != c[0] {
						format!(
							"{:02}:{:02}:{:02}:{:02}",
							c[0], c[1], c[2], c[3]
						).into()
					}
					else {
						format!(
							"{:02}:{:02}:{:02}",
							c[1], c[2], c[3]
						).into()
					}
				}
			}
		}
	};
}



impl_elapsed!(usize);
impl_elapsed!(u32);
impl_elapsed!(u64);
impl_elapsed!(u128);
impl_elapsed!(i32);
impl_elapsed!(i64);



/// Make basic path data more accessible.
pub trait PathProps {
	/// File extension.
	fn file_extension(&self) -> Cow<'static, str>;

	/// File name.
	fn file_name(&self) -> Cow<'static, str>;

	/// File Size.
	fn file_size(&self) -> u64;

	/// Is Executable?
	fn is_executable(&self) -> bool;
}



impl<T> PathProps for T
where T: AsRef<Path> {
	/// Extension.
	fn file_extension(&self) -> Cow<'static, str> {
		let path = self.as_ref();

		if path.is_dir() {
			Cow::Borrowed("")
		}
		else {
			match path.extension() {
				Some(ext) => Cow::Owned(ext.to_str().unwrap_or("").to_lowercase()),
				_ => Cow::Borrowed(""),
			}
		}
	}

	/// File name.
	fn file_name(&self) -> Cow<'static, str> {
		let path = self.as_ref();

		if path.is_dir() {
			Cow::Borrowed("")
		}
		else {
			match path.file_name() {
				Some(name) => Cow::Owned(name.to_str().unwrap_or("").to_string()),
				_ => Cow::Borrowed(""),
			}
		}
	}

	/// File Size.
	fn file_size(&self) -> u64 {
		if let Ok(meta) = self.as_ref().metadata() {
			if meta.is_file() {
				return meta.len();
			}
		}

		0
	}

	/// Is Executable?
	fn is_executable(&self) -> bool {
		if let Ok(meta) = self.as_ref().metadata() {
			if meta.is_file() {
				let permissions = meta.permissions();
				return permissions.mode() & 0o111 != 0;
			}
		}

		return false;
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn elapsed_chunks() {
		assert_eq!(0.elapsed_chunks(), [0, 0, 0, 0]);
		assert_eq!(50.elapsed_chunks(), [0, 0, 0, 50]);
		assert_eq!(100.elapsed_chunks(), [0, 0, 1, 40]);
		assert_eq!(1000.elapsed_chunks(), [0, 0, 16, 40]);
		assert_eq!(10000.elapsed_chunks(), [0, 2, 46, 40]);
		assert_eq!(100000.elapsed_chunks(), [1, 3, 46, 40]);
		assert_eq!(1000000.elapsed_chunks(), [11, 13, 46, 40]);
	}

	#[test]
	fn elapsed() {
		assert_eq!(0.elapsed(), Cow::Borrowed("0 seconds"));
		assert_eq!(1.elapsed(), Cow::Borrowed("1 second"));
		assert_eq!(50.elapsed(), Cow::Borrowed("50 seconds"));
		assert_eq!(100.elapsed(), Cow::Borrowed("1 minute and 40 seconds"));
		assert_eq!(1000.elapsed(), Cow::Borrowed("16 minutes and 40 seconds"));
		assert_eq!(10000.elapsed(), Cow::Borrowed("2 hours, 46 minutes, and 40 seconds"));
		assert_eq!(100000.elapsed(), Cow::Borrowed("1 day, 3 hours, 46 minutes, and 40 seconds"));
		assert_eq!(1000000.elapsed(), Cow::Borrowed("11 days, 13 hours, 46 minutes, and 40 seconds"));
	}

	#[test]
	fn elapsed_short() {
		assert_eq!(0.elapsed_short(), Cow::Borrowed("00:00:00"));
		assert_eq!(1.elapsed_short(), Cow::Borrowed("00:00:01"));
		assert_eq!(50.elapsed_short(), Cow::Borrowed("00:00:50"));
		assert_eq!(100.elapsed_short(), Cow::Borrowed("00:01:40"));
		assert_eq!(1000.elapsed_short(), Cow::Borrowed("00:16:40"));
		assert_eq!(10000.elapsed_short(), Cow::Borrowed("02:46:40"));
		assert_eq!(100000.elapsed_short(), Cow::Borrowed("01:03:46:40"));
		assert_eq!(1000000.elapsed_short(), Cow::Borrowed("11:13:46:40"));
	}

	#[test]
	fn file_extension() {
		assert_eq!(PathBuf::from("foo/bar.JS").file_extension(), "js");
		assert_eq!(PathBuf::from("src/lib.rs").file_extension(), "rs");

		assert_eq!(PathBuf::from("foo/bar").file_extension(), "");
		assert_eq!(PathBuf::from(env!("CARGO_MANIFEST_DIR")).file_extension(), "");
	}

	#[test]
	fn file_name() {
		assert_eq!(PathBuf::from("foo/bar.JS").file_name(), "bar.JS");
		assert_eq!(PathBuf::from("src/lib.rs").file_name(), "lib.rs");

		// Should return "bar" since the path doesn't exist and might be
		// intended to hold a file at some point.
		assert_eq!(PathBuf::from("foo/bar").file_name(), "bar");

		// This is definitely a directory, though, so shouldn't return
		// anything.
		assert_eq!(PathBuf::from(env!("CARGO_MANIFEST_DIR")).file_name(), "");
	}

	#[test]
	fn file_size() {
		// These should come up zero.
		assert_eq!(PathBuf::from("foo/bar.JS").file_size(), 0);
		assert_eq!(PathBuf::from(env!("CARGO_MANIFEST_DIR")).file_size(), 0);

		// And something we know.
		assert_eq!(PathBuf::from("tests/assets/file.txt").file_size(), 26);
	}

	#[test]
	fn is_executable() {
		// These should come up false.
		assert_eq!(PathBuf::from("foo/bar.JS").is_executable(), false);
		assert_eq!(PathBuf::from("tests/assets/file.txt").is_executable(), false);
		assert_eq!(PathBuf::from(env!("CARGO_MANIFEST_DIR")).is_executable(), false);

		// But this should come up true.
		assert_eq!(PathBuf::from("tests/assets/is-executable.sh").is_executable(), true);
	}

	#[test]
	fn parent_abs() {
		// A known file.
		let file: PathBuf = PathBuf::from("./src/lib.rs");
		assert!(file.is_file());

		// The canonical parent.
		let dir: PathBuf = PathBuf::from("./src")
			.canonicalize()
			.expect("Parent, damn it.");
		assert!(dir.is_dir());

		// The two should match.
		assert_eq!(file.parent_abs().unwrap(), dir);

		// This should also work on directories.
		let dir2: PathBuf = PathBuf::from(".")
			.canonicalize()
			.expect("Parent, damn it.");
		assert!(dir2.is_dir());
		assert_eq!(dir.parent_abs().unwrap(), dir2);
	}

	#[test]
	fn to_path_buf_abs() {
		let path = PathBuf::from("src/lib.rs");
		let canon = path.canonicalize().expect("Canon, damn it!");

		assert_eq!(
			path.to_str().expect("Strings, damn it!"),
			"src/lib.rs",
		);
		assert_eq!(
			path.to_path_buf_abs().to_str().expect("Strings, damn it!"),
			canon.to_str().expect("Strings, damn it!"),
		);
	}
}
