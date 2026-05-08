use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// An unsigned duration with millisecond precision.
///
/// Maximum duration is about 48 days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct ShortDuration {
    millis: u32,
}

impl ShortDuration {
    /// Creates a new `ShortDuration` from milliseconds.
    #[must_use]
    pub const fn from_millis(millis: u32) -> Self {
        Self { millis }
    }

    /// Creates a new [`ShortDuration`] from `d` if it fits. Otherwise, returns [`None`].
    #[must_use]
    pub fn from_std(d: std::time::Duration) -> Option<Self> {
        Self::try_from(d).ok()
    }

    /// Creates a new `ShortDuration` from milliseconds.
    #[must_use]
    pub const fn from_seconds(secs: u32) -> Self {
        Self {
            millis: secs.saturating_mul(1000),
        }
    }

    /// Returns the total number of milliseconds in this duration.
    #[must_use]
    pub fn as_millis(&self) -> u32 {
        self.millis
    }

    /// Returns the whole seconds part of this duration.
    #[must_use]
    pub fn as_secs(&self) -> u32 {
        self.millis / 1000
    }

    /// Converts `self` to [`std::time::Duration`].
    /// This conversion is not free.
    #[must_use]
    pub fn as_std(&self) -> std::time::Duration {
        std::time::Duration::from_millis(u64::from(self.as_millis()))
    }

    /// Returns the milliseconds part not making up a full second.
    #[must_use]
    pub fn subsec_millis(&self) -> u32 {
        self.millis.rem_euclid(1000)
    }

    /// Returns the number of seconds
    ///
    /// If the duration of `self` is large (> 2.3 hrs), this will cause a loss of
    /// precision.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "Precision loss documented and acceptable"
    )]
    pub fn as_secs_f32(&self) -> f32 {
        self.millis as f32 / 1000.0
    }

    /// Checked add – returns None if overflow occurs.
    pub fn checked_add(self, other: ShortDuration) -> Option<Self> {
        self.millis.checked_add(other.millis).map(Self::from_millis)
    }

    /// Checked sub – returns None if underflow occurs.
    pub fn checked_sub(self, other: ShortDuration) -> Option<Self> {
        self.millis.checked_sub(other.millis).map(Self::from_millis)
    }

    /// Subtraction, saturating at u32 bounds instead of overflowing
    #[must_use]
    pub fn saturating_sub(self, other: ShortDuration) -> Self {
        Self::from_millis(self.millis.saturating_sub(other.millis))
    }

    /// Addition, saturating at u32 bounds instead of overflowing
    #[must_use]
    pub fn saturating_add(self, other: ShortDuration) -> Self {
        Self::from_millis(self.millis.saturating_add(other.millis))
    }

    /// Multiplication, saturating at u32 bounds instead of overflowing
    #[must_use]
    pub fn saturating_mul(self, rhs: u32) -> Self {
        Self::from_millis(self.millis.saturating_mul(rhs))
    }

    /// Division, saturating at u32 bounds instead of overflowing
    #[must_use]
    pub fn saturating_div(self, rhs: u32) -> Self {
        Self::from_millis(self.millis.saturating_div(rhs))
    }

    /// Absolute zero duration
    pub const ZERO: ShortDuration = ShortDuration { millis: 0 };
}

impl Default for ShortDuration {
    fn default() -> Self {
        ShortDuration::ZERO
    }
}

#[derive(Debug)]
pub struct OutOfBoundsError;

impl TryFrom<std::time::Duration> for ShortDuration {
    type Error = OutOfBoundsError;

    fn try_from(d: std::time::Duration) -> Result<Self, Self::Error> {
        let millis = u32::try_from(d.as_millis());

        if let Ok(millis) = millis {
            Ok(Self::from_millis(millis))
        } else {
            Err(OutOfBoundsError)
        }
    }
}

impl From<ShortDuration> for std::time::Duration {
    fn from(sd: ShortDuration) -> Self {
        sd.as_std()
    }
}

impl Add for ShortDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_millis(self.millis.saturating_add(rhs.millis))
    }
}

impl Sub for ShortDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_millis(self.millis.saturating_sub(rhs.millis))
    }
}

impl Mul<u32> for ShortDuration {
    type Output = ShortDuration;

    fn mul(self, rhs: u32) -> Self::Output {
        ShortDuration::from_millis(self.as_millis() * rhs)
    }
}

impl Div<u32> for ShortDuration {
    type Output = ShortDuration;

    fn div(self, rhs: u32) -> Self::Output {
        ShortDuration::from_millis(self.as_millis() / rhs)
    }
}

impl AddAssign for ShortDuration {
    fn add_assign(&mut self, rhs: Self) {
        self.millis += rhs.millis;
    }
}

impl SubAssign for ShortDuration {
    fn sub_assign(&mut self, rhs: Self) {
        self.millis -= rhs.millis;
    }
}
