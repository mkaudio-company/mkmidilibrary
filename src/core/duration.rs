//! Duration and rhythm representation
//!
//! Duration represents the temporal length of musical events,
//! measured in quarter note lengths.

use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

use num::{One, Zero};

use super::{Fraction, ParseError};

/// Duration type (note value)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationType {
    /// Maxima (8 whole notes)
    Maxima,
    /// Longa (4 whole notes)
    Longa,
    /// Breve / Double whole note (2 whole notes)
    Breve,
    /// Whole note / Semibreve
    Whole,
    /// Half note / Minim
    Half,
    /// Quarter note / Crotchet
    Quarter,
    /// Eighth note / Quaver
    Eighth,
    /// 16th note / Semiquaver
    N16th,
    /// 32nd note / Demisemiquaver
    N32nd,
    /// 64th note / Hemidemisemiquaver
    N64th,
    /// 128th note
    N128th,
    /// 256th note
    N256th,
    /// Zero duration (grace note)
    Zero,
}

impl DurationType {
    /// Get the quarter note length for this duration type (without dots)
    pub fn quarter_length(&self) -> Fraction {
        match self {
            DurationType::Maxima => Fraction::new(32, 1),
            DurationType::Longa => Fraction::new(16, 1),
            DurationType::Breve => Fraction::new(8, 1),
            DurationType::Whole => Fraction::new(4, 1),
            DurationType::Half => Fraction::new(2, 1),
            DurationType::Quarter => Fraction::one(),
            DurationType::Eighth => Fraction::new(1, 2),
            DurationType::N16th => Fraction::new(1, 4),
            DurationType::N32nd => Fraction::new(1, 8),
            DurationType::N64th => Fraction::new(1, 16),
            DurationType::N128th => Fraction::new(1, 32),
            DurationType::N256th => Fraction::new(1, 64),
            DurationType::Zero => Fraction::zero(),
        }
    }

    /// Get duration type from quarter length (returns closest match)
    pub fn from_quarter_length(ql: Fraction) -> Option<DurationType> {
        if ql == Fraction::zero() {
            return Some(DurationType::Zero);
        }

        let types = [
            DurationType::Maxima,
            DurationType::Longa,
            DurationType::Breve,
            DurationType::Whole,
            DurationType::Half,
            DurationType::Quarter,
            DurationType::Eighth,
            DurationType::N16th,
            DurationType::N32nd,
            DurationType::N64th,
            DurationType::N128th,
            DurationType::N256th,
        ];

        for t in types {
            if t.quarter_length() == ql {
                return Some(t);
            }
        }

        None
    }

    /// Get the standard name
    pub fn name(&self) -> &'static str {
        match self {
            DurationType::Maxima => "maxima",
            DurationType::Longa => "longa",
            DurationType::Breve => "breve",
            DurationType::Whole => "whole",
            DurationType::Half => "half",
            DurationType::Quarter => "quarter",
            DurationType::Eighth => "eighth",
            DurationType::N16th => "16th",
            DurationType::N32nd => "32nd",
            DurationType::N64th => "64th",
            DurationType::N128th => "128th",
            DurationType::N256th => "256th",
            DurationType::Zero => "zero",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<DurationType, ParseError> {
        match s.to_lowercase().as_str() {
            "maxima" => Ok(DurationType::Maxima),
            "longa" | "long" => Ok(DurationType::Longa),
            "breve" | "double whole" | "double-whole" => Ok(DurationType::Breve),
            "whole" | "1" | "semibreve" => Ok(DurationType::Whole),
            "half" | "2" | "minim" => Ok(DurationType::Half),
            "quarter" | "4" | "crotchet" => Ok(DurationType::Quarter),
            "eighth" | "8" | "8th" | "quaver" => Ok(DurationType::Eighth),
            "16th" | "16" | "semiquaver" => Ok(DurationType::N16th),
            "32nd" | "32" | "demisemiquaver" => Ok(DurationType::N32nd),
            "64th" | "64" | "hemidemisemiquaver" => Ok(DurationType::N64th),
            "128th" | "128" => Ok(DurationType::N128th),
            "256th" | "256" => Ok(DurationType::N256th),
            "zero" | "0" | "grace" => Ok(DurationType::Zero),
            _ => Err(ParseError::InvalidDurationType(s.to_string())),
        }
    }
}

impl fmt::Display for DurationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A tuplet modification to duration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tuplet {
    /// Number of notes in the tuplet group
    pub actual: u8,
    /// Number of notes the tuplet replaces
    pub normal: u8,
    /// Duration type of the tuplet notes
    pub duration_type: Option<DurationType>,
}

impl Tuplet {
    /// Create a new tuplet
    pub fn new(actual: u8, normal: u8) -> Self {
        Self {
            actual,
            normal,
            duration_type: None,
        }
    }

