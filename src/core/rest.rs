//! Rest representation
//!
//! A Rest represents a period of silence with a duration.

use std::fmt;

use super::{Duration, Fraction};

/// A musical rest (silence with duration)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rest {
    /// The duration
    duration: Duration,
    /// Offset within the stream (in quarter lengths)
    offset: Fraction,
    /// Whether this rest should be hidden in notation
    hidden: bool,
    /// Full measure rest
    full_measure: bool,
}

impl Rest {
    /// Create a new rest
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            offset: Fraction::new(0, 1),
            hidden: false,
            full_measure: false,
        }
    }

    /// Create a whole rest
    pub fn whole() -> Self {
        Self::new(Duration::whole())
    }

    /// Create a half rest
    pub fn half() -> Self {
        Self::new(Duration::half())
    }

    /// Create a quarter rest
    pub fn quarter() -> Self {
        Self::new(Duration::quarter())
    }

    /// Create an eighth rest
    pub fn eighth() -> Self {
        Self::new(Duration::eighth())
    }

    /// Create a 16th rest
    pub fn sixteenth() -> Self {
        Self::new(Duration::sixteenth())
    }

    /// Create a full-measure rest
    pub fn full_measure(duration: Duration) -> Self {
        let mut rest = Self::new(duration);
        rest.full_measure = true;
        rest
    }

    /// Get the duration
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    /// Get mutable duration
    pub fn duration_mut(&mut self) -> &mut Duration {
        &mut self.duration
    }

    /// Set the duration
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    /// Get the offset
    pub fn offset(&self) -> Fraction {
        self.offset
    }

    /// Set the offset
    pub fn set_offset(&mut self, offset: Fraction) {
        self.offset = offset;
    }

    /// Get the quarter length
    pub fn quarter_length(&self) -> Fraction {
        self.duration.quarter_length()
    }

    /// Check if the rest is hidden
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Set whether the rest is hidden
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    /// Check if this is a full-measure rest
    pub fn is_full_measure(&self) -> bool {
        self.full_measure
    }

    /// Set whether this is a full-measure rest
    pub fn set_full_measure(&mut self, full_measure: bool) {
        self.full_measure = full_measure;
    }

    /// Scale the duration
    pub fn augment_or_diminish(&self, scalar: Fraction) -> Rest {
        let mut scaled = self.clone();
        scaled.duration = self.duration.augment_or_diminish(scalar);
        scaled
    }
}

impl Default for Rest {
    fn default() -> Self {
        Self::quarter()
    }
}

impl fmt::Display for Rest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rest {}", self.duration)
    }
}

impl From<Duration> for Rest {
    fn from(duration: Duration) -> Self {
        Self::new(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_creation() {
        let rest = Rest::quarter();
        assert_eq!(rest.quarter_length(), Fraction::new(1, 1));
    }

    #[test]
    fn test_rest_full_measure() {
        let rest = Rest::full_measure(Duration::whole());
        assert!(rest.is_full_measure());
        assert_eq!(rest.quarter_length(), Fraction::new(4, 1));
    }

    #[test]
    fn test_rest_augment() {
        let rest = Rest::quarter();
        let augmented = rest.augment_or_diminish(Fraction::new(2, 1));
        assert_eq!(augmented.quarter_length(), Fraction::new(2, 1));
    }
}
