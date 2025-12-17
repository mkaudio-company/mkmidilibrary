//! Time signature / meter representation

use std::fmt;

use crate::core::Fraction;

/// A time signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeSignature {
    /// Numerator (number of beats)
    numerator: u8,
    /// Denominator (beat unit as power of 2)
    denominator: u8,
}

impl TimeSignature {
    /// Create a new time signature
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Create common time (4/4)
    pub fn common_time() -> Self {
        Self::new(4, 4)
    }

    /// Create cut time (2/2)
    pub fn cut_time() -> Self {
        Self::new(2, 2)
    }

    /// Create 3/4 time
    pub fn three_four() -> Self {
        Self::new(3, 4)
    }

    /// Create 6/8 time
    pub fn six_eight() -> Self {
        Self::new(6, 8)
    }

    /// Get the numerator
    pub fn numerator(&self) -> u8 {
        self.numerator
    }

    /// Get the denominator
    pub fn denominator(&self) -> u8 {
        self.denominator
    }

    /// Get the bar duration in quarter lengths
    pub fn bar_duration(&self) -> Fraction {
        // Duration = numerator * (4 / denominator)
        Fraction::new(self.numerator as i64 * 4, self.denominator as i64)
    }

    /// Get the beat duration in quarter lengths
    pub fn beat_duration(&self) -> Fraction {
        Fraction::new(4, self.denominator as i64)
    }

    /// Get the number of beats per bar
    pub fn beats_per_bar(&self) -> u8 {
        if self.is_compound() {
            self.numerator / 3
        } else {
            self.numerator
        }
    }

    /// Check if this is compound meter (divisible by 3)
    pub fn is_compound(&self) -> bool {
        self.numerator > 3 && self.numerator % 3 == 0
    }

    /// Check if this is simple meter
    pub fn is_simple(&self) -> bool {
        !self.is_compound()
    }

    /// Check if this is duple meter (2 or 6 beats)
    pub fn is_duple(&self) -> bool {
        let beats = self.beats_per_bar();
        beats == 2
    }

    /// Check if this is triple meter (3 or 9 beats)
    pub fn is_triple(&self) -> bool {
        let beats = self.beats_per_bar();
        beats == 3
    }

    /// Check if this is quadruple meter (4 or 12 beats)
    pub fn is_quadruple(&self) -> bool {
        let beats = self.beats_per_bar();
        beats == 4
    }

    /// Get the beat groupings for beaming
    pub fn beat_groups(&self) -> Vec<Fraction> {
        let beat = self.beat_duration();
        (0..self.beats_per_bar())
            .map(|i| beat * Fraction::from(i as i64))
            .collect()
    }

    /// Check if this is an additive meter (like 5/8 or 7/8)
    pub fn is_additive(&self) -> bool {
        matches!(self.numerator, 5 | 7 | 11 | 13)
    }

    /// Get common additive groupings (e.g., 5/8 = 3+2 or 2+3)
    pub fn additive_groupings(&self) -> Vec<Vec<u8>> {
        match self.numerator {
            5 => vec![vec![3, 2], vec![2, 3]],
            7 => vec![vec![4, 3], vec![3, 4], vec![2, 2, 3], vec![3, 2, 2]],
            11 => vec![vec![4, 4, 3], vec![3, 4, 4]],
            _ => vec![],
        }
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self::common_time()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_signature_creation() {
        let ts = TimeSignature::new(4, 4);
        assert_eq!(ts.numerator(), 4);
        assert_eq!(ts.denominator(), 4);
    }

    #[test]
    fn test_bar_duration() {
        assert_eq!(
            TimeSignature::common_time().bar_duration(),
            Fraction::new(4, 1)
        );
        assert_eq!(
            TimeSignature::three_four().bar_duration(),
            Fraction::new(3, 1)
        );
        assert_eq!(
            TimeSignature::six_eight().bar_duration(),
            Fraction::new(3, 1)
        );
    }

    #[test]
    fn test_compound_meter() {
        assert!(TimeSignature::six_eight().is_compound());
        assert!(!TimeSignature::three_four().is_compound());
        assert!(!TimeSignature::common_time().is_compound());
    }

    #[test]
    fn test_beats_per_bar() {
        assert_eq!(TimeSignature::common_time().beats_per_bar(), 4);
        assert_eq!(TimeSignature::three_four().beats_per_bar(), 3);
        assert_eq!(TimeSignature::six_eight().beats_per_bar(), 2); // Compound duple
    }

    #[test]
    fn test_meter_type() {
        assert!(TimeSignature::cut_time().is_duple());
        assert!(TimeSignature::three_four().is_triple());
        assert!(TimeSignature::common_time().is_quadruple());
    }
}
