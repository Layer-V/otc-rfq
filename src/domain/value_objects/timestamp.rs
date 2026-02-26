//! # Timestamp Value Object
//!
//! DateTime wrapper with domain-specific methods.
//!
//! This module provides the [`Timestamp`] type for representing points in time
//! with nanosecond precision, suitable for trading systems.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::timestamp::Timestamp;
//!
//! let now = Timestamp::now();
//! let later = now.add_secs(60);
//!
//! assert!(later.is_after(&now));
//! ```

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// A UTC timestamp with nanosecond precision.
///
/// Wraps `chrono::DateTime<Utc>` with domain-specific methods for
/// trading operations.
///
/// # Invariants
///
/// - Always in UTC timezone
/// - Nanosecond precision
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::timestamp::Timestamp;
///
/// // Create current timestamp
/// let now = Timestamp::now();
///
/// // Add time
/// let in_one_minute = now.add_secs(60);
///
/// // A future timestamp is not expired
/// assert!(!in_one_minute.is_expired());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// FIX protocol timestamp format: YYYYMMDD-HH:MM:SS.sss
    pub const FIX_FORMAT: &'static str = "%Y%m%d-%H:%M:%S%.3f";

    /// Creates a timestamp for the current moment.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let future = Timestamp::now().add_secs(60);
    /// assert!(!future.is_expired());
    /// ```
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Creates a timestamp from Unix milliseconds.
    ///
    /// # Arguments
    ///
    /// * `millis` - Milliseconds since Unix epoch
    ///
    /// # Returns
    ///
    /// `Some(Timestamp)` if the value is valid, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_millis(1704067200000).unwrap();
    /// assert_eq!(ts.timestamp_millis(), 1704067200000);
    /// ```
    #[must_use]
    pub fn from_millis(millis: i64) -> Option<Self> {
        Utc.timestamp_millis_opt(millis).single().map(Self)
    }

    /// Creates a timestamp from Unix nanoseconds.
    ///
    /// # Arguments
    ///
    /// * `nanos` - Nanoseconds since Unix epoch
    ///
    /// # Returns
    ///
    /// `Some(Timestamp)` if the value is valid, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_nanos(1704067200_000_000_000).unwrap();
    /// assert_eq!(ts.timestamp_nanos(), Some(1704067200_000_000_000));
    /// ```
    #[must_use]
    pub fn from_nanos(nanos: i64) -> Option<Self> {
        let secs = nanos / 1_000_000_000;
        let nsecs = (nanos % 1_000_000_000) as u32;
        Utc.timestamp_opt(secs, nsecs).single().map(Self)
    }

    /// Creates a timestamp from Unix seconds.
    ///
    /// # Arguments
    ///
    /// * `secs` - Seconds since Unix epoch
    ///
    /// # Returns
    ///
    /// `Some(Timestamp)` if the value is valid, `None` otherwise.
    #[must_use]
    pub fn from_secs(secs: i64) -> Option<Self> {
        Utc.timestamp_opt(secs, 0).single().map(Self)
    }

    /// Returns the Unix timestamp in milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_millis(1704067200000).unwrap();
    /// assert_eq!(ts.timestamp_millis(), 1704067200000);
    /// ```
    #[inline]
    #[must_use]
    pub fn timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    /// Returns the Unix timestamp in nanoseconds.
    ///
    /// Returns `None` if the timestamp cannot be represented as i64 nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(1704067200).unwrap();
    /// assert_eq!(ts.timestamp_nanos(), Some(1704067200_000_000_000));
    /// ```
    #[inline]
    #[must_use]
    pub fn timestamp_nanos(&self) -> Option<i64> {
        self.0.timestamp_nanos_opt()
    }

    /// Returns the Unix timestamp in seconds.
    #[inline]
    #[must_use]
    pub fn timestamp_secs(&self) -> i64 {
        self.0.timestamp()
    }

    /// Adds seconds to the timestamp.
    ///
    /// # Arguments
    ///
    /// * `secs` - Number of seconds to add (can be negative)
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(1000).unwrap();
    /// let later = ts.add_secs(60);
    /// assert_eq!(later.timestamp_secs(), 1060);
    /// ```
    #[must_use]
    pub fn add_secs(&self, secs: i64) -> Self {
        Self(self.0 + Duration::seconds(secs))
    }

    /// Adds milliseconds to the timestamp.
    ///
    /// # Arguments
    ///
    /// * `millis` - Number of milliseconds to add (can be negative)
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_millis(1000).unwrap();
    /// let later = ts.add_millis(500);
    /// assert_eq!(later.timestamp_millis(), 1500);
    /// ```
    #[must_use]
    pub fn add_millis(&self, millis: i64) -> Self {
        Self(self.0 + Duration::milliseconds(millis))
    }

    /// Subtracts seconds from the timestamp.
    ///
    /// # Arguments
    ///
    /// * `secs` - Number of seconds to subtract
    #[must_use]
    pub fn sub_secs(&self, secs: i64) -> Self {
        Self(self.0 - Duration::seconds(secs))
    }

    /// Returns true if this timestamp is in the past.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let past = Timestamp::from_secs(0).unwrap();
    /// assert!(past.is_expired());
    ///
    /// let future = Timestamp::now().add_secs(3600);
    /// assert!(!future.is_expired());
    /// ```
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.0 < Utc::now()
    }

    /// Returns true if this timestamp is before another.
    ///
    /// # Arguments
    ///
    /// * `other` - The timestamp to compare against
    #[inline]
    #[must_use]
    pub fn is_before(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    /// Returns true if this timestamp is after another.
    ///
    /// # Arguments
    ///
    /// * `other` - The timestamp to compare against
    #[inline]
    #[must_use]
    pub fn is_after(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    /// Returns the duration between this timestamp and another.
    ///
    /// Returns a positive duration if `other` is after `self`.
    ///
    /// # Arguments
    ///
    /// * `other` - The timestamp to compare against
    #[must_use]
    pub fn duration_until(&self, other: &Self) -> std::time::Duration {
        let diff = other.0 - self.0;
        diff.to_std().unwrap_or(std::time::Duration::ZERO)
    }

    /// Formats the timestamp for FIX protocol.
    ///
    /// Format: `YYYYMMDD-HH:MM:SS.sss`
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_millis(1704067200123).unwrap();
    /// let fix = ts.to_fix_format();
    /// assert!(fix.contains("-"));
    /// assert!(fix.contains(":"));
    /// ```
    #[must_use]
    pub fn to_fix_format(&self) -> String {
        self.0.format(Self::FIX_FORMAT).to_string()
    }

    /// Formats the timestamp as ISO 8601.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(1704067200).unwrap();
    /// let iso = ts.to_iso8601();
    /// assert!(iso.contains("2024-01-01"));
    /// ```
    #[must_use]
    pub fn to_iso8601(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Returns the underlying DateTime.
    #[inline]
    #[must_use]
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

impl Add<std::time::Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: std::time::Duration) -> Self::Output {
        Self(self.0 + Duration::from_std(rhs).unwrap_or(Duration::zero()))
    }
}

impl Sub<std::time::Duration> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: std::time::Duration) -> Self::Output {
        Self(self.0 - Duration::from_std(rhs).unwrap_or(Duration::zero()))
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = std::time::Duration;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        (self.0 - rhs.0)
            .to_std()
            .unwrap_or(std::time::Duration::ZERO)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod construction {
        use super::*;

        #[test]
        fn now_creates_current_time() {
            let before = Utc::now();
            let ts = Timestamp::now();
            let after = Utc::now();

            assert!(ts.0 >= before);
            assert!(ts.0 <= after);
        }

        #[test]
        fn from_millis_works() {
            let ts = Timestamp::from_millis(1704067200000).unwrap();
            assert_eq!(ts.timestamp_millis(), 1704067200000);
        }

        #[test]
        fn from_nanos_works() {
            let ts = Timestamp::from_nanos(1_704_067_200_000_000_000).unwrap();
            assert_eq!(ts.timestamp_nanos(), Some(1_704_067_200_000_000_000));
        }

        #[test]
        fn from_secs_works() {
            let ts = Timestamp::from_secs(1704067200).unwrap();
            assert_eq!(ts.timestamp_secs(), 1704067200);
        }

        #[test]
        fn default_is_now() {
            let before = Utc::now();
            let ts = Timestamp::default();
            let after = Utc::now();

            assert!(ts.0 >= before);
            assert!(ts.0 <= after);
        }
    }