    /// Create a triplet (3 in the space of 2)
    pub fn triplet() -> Self {
        Self::new(3, 2)
    }

    /// Create a duplet (2 in the space of 3)
    pub fn duplet() -> Self {
        Self::new(2, 3)
    }

    /// Create a quintuplet (5 in the space of 4)
    pub fn quintuplet() -> Self {
        Self::new(5, 4)
    }

    /// Get the multiplier for this tuplet
    pub fn multiplier(&self) -> Fraction {
        Fraction::new(self.normal as i64, self.actual as i64)
    }
}

impl Default for Tuplet {
    fn default() -> Self {
        Self::triplet()
    }
}

/// A musical duration measured in quarter note lengths
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duration {
    /// The quarter note length
    quarter_length: Fraction,
    /// The base duration type (if known)
    type_: Option<DurationType>,
    /// Number of dots
    dots: u8,
    /// Tuplets applied to this duration
    tuplets: Vec<Tuplet>,
    /// Whether type and quarter_length are linked
    linked: bool,
}

impl Duration {
    /// Create a new duration from quarter note length
    pub fn from_quarter_length(ql: impl Into<Fraction>) -> Self {
        let ql = ql.into();
        let (type_, dots) = Self::infer_type_and_dots(ql);

        Self {
            quarter_length: ql,
            type_,
            dots,
            tuplets: Vec::new(),
            linked: true,
        }
    }

    /// Create a duration from type and dots
    pub fn from_type(type_: DurationType, dots: u8) -> Self {
        let quarter_length = Self::calculate_quarter_length(type_, dots, &[]);

        Self {
            quarter_length,
            type_: Some(type_),
            dots,
            tuplets: Vec::new(),
            linked: true,
        }
    }

    /// Create a zero duration (for grace notes)
    pub fn zero() -> Self {
        Self::from_type(DurationType::Zero, 0)
    }

    /// Create a whole note duration
    pub fn whole() -> Self {
        Self::from_type(DurationType::Whole, 0)
    }

    /// Create a half note duration
    pub fn half() -> Self {
        Self::from_type(DurationType::Half, 0)
    }

    /// Create a quarter note duration
    pub fn quarter() -> Self {
        Self::from_type(DurationType::Quarter, 0)
    }

    /// Create an eighth note duration
    pub fn eighth() -> Self {
        Self::from_type(DurationType::Eighth, 0)
    }

    /// Create a 16th note duration
    pub fn sixteenth() -> Self {
        Self::from_type(DurationType::N16th, 0)
    }

    /// Get the quarter note length
    pub fn quarter_length(&self) -> Fraction {
        self.quarter_length
    }

    /// Get the quarter length as f64
    pub fn quarter_length_f64(&self) -> f64 {
        *self.quarter_length.numer() as f64 / *self.quarter_length.denom() as f64
    }

    /// Set the quarter note length
    pub fn set_quarter_length(&mut self, ql: impl Into<Fraction>) {
        self.quarter_length = ql.into();
        if self.linked {
            let (type_, dots) = Self::infer_type_and_dots(self.quarter_length);
            self.type_ = type_;
            self.dots = dots;
        }
    }

