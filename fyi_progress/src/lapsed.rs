/*!
# FYI Progress: Lapsed.
*/

use std::borrow::Cow;
use fyi_msg::{
	Timestamp,
	utility,
};



#[must_use]
/// Elapsed Short.
pub fn compact(num: u32) -> Cow<'static, [u8]> {
	static ZERO: &[u8] = b"00:00:00";

	if 0 == num {
		Cow::Borrowed(ZERO)
	}
	// Under a day means 3 pairs.
	else if num < 86400 {
		Cow::Owned({
			let c = secs_chunks(num);
			let mut buf: Vec<u8> = ZERO.to_vec();
			utility::slice_swap(&mut buf[0..2], Timestamp::time_format_dd(c[1]));
			utility::slice_swap(&mut buf[3..5], Timestamp::time_format_dd(c[2]));
			utility::slice_swap(&mut buf[6..8], Timestamp::time_format_dd(c[3]));
			buf
		})
	}
	// Under 60 days is 4 pairs, all of which can be handled by our double-
	// digit time helper.
	else if num < 5_184_000 {
		Cow::Owned({
			let c = secs_chunks(num);
			let mut buf: Vec<u8> = Vec::from(&b"00:00:00:00"[..]);
			utility::slice_swap(&mut buf[0..2], Timestamp::time_format_dd(c[0]));
			utility::slice_swap(&mut buf[3..5], Timestamp::time_format_dd(c[1]));
			utility::slice_swap(&mut buf[6..8], Timestamp::time_format_dd(c[2]));
			utility::slice_swap(&mut buf[9..11], Timestamp::time_format_dd(c[3]));
			buf
		})
	}
	// Above 60 days, we need to write the day portion directly, but the rest
	// can still be handled like above.
	else {
		Cow::Owned({
			let c = secs_chunks(num);
			let mut buf: Vec<u8> = Vec::with_capacity(11);

			// Write the days.
			itoa::write(&mut buf, c[0]).unwrap();

			// Attach the rest of the skeleton.
			buf.push(b':');
			buf.extend_from_slice(ZERO);

			// Overwrite the particulars.
			utility::slice_swap(&mut buf[3..5], Timestamp::time_format_dd(c[1]));
			utility::slice_swap(&mut buf[6..8], Timestamp::time_format_dd(c[2]));
			utility::slice_swap(&mut buf[9..11], Timestamp::time_format_dd(c[3]));

			buf
		})
	}
}

#[must_use]
/// Elapsed Chunks
///
/// Return a fixed array containing the number of days, hours,
/// minutes, and seconds.
pub fn secs_chunks(num: u32) -> [u32; 4] {
	let mut out: [u32; 4] = [0, 0, 0, num];

	// Days.
	if out[3] >= 86400 {
		out[0] = num_integer::div_floor(out[3], 86400);
		out[3] -= out[0] * 86400;
	}

	// Hours.
	if out[3] >= 3600 {
		out[1] = num_integer::div_floor(out[3], 3600);
		out[3] -= out[1] * 3600;
	}

	// Minutes.
	if out[3] >= 60 {
		out[2] = num_integer::div_floor(out[3], 60);
		out[3] -= out[2] * 60;
	}

	out
}