    mod arithmetic {
        use super::*;

        #[test]
        fn add_secs_works() {
            let ts = Timestamp::from_secs(1000).unwrap();
            let later = ts.add_secs(60);
            assert_eq!(later.timestamp_secs(), 1060);
        }

        #[test]
        fn add_millis_works() {
            let ts = Timestamp::from_millis(1000).unwrap();
            let later = ts.add_millis(500);
            assert_eq!(later.timestamp_millis(), 1500);
        }

        #[test]
        fn sub_secs_works() {
            let ts = Timestamp::from_secs(1000).unwrap();
            let earlier = ts.sub_secs(60);
            assert_eq!(earlier.timestamp_secs(), 940);
        }

        #[test]
        fn add_negative_secs() {
            let ts = Timestamp::from_secs(1000).unwrap();
            let earlier = ts.add_secs(-60);
            assert_eq!(earlier.timestamp_secs(), 940);
        }

        #[test]
        fn std_duration_add() {
            let ts = Timestamp::from_secs(1000).unwrap();
            let later = ts + std::time::Duration::from_secs(60);
            assert_eq!(later.timestamp_secs(), 1060);
        }

        #[test]
        fn std_duration_sub() {
            let ts = Timestamp::from_secs(1000).unwrap();
            let earlier = ts - std::time::Duration::from_secs(60);
            assert_eq!(earlier.timestamp_secs(), 940);
        }

