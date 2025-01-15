/// A list specifying general categories of Interval errors.
#[derive(Debug, PartialEq)]
pub enum IntervalError {
    /// Start is not less than or equal to end
    StartEndRangeInvalid,
    /// Two intervals to be merged do not overlap
    NonOverlappingInterval,
}

/// A closed-interval [`start`, `end`] type used for representing a range of
/// values between `start` and `end` inclusively.
///
/// # Examples
///
/// You can create an `Interval` using `new`.
///
/// ```rust
/// let interval = Interval::new(1, 10).unwrap();
/// assert_eq!(interval.start, 1);
/// assert_eq!(interval.end, 10);
/// ```
#[derive(Debug, PartialEq)]
pub struct Interval<T> {
    pub start: T,
    pub end: T,
}

impl<T: Copy + PartialOrd> Interval<T> {
    /// Creates a new `Interval` set to `start` and `end`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let interval = Interval::new(1, 10).unwrap();
    /// assert_eq!(interval.start, 1);
    /// assert_eq!(interval.end, 10);
    /// ```
    pub fn new(start: T, end: T) -> Result<Self, IntervalError> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(IntervalError::StartEndRangeInvalid)
        }
    }

    /// Checks if two intervals overlap. Overlapping intervals have at least
    /// one point in common.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let a = Interval::new(1, 3).unwrap();
    /// let b = Interval::new(3, 5).unwrap();
    /// assert_eq!(a.overlaps(&b), true);
    /// assert_eq!(b.overlaps(&a), true);
    /// ```
    ///
    /// ```rust
    /// let a = Interval::new(1, 5).unwrap();
    /// let b = Interval::new(2, 4).unwrap();
    /// assert_eq!(a.overlaps(&b), true);
    /// assert_eq!(b.overlaps(&a), true);
    /// ```
    ///
    /// ```rust
    /// let a = Interval::new(1, 3).unwrap();
    /// let b = Interval::new(4, 6).unwrap();
    /// assert_eq!(a.overlaps(&b), false);
    /// assert_eq!(b.overlaps(&a), true);
    /// ```
    pub fn overlaps(&self, other: &Interval<T>) -> bool {
        self.end >= other.start
    }

    /// Merges two intervals returning a new `Interval`.
    ///
    /// The merged `Interval` range includes the union of ranges from each
    /// `Interval`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let a = Interval::new(1, 3).unwrap();
    /// let b = Interval::new(3, 5).unwrap();
    /// let c = a.merge(&b).unwrap();
    /// assert_eq!(c.start, 1);
    /// assert_eq!(c.end, 5);
    /// ```
    pub fn merge(&self, other: &Self) -> Result<Self, IntervalError> {
        if self.overlaps(other) {
            Ok(Self {
                start: self.start,
                end: other.end,
            })
        } else {
            Err(IntervalError::NonOverlappingInterval)
        }
    }
}

use std::fmt;
impl<T: fmt::Display> fmt::Display for Interval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}

use std::cmp::Ordering;
impl<T: PartialEq + PartialOrd> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.end < other.start {
            Some(Ordering::Less)
        } else if self.start > other.end {
            Some(Ordering::Greater)
        } else {
            None // Intervals overlap
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_good() {
        let intervals =
            vec![(-2, -1), (-1, -1), (-1, 0), (0, 0), (0, 1), (1, 1), (1, 2)];

        for (start, end) in intervals {
            let interval = Interval::new(start, end).unwrap();
            assert_eq!(interval.start, start);
            assert_eq!(interval.end, end);
        }
    }

    #[test]
    fn new_bad() {
        let intervals = vec![(-1, -2), (0, -1), (1, 0), (2, 1)];

        for (start, end) in intervals {
            let error = Interval::new(start, end).unwrap_err();
            assert_eq!(IntervalError::StartEndRangeInvalid, error);
        }
    }

    #[test]
    fn overlapping() {
        let a = Interval::new(0, 0).unwrap();
        let b = Interval::new(0, 0).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));

        let a = Interval::new(-1, 0).unwrap();
        let b = Interval::new(0, 0).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));

        let a = Interval::new(-1, 0).unwrap();
        let b = Interval::new(0, 1).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));

        let a = Interval::new(0, 0).unwrap();
        let b = Interval::new(0, 1).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));

        let a = Interval::new(0, 1).unwrap();
        let b = Interval::new(1, 1).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));

        let a = Interval::new(0, 1).unwrap();
        let b = Interval::new(1, 2).unwrap();
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn non_overlapping() {
        let a = Interval::new(-4, -3).unwrap();
        let b = Interval::new(-2, -1).unwrap();
        assert!(!a.overlaps(&b));

        let a = Interval::new(-3, -2).unwrap();
        let b = Interval::new(-1, 0).unwrap();
        assert!(!a.overlaps(&b));

        let a = Interval::new(-2, -1).unwrap();
        let b = Interval::new(0, 1).unwrap();
        assert!(!a.overlaps(&b));

        let a = Interval::new(-1, 0).unwrap();
        let b = Interval::new(1, 2).unwrap();
        assert!(!a.overlaps(&b));

        let a = Interval::new(0, 1).unwrap();
        let b = Interval::new(2, 3).unwrap();
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn merge_good() {
        let a = Interval::new(0, 0).unwrap();
        let b = Interval::new(0, 0).unwrap();
        let c = a.merge(&b).unwrap();
        assert_eq!(c.start, 0);
        assert_eq!(c.end, 0);

        let a = Interval::new(-3, -2).unwrap();
        let b = Interval::new(-2, -1).unwrap();
        let c = a.merge(&b).unwrap();
        assert_eq!(c.start, -3);
        assert_eq!(c.end, -1);

        let a = Interval::new(-2, -1).unwrap();
        let b = Interval::new(-1, 0).unwrap();
        let c = a.merge(&b).unwrap();
        assert_eq!(c.start, -2);
        assert_eq!(c.end, 0);

        let a = Interval::new(-1, 0).unwrap();
        let b = Interval::new(0, 1).unwrap();
        let c = a.merge(&b).unwrap();
        assert_eq!(c.start, -1);
        assert_eq!(c.end, 1);

        let a = Interval::new(0, 1).unwrap();
        let b = Interval::new(1, 2).unwrap();
        let c = a.merge(&b).unwrap();
        assert_eq!(c.start, 0);
        assert_eq!(c.end, 2);

        let a = Interval::new(1, 2).unwrap();
        let b = Interval::new(2, 3).unwrap();
        let c = a.merge(&b).unwrap();
        assert_eq!(c.start, 1);
        assert_eq!(c.end, 3);
    }

    #[test]
    fn merge_bad() {
        let a = Interval::new(-1, -1).unwrap();
        let b = Interval::new(0, 0).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);

        let a = Interval::new(0, 0).unwrap();
        let b = Interval::new(1, 1).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);

        let a = Interval::new(-3, -2).unwrap();
        let b = Interval::new(-1, 0).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);

        let a = Interval::new(-2, -1).unwrap();
        let b = Interval::new(0, 1).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);

        let a = Interval::new(-1, 0).unwrap();
        let b = Interval::new(1, 2).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);

        let a = Interval::new(0, 1).unwrap();
        let b = Interval::new(2, 3).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);

        let a = Interval::new(1, 2).unwrap();
        let b = Interval::new(3, 4).unwrap();
        let c = a.merge(&b).unwrap_err();
        assert_eq!(IntervalError::NonOverlappingInterval, c);
    }
}
