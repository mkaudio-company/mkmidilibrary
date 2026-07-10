//! Rest representation
//!
//! A Rest represents a period of silence with a duration.

use std::fmt;

use super::{Duration, Fraction};

/// Whether a rest fills its entire enclosing measure. Unlike a plain
/// bool, this mirrors music21's 4-state `Rest.fullMeasure`: `Auto` (the
/// default) determines this from context — comparing the rest's own
/// duration against its enclosing measure's duration (see
/// `Rest::is_full_measure_in_context`) — rather than being explicitly set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FullMeasureRest {
    /// Explicitly not a full-measure rest, regardless of duration.
    False,
    /// Explicitly a full-measure rest, regardless of duration.
    True,
    /// Auto-detect from context: a full-measure rest if its own duration
    /// equals its enclosing measure's duration.
    #[default]
    Auto,
    /// Always display as a single full-measure (whole) rest symbol
    /// regardless of the rest's own notated duration (e.g. a
    /// whole-measure rest in 5/8 time is still drawn as one whole-rest
    /// symbol rather than a tied group).
    Always,
}

/// A musical rest (silence with duration)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rest {
    /// The duration
    duration: Duration,
    /// Offset within the stream (in quarter lengths)
    offset: Fraction,
    /// Whether this rest should be hidden in notation
    hidden: bool,
    /// Full measure rest mode (see `FullMeasureRest`)
    full_measure: FullMeasureRest,
    /// Vertical display offset (in staff steps/lines) from the rest's
    /// default vertical position — used e.g. to avoid collisions between
    /// rests in different voices sharing a staff.
    step_shift: i8,
}

impl Rest {
    /// Create a new rest
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            offset: Fraction::new(0, 1),
            hidden: false,
            full_measure: FullMeasureRest::Auto,
            step_shift: 0,
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

    /// Create an explicitly full-measure rest.
    pub fn full_measure(duration: Duration) -> Self {
        let mut rest = Self::new(duration);
        rest.full_measure = FullMeasureRest::True;
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

    /// Check if this is an explicitly full-measure rest (`True` or
    /// `Always`), without any measure-duration context. An `Auto` rest
    /// (the default) reports `false` here regardless of its own duration —
    /// use `is_full_measure_in_context` when a measure duration is
    /// available to correctly resolve `Auto`.
    pub fn is_full_measure(&self) -> bool {
        matches!(self.full_measure, FullMeasureRest::True | FullMeasureRest::Always)
    }

    /// Set whether this is an explicitly full-measure rest (`True`/`False`).
    /// For the full 4-state control (including `Auto`/`Always`), use
    /// `set_full_measure_mode`.
    pub fn set_full_measure(&mut self, full_measure: bool) {
        self.full_measure = if full_measure {
            FullMeasureRest::True
        } else {
            FullMeasureRest::False
        };
    }

    /// Get the 4-state full-measure mode.
    pub fn full_measure_mode(&self) -> FullMeasureRest {
        self.full_measure
    }

    /// Set the 4-state full-measure mode.
    pub fn set_full_measure_mode(&mut self, mode: FullMeasureRest) {
        self.full_measure = mode;
    }

    /// Resolve whether this rest should be notated as filling a measure of
    /// the given duration: `True`/`False` are explicit regardless of
    /// `measure_duration`; `Always` is always true; `Auto` (the default)
    /// compares this rest's own duration to `measure_duration`.
    pub fn is_full_measure_in_context(&self, measure_duration: Fraction) -> bool {
        match self.full_measure {
            FullMeasureRest::True | FullMeasureRest::Always => true,
            FullMeasureRest::False => false,
            FullMeasureRest::Auto => self.quarter_length() == measure_duration,
        }
    }

    /// Get the vertical display offset (in staff steps).
    pub fn step_shift(&self) -> i8 {
        self.step_shift
    }

    /// Set the vertical display offset (in staff steps).
    pub fn set_step_shift(&mut self, shift: i8) {
        self.step_shift = shift;
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
    fn test_rest_step_shift() {
        let mut rest = Rest::quarter();
        assert_eq!(rest.step_shift(), 0);
        rest.set_step_shift(2);
        assert_eq!(rest.step_shift(), 2);
    }

    #[test]
    fn test_rest_full_measure_default_is_auto() {
        let rest = Rest::whole();
        assert_eq!(rest.full_measure_mode(), FullMeasureRest::Auto);
        // Without context, an Auto rest reports false...
        assert!(!rest.is_full_measure());
        // ...but with a matching measure duration, Auto resolves to true.
        assert!(rest.is_full_measure_in_context(Fraction::new(4, 1)));
        // And to false against a non-matching measure duration (e.g. 3/4 time).
        assert!(!rest.is_full_measure_in_context(Fraction::new(3, 1)));
    }

    #[test]
    fn test_rest_full_measure_explicit_states() {
        let mut rest = Rest::quarter();

        rest.set_full_measure_mode(FullMeasureRest::True);
        assert!(rest.is_full_measure());
        assert!(rest.is_full_measure_in_context(Fraction::new(3, 1))); // explicit, ignores context

        rest.set_full_measure_mode(FullMeasureRest::False);
        assert!(!rest.is_full_measure());
        assert!(!rest.is_full_measure_in_context(Fraction::new(1, 1))); // explicit false even if duration matches

        rest.set_full_measure_mode(FullMeasureRest::Always);
        assert!(rest.is_full_measure());
        assert!(rest.is_full_measure_in_context(Fraction::new(100, 1))); // always true regardless
    }

    #[test]
    fn test_rest_augment() {
        let rest = Rest::quarter();
        let augmented = rest.augment_or_diminish(Fraction::new(2, 1));
        assert_eq!(augmented.quarter_length(), Fraction::new(2, 1));
    }
}
