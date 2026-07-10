//! Accidentals and microtones

use std::fmt;

use super::ParseError;

/// Musical accidental (sharp, flat, natural, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Accidental {
    /// Triple flat (bbb) - lowers pitch by 3 semitones
    TripleFlat,
    /// Double flat (bb) - lowers pitch by 2 semitones
    DoubleFlat,
    /// Flat (b) - lowers pitch by 1 semitone
    Flat,
    /// Natural - no alteration
    #[default]
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

    /// List the canonical string names of all accidental variants (the
    /// preferred names accepted by `from_str`).
    pub fn list_names() -> Vec<&'static str> {
        vec![
            "triple-flat",
            "double-flat",
            "flat",
            "natural",
            "sharp",
            "double-sharp",
            "triple-sharp",
            "quarter-flat",
            "quarter-sharp",
        ]
    }
}

impl std::str::FromStr for Accidental {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Accidental, ParseError> {
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
}

impl fmt::Display for Accidental {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ascii())
    }
}

/// How an accidental's display status was/should be determined. `Accidental`
/// itself is a plain `Copy` value type used throughout the crate, so this
/// display-state machine lives in a separate wrapper (`AccidentalDisplay`)
/// rather than adding fields to `Accidental` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccidentalDisplayType {
    #[default]
    Normal,
    /// Always display, regardless of key signature/measure context.
    Always,
    /// Never display.
    Never,
    /// Display only if it would otherwise be ambiguous (e.g. it differs
    /// from the key signature or a previous accidental in the same
    /// measure on the same line/space).
    IfNeeded,
}

/// Pairs an `Accidental` with whether it should actually be printed in
/// notation, and why. Mirrors music21's accidental-display-state fields
/// (`displayStatus`, `displayType`) that `Accidental` alone doesn't carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccidentalDisplay {
    /// The accidental itself.
    pub accidental: Accidental,
    /// Whether this accidental should actually be displayed. `None` means
    /// undetermined (not yet computed by a display-status algorithm, e.g.
    /// `Pitch::update_accidental_display`).
    pub display_status: Option<bool>,
    /// How the display status was (or should be) determined.
    pub display_type: AccidentalDisplayType,
}

impl AccidentalDisplay {
    /// Create a new display wrapper with undetermined display status.
    pub fn new(accidental: Accidental) -> Self {
        Self {
            accidental,
            display_status: None,
            display_type: AccidentalDisplayType::Normal,
        }
    }

    /// Set the accidental and/or display type independently — i.e. without
    /// resetting the other one or the display status. Mirrors music21's
    /// `Accidental.setAttributeIndependently`.
    pub fn set_attribute_independently(
        &mut self,
        accidental: Option<Accidental>,
        display_type: Option<AccidentalDisplayType>,
    ) {
        if let Some(a) = accidental {
            self.accidental = a;
        }
        if let Some(t) = display_type {
            self.display_type = t;
        }
    }

    /// Inherit the display status from another accidental's display
    /// decision (e.g. a tied-over note should not re-display its
    /// accidental, inheriting the "don't display" decision from the note
    /// it's tied from).
    pub fn inherit_display(&mut self, other: &AccidentalDisplay) {
        self.display_status = other.display_status;
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
    use std::str::FromStr;

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

    #[test]
    fn test_accidental_list_names() {
        let names = Accidental::list_names();
        assert!(names.contains(&"sharp"));
        assert!(names.contains(&"flat"));
        assert_eq!(names.len(), 9);
    }

    #[test]
    fn test_accidental_display() {
        let mut display = AccidentalDisplay::new(Accidental::Sharp);
        assert_eq!(display.display_status, None);
        assert_eq!(display.display_type, AccidentalDisplayType::Normal);

        display.set_attribute_independently(None, Some(AccidentalDisplayType::Always));
        assert_eq!(display.accidental, Accidental::Sharp); // unchanged
        assert_eq!(display.display_type, AccidentalDisplayType::Always);

        let mut other = AccidentalDisplay::new(Accidental::Flat);
        other.display_status = Some(false);
        display.inherit_display(&other);
        assert_eq!(display.display_status, Some(false));
    }
}
