//! Accidentals and microtones

use std::fmt;

use super::ParseError;

/// Musical accidental (sharp, flat, natural, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
    /// Triple flat (bbb) - lowers pitch by 3 semitones
    TripleFlat,
    /// Double flat (bb) - lowers pitch by 2 semitones
    DoubleFlat,
    /// Flat (b) - lowers pitch by 1 semitone
    Flat,
    /// Natural - no alteration
    Natural,
    /// Sharp (#) - raises pitch by 1 semitone
    Sharp,
    /// Double sharp (x) - raises pitch by 2 semitones
    DoubleSharp,
    /// Triple sharp - raises pitch by 3 semitones
    TripleSharp,
    /// Quarter-tone flat
    QuarterFlat,
    /// Quarter-tone sharp
    QuarterSharp,
    /// Three-quarter flat
    ThreeQuarterFlat,
    /// Three-quarter sharp
    ThreeQuarterSharp,
}

impl Accidental {
    /// Get the semitone alteration value
    pub fn alter(&self) -> f64 {
        match self {
            Accidental::TripleFlat => -3.0,
            Accidental::DoubleFlat => -2.0,
            Accidental::Flat => -1.0,
            Accidental::Natural => 0.0,
            Accidental::Sharp => 1.0,
            Accidental::DoubleSharp => 2.0,
            Accidental::TripleSharp => 3.0,
            Accidental::QuarterFlat => -0.5,
            Accidental::QuarterSharp => 0.5,
            Accidental::ThreeQuarterFlat => -1.5,
            Accidental::ThreeQuarterSharp => 1.5,
        }
    }

    /// Create accidental from semitone alteration
    pub fn from_alter(alter: f64) -> Option<Accidental> {
        match alter {
            x if (x - -3.0).abs() < 0.01 => Some(Accidental::TripleFlat),
            x if (x - -2.0).abs() < 0.01 => Some(Accidental::DoubleFlat),
            x if (x - -1.5).abs() < 0.01 => Some(Accidental::ThreeQuarterFlat),
            x if (x - -1.0).abs() < 0.01 => Some(Accidental::Flat),
            x if (x - -0.5).abs() < 0.01 => Some(Accidental::QuarterFlat),
            x if x.abs() < 0.01 => Some(Accidental::Natural),
            x if (x - 0.5).abs() < 0.01 => Some(Accidental::QuarterSharp),
            x if (x - 1.0).abs() < 0.01 => Some(Accidental::Sharp),
            x if (x - 1.5).abs() < 0.01 => Some(Accidental::ThreeQuarterSharp),
            x if (x - 2.0).abs() < 0.01 => Some(Accidental::DoubleSharp),
            x if (x - 3.0).abs() < 0.01 => Some(Accidental::TripleSharp),
            _ => None,
        }
    }

    /// Parse accidental from string
    pub fn from_str(s: &str) -> Result<Accidental, ParseError> {
        match s.to_lowercase().as_str() {
            "bbb" | "triple-flat" => Ok(Accidental::TripleFlat),
            "bb" | "--" | "double-flat" => Ok(Accidental::DoubleFlat),
            "b" | "-" | "flat" => Ok(Accidental::Flat),
            "" | "n" | "natural" => Ok(Accidental::Natural),
            "#" | "sharp" => Ok(Accidental::Sharp),
            "##" | "x" | "++" | "double-sharp" => Ok(Accidental::DoubleSharp),
            "###" | "triple-sharp" => Ok(Accidental::TripleSharp),
            "~" | "half-flat" | "quarter-flat" => Ok(Accidental::QuarterFlat),
            "`" | "half-sharp" | "quarter-sharp" => Ok(Accidental::QuarterSharp),
            _ => Err(ParseError::InvalidAccidental(s.to_string())),
        }
    }

    /// Get the Unicode symbol for this accidental
    pub fn unicode(&self) -> &'static str {
        match self {
            Accidental::TripleFlat => "\u{266D}\u{266D}\u{266D}",
            Accidental::DoubleFlat => "\u{1D12B}",
            Accidental::Flat => "\u{266D}",
            Accidental::Natural => "\u{266E}",
            Accidental::Sharp => "\u{266F}",
            Accidental::DoubleSharp => "\u{1D12A}",
            Accidental::TripleSharp => "\u{266F}\u{266F}\u{266F}",
            Accidental::QuarterFlat => "\u{1D132}",
            Accidental::QuarterSharp => "\u{1D132}",
            Accidental::ThreeQuarterFlat => "\u{1D12B}\u{1D132}",
            Accidental::ThreeQuarterSharp => "\u{1D12A}\u{1D132}",
        }
    }

    /// Get ASCII representation
    pub fn ascii(&self) -> &'static str {
        match self {
            Accidental::TripleFlat => "bbb",
            Accidental::DoubleFlat => "bb",
            Accidental::Flat => "b",
            Accidental::Natural => "",
            Accidental::Sharp => "#",
            Accidental::DoubleSharp => "##",
            Accidental::TripleSharp => "###",
            Accidental::QuarterFlat => "~",
            Accidental::QuarterSharp => "`",
            Accidental::ThreeQuarterFlat => "b~",
            Accidental::ThreeQuarterSharp => "#`",
        }
    }

    /// Check if this is a standard (non-microtonal) accidental
    pub fn is_standard(&self) -> bool {
        matches!(
            self,
            Accidental::TripleFlat
                | Accidental::DoubleFlat
                | Accidental::Flat
                | Accidental::Natural
                | Accidental::Sharp
                | Accidental::DoubleSharp
                | Accidental::TripleSharp
        )
    }
}

impl Default for Accidental {
    fn default() -> Self {
        Accidental::Natural
    }
}

impl fmt::Display for Accidental {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ascii())
    }
}

/// Microtone adjustment in cents
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Microtone {
    /// Adjustment in cents (100 cents = 1 semitone)
    cents: f64,
}

impl Microtone {
    /// Create a new microtone adjustment
    pub fn new(cents: f64) -> Self {
        Self { cents }
    }

    /// Get the cents value
    pub fn cents(&self) -> f64 {
        self.cents
    }

    /// Get the semitone alteration (cents / 100)
    pub fn alter(&self) -> f64 {
        self.cents / 100.0
    }

    /// Create from semitone alteration
    pub fn from_alter(alter: f64) -> Self {
        Self {
            cents: alter * 100.0,
        }
    }
}

impl Default for Microtone {
    fn default() -> Self {
        Self { cents: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accidental_alter() {
        assert_eq!(Accidental::Sharp.alter(), 1.0);
        assert_eq!(Accidental::Flat.alter(), -1.0);
        assert_eq!(Accidental::DoubleSharp.alter(), 2.0);
        assert_eq!(Accidental::Natural.alter(), 0.0);
    }

    #[test]
    fn test_accidental_from_alter() {
        assert_eq!(Accidental::from_alter(1.0), Some(Accidental::Sharp));
        assert_eq!(Accidental::from_alter(-1.0), Some(Accidental::Flat));
        assert_eq!(Accidental::from_alter(0.5), Some(Accidental::QuarterSharp));
    }

    #[test]
    fn test_accidental_parse() {
        assert_eq!(Accidental::from_str("#").unwrap(), Accidental::Sharp);
        assert_eq!(Accidental::from_str("b").unwrap(), Accidental::Flat);
        assert_eq!(Accidental::from_str("bb").unwrap(), Accidental::DoubleFlat);
    }

    #[test]
    fn test_microtone() {
        let m = Microtone::new(50.0);
        assert_eq!(m.cents(), 50.0);
        assert_eq!(m.alter(), 0.5);
    }
}