        #[test]
        fn timestamp_difference() {
            let ts1 = Timestamp::from_secs(1000).unwrap();
            let ts2 = Timestamp::from_secs(1060).unwrap();
            let diff = ts2 - ts1;
            assert_eq!(diff.as_secs(), 60);
        }
    }

    mod comparison {
        use super::*;

        #[test]
        fn is_expired_past() {
            let past = Timestamp::from_secs(0).unwrap();
            assert!(past.is_expired());
        }

        #[test]
        fn is_expired_future() {
            let future = Timestamp::now().add_secs(3600);
            assert!(!future.is_expired());
        }

        #[test]
        fn is_before() {
            let ts1 = Timestamp::from_secs(1000).unwrap();
            let ts2 = Timestamp::from_secs(2000).unwrap();
            assert!(ts1.is_before(&ts2));
            assert!(!ts2.is_before(&ts1));
        }

        #[test]
        fn is_after() {
            let ts1 = Timestamp::from_secs(1000).unwrap();
            let ts2 = Timestamp::from_secs(2000).unwrap();
            assert!(ts2.is_after(&ts1));
            assert!(!ts1.is_after(&ts2));
        }

        #[test]
        fn duration_until() {
            let ts1 = Timestamp::from_secs(1000).unwrap();
            let ts2 = Timestamp::from_secs(1060).unwrap();
            let duration = ts1.duration_until(&ts2);
            assert_eq!(duration.as_secs(), 60);
        }

        #[test]
        fn ordering() {
            let ts1 = Timestamp::from_secs(1000).unwrap();
            let ts2 = Timestamp::from_secs(2000).unwrap();
            assert!(ts1 < ts2);
            assert!(ts2 > ts1);
        }
    }

    mod formatting {
        use super::*;

        #[test]
        fn to_fix_format() {
            let ts = Timestamp::from_millis(1704067200123).unwrap();
            let fix = ts.to_fix_format();
            // Format: YYYYMMDD-HH:MM:SS.sss
            assert!(fix.contains("-"));
            assert!(fix.contains(":"));
            assert!(fix.contains("."));
        }

        #[test]
        fn to_iso8601() {
            let ts = Timestamp::from_secs(1704067200).unwrap();
            let iso = ts.to_iso8601();
            assert!(iso.contains("T"));
            assert!(iso.ends_with("Z") || iso.contains("+00:00"));
        }

        #[test]
        fn display_format() {
            let ts = Timestamp::from_secs(1704067200).unwrap();
            let display = ts.to_string();
            assert!(display.contains("T"));
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let ts = Timestamp::from_millis(1704067200123).unwrap();
            let json = serde_json::to_string(&ts).unwrap();
            let deserialized: Timestamp = serde_json::from_str(&json).unwrap();
            assert_eq!(ts, deserialized);
        }

        #[test]
        fn serde_iso8601_format() {
            let ts = Timestamp::from_secs(1704067200).unwrap();
            let json = serde_json::to_string(&ts).unwrap();
            // Should serialize as ISO 8601 string
            assert!(json.contains("2024"));
        }
    }

    mod conversion {
        use super::*;

        #[test]
        fn from_datetime() {
            let dt = Utc::now();
            let ts: Timestamp = dt.into();
            assert_eq!(ts.as_datetime(), &dt);
        }

        #[test]
        fn into_datetime() {
            let ts = Timestamp::now();
            let dt: DateTime<Utc> = ts.into();
            assert_eq!(&dt, ts.as_datetime());
        }
    }
}