    /// Get the duration type
    pub fn type_(&self) -> Option<DurationType> {
        self.type_
    }

    /// Set the duration type
    pub fn set_type(&mut self, type_: DurationType) {
        self.type_ = Some(type_);
        if self.linked {
            self.quarter_length = Self::calculate_quarter_length(type_, self.dots, &self.tuplets);
        }
    }

    /// Get the number of dots
    pub fn dots(&self) -> u8 {
        self.dots
    }

    /// Set the number of dots
    pub fn set_dots(&mut self, dots: u8) {
        self.dots = dots;
        if self.linked {
            if let Some(type_) = self.type_ {
                self.quarter_length = Self::calculate_quarter_length(type_, dots, &self.tuplets);
            }
        }
    }

    /// Get the tuplets
    pub fn tuplets(&self) -> &[Tuplet] {
        &self.tuplets
    }

    /// Add a tuplet
    pub fn add_tuplet(&mut self, tuplet: Tuplet) {
        self.tuplets.push(tuplet);
        if self.linked {
            if let Some(type_) = self.type_ {
                self.quarter_length =
                    Self::calculate_quarter_length(type_, self.dots, &self.tuplets);
            }
        }
    }

    /// Clear all tuplets
    pub fn clear_tuplets(&mut self) {
        self.tuplets.clear();
        if self.linked {
            if let Some(type_) = self.type_ {
                self.quarter_length =
                    Self::calculate_quarter_length(type_, self.dots, &self.tuplets);
            }
        }
    }

    /// Check if linked mode is enabled
    pub fn linked(&self) -> bool {
        self.linked
    }

    /// Set linked mode
    pub fn set_linked(&mut self, linked: bool) {
        self.linked = linked;
    }

    /// Scale the duration by a factor
    pub fn augment_or_diminish(&self, scalar: impl Into<Fraction>) -> Duration {
        let new_ql = self.quarter_length * scalar.into();
        Duration::from_quarter_length(new_ql)
    }

    /// Double the duration
    pub fn augment(&self) -> Duration {
        self.augment_or_diminish(Fraction::new(2, 1))
    }

    /// Halve the duration
    pub fn diminish(&self) -> Duration {
        self.augment_or_diminish(Fraction::new(1, 2))
    }

    /// Check if this is a complex duration (requires ties)
    pub fn is_complex(&self) -> bool {
        self.type_.is_none()
    }

    /// Get a human-readable description
    pub fn full_name(&self) -> String {
        let mut name = String::new();

        if let Some(type_) = self.type_ {
            // Add dots
            for _ in 0..self.dots {
                name.push_str("Dotted ");
            }

            name.push_str(type_.name());

            // Add tuplets
            for tuplet in &self.tuplets {
                name.push_str(&format!(" ({} in {})", tuplet.actual, tuplet.normal));
            }
        } else {
            name = format!("Complex ({})", self.quarter_length);
        }

        name
    }

    /// Calculate quarter length from type, dots, and tuplets
    fn calculate_quarter_length(type_: DurationType, dots: u8, tuplets: &[Tuplet]) -> Fraction {
        let base = type_.quarter_length();

        // Apply dots: each dot adds half of the previous value
        let mut ql = base;
        let mut dot_value = base / 2;
        for _ in 0..dots {
            ql = ql + dot_value;
            dot_value = dot_value / 2;
        }

        // Apply tuplets
        for tuplet in tuplets {
            ql = ql * tuplet.multiplier();
        }

        ql
    }

