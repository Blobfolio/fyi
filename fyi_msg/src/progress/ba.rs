/*!
# FYI Msg - Progless Before/After
*/

use std::num::NonZeroU64;



#[derive(Debug, Copy, Clone)]
/// # Before and After.
///
/// This is a potentially useful companion to [`Progless`] that tracks an
/// arbitrary non-zero before and after state. It was created to make it easire
/// to track before/after file sizes from minification-type tasks, but it
/// doesn't ascribe any particular meaning to the data it holds.
///
/// ## Examples
///
/// Usage is as simple as:
///
/// ```no_run
/// use fyi_msg::BeforeAfter;
///
/// let mut ba = BeforeAfter::start(123_u64);
///
/// // Do some stuff.
///
/// ba.stop(50_u64);
/// ```
///
/// Once before and after are set, you can use the getter methods [`BeforeAfter::before`]
/// and [`BeforeAfter::after`] to obtain the values.
///
/// For relative changes where `after` is expected to be smaller than `before`,
/// there is [`BeforeAfter::less`] and [`BeforeAfter::less_percent`] to obtain
/// the relative difference.
///
/// For cases where `after` is expected to be larger, use [`BeforeAfter::more`]
/// and [`BeforeAfter::more_percent`] instead.
pub struct BeforeAfter {
	before: Option<NonZeroU64>,
	after: Option<NonZeroU64>,
}

impl From<(u64, u64)> for BeforeAfter {
	fn from(src: (u64, u64)) -> Self {
		Self {
			before: NonZeroU64::new(src.0),
			after: NonZeroU64::new(src.1),
		}
	}
}

impl BeforeAfter {
	#[must_use]
	#[inline]
	/// # New Instance: Set Before.
	///
	/// This creates a new instance with the defined starting point.
	///
	/// A `before` value of `0_u64` is equivalent to `None`. The instance will
	/// still be created, but the difference methods won't return any values.
	pub const fn start(before: u64) -> Self {
		Self {
			before: NonZeroU64::new(before),
			after: None,
		}
	}

	#[inline]
	/// # Finish Instance: Set After.
	///
	/// This sets the `after` value of an existing instance, closing it out.
	///
	/// An `after` value of `0_u64` is equivalent to `None`, meaning the
	/// difference methods won't return any values.
	pub fn stop(&mut self, after: u64) {
		self.after = NonZeroU64::new(after);
	}

	#[must_use]
	#[inline]
	/// # Get Before.
	///
	/// Return the `before` value if non-zero, otherwise `None`.
	pub const fn before(&self) -> Option<NonZeroU64> { self.before }

	#[must_use]
	#[inline]
	/// # Get After.
	///
	/// Return the `after` value if non-zero, otherwise `None`.
	pub const fn after(&self) -> Option<NonZeroU64> { self.after }

	#[must_use]
	/// # Get Difference (After < Before).
	///
	/// If the after state is expected to be smaller than the before state,
	/// return the difference. If either state is unset/zero, or after is
	/// larger, `None` is returned.
	pub const fn less(&self) -> Option<NonZeroU64> {
		if let Some(b) = self.before {
			if let Some(a) = self.after {
				return NonZeroU64::new(b.get().saturating_sub(a.get()));
			}
		}
		None
	}

	#[must_use]
	/// # Percentage Difference (After < Before).
	///
	/// This is the same as [`BeforeAfter::less`], but returns a percentage of
	/// the difference over `before`.
	pub fn less_percent(&self) -> Option<f64> {
		self.less().and_then(|l| dactyl::int_div_float(l.get(), self.before?.get()))
	}

	#[must_use]
	/// # Get Difference (After > Before).
	///
	/// If the after state is expected to be larger than the before state,
	/// return the difference. If either state is unset/zero, or after is
	/// smaller, `None` is returned.
	pub const fn more(&self) -> Option<NonZeroU64> {
		if let Some(b) = self.before {
			if let Some(a) = self.after {
				return NonZeroU64::new(a.get().saturating_sub(b.get()));
			}
		}
		None
	}

	#[must_use]
	/// # Percentage Difference (After > Before).
	///
	/// This is the same as [`BeforeAfter::more`], but returns a percentage of
	/// the difference over `before`.
	pub fn more_percent(&self) -> Option<f64> {
		self.more().and_then(|m| dactyl::int_div_float(m.get(), self.before?.get()))
	}
}