    /// Infer type and dots from quarter length
    fn infer_type_and_dots(ql: Fraction) -> (Option<DurationType>, u8) {
        if ql == Fraction::zero() {
            return (Some(DurationType::Zero), 0);
        }

        // Try each duration type with 0-4 dots
        let types = [
            DurationType::Maxima,
            DurationType::Longa,
            DurationType::Breve,
            DurationType::Whole,
            DurationType::Half,
            DurationType::Quarter,
            DurationType::Eighth,
            DurationType::N16th,
            DurationType::N32nd,
            DurationType::N64th,
            DurationType::N128th,
            DurationType::N256th,
        ];

        for type_ in types {
            for dots in 0..=4 {
                if Self::calculate_quarter_length(type_, dots, &[]) == ql {
                    return (Some(type_), dots);
                }
            }
        }

        (None, 0)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::quarter()
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length + rhs.quarter_length)
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length - rhs.quarter_length)
    }
}

impl Mul<Fraction> for Duration {
    type Output = Duration;

    fn mul(self, rhs: Fraction) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length * rhs)
    }
}

impl Div<Fraction> for Duration {
    type Output = Duration;

    fn div(self, rhs: Fraction) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length / rhs)
    }
}

impl From<DurationType> for Duration {
    fn from(type_: DurationType) -> Self {
        Duration::from_type(type_, 0)
    }
}

impl FromStr for Duration {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse as a duration type first
        if let Ok(type_) = DurationType::from_str(s) {
            return Ok(Duration::from_type(type_, 0));
        }

        // Try to parse as a fraction (e.g., "1/4" for quarter note)
        if let Some((num, denom)) = s.split_once('/') {
            if let (Ok(n), Ok(d)) = (num.trim().parse::<i64>(), denom.trim().parse::<i64>()) {
                return Ok(Duration::from_quarter_length(Fraction::new(n, d)));
            }
        }

        // Try to parse as a decimal
        if let Ok(f) = s.parse::<f64>() {
            // Convert to fraction (approximate)
            let denom = 256i64;
            let numer = (f * denom as f64).round() as i64;
            return Ok(Duration::from_quarter_length(Fraction::new(numer, denom)));
        }

        Err(ParseError::InvalidDurationType(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_quarter_length() {
        assert_eq!(Duration::whole().quarter_length(), Fraction::new(4, 1));
        assert_eq!(Duration::half().quarter_length(), Fraction::new(2, 1));
        assert_eq!(Duration::quarter().quarter_length(), Fraction::one());
        assert_eq!(Duration::eighth().quarter_length(), Fraction::new(1, 2));
    }

    #[test]
    fn test_duration_from_quarter_length() {
        let d = Duration::from_quarter_length(Fraction::new(4, 1));
        assert_eq!(d.type_(), Some(DurationType::Whole));
        assert_eq!(d.dots(), 0);

        let d = Duration::from_quarter_length(Fraction::new(3, 2));
        assert_eq!(d.type_(), Some(DurationType::Quarter));
        assert_eq!(d.dots(), 1); // dotted quarter
    }

    #[test]
    fn test_duration_dots() {
        let dotted_half = Duration::from_type(DurationType::Half, 1);
        assert_eq!(dotted_half.quarter_length(), Fraction::new(3, 1));

        let double_dotted_quarter = Duration::from_type(DurationType::Quarter, 2);
        assert_eq!(double_dotted_quarter.quarter_length(), Fraction::new(7, 4));
    }

    #[test]
    fn test_duration_tuplet() {
        let mut d = Duration::quarter();
        d.add_tuplet(Tuplet::triplet());
        assert_eq!(d.quarter_length(), Fraction::new(2, 3));
    }

    #[test]
    fn test_duration_arithmetic() {
        let half = Duration::half();
        let quarter = Duration::quarter();

        let sum = half.clone() + quarter.clone();
        assert_eq!(sum.quarter_length(), Fraction::new(3, 1));

        let diff = half - quarter;
        assert_eq!(diff.quarter_length(), Fraction::one());
    }

    #[test]
    fn test_duration_augment_diminish() {
        let quarter = Duration::quarter();

        let half = quarter.augment();
        assert_eq!(half.quarter_length(), Fraction::new(2, 1));

        let eighth = quarter.diminish();
        assert_eq!(eighth.quarter_length(), Fraction::new(1, 2));
    }
}
