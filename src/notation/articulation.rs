//! Articulation markings

use std::fmt;

/// Placement for articulation marks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ArticulationPlacement {
    #[default]
    Auto,
    Above,
    Below,
}

/// Type of articulation mark
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArticulationMark {
    /// Accent (>)
    Accent,
    /// Strong accent (^)
    StrongAccent,
    /// Staccato (.)
    Staccato,
    /// Staccatissimo (wedge)
    Staccatissimo,
    /// Tenuto (-)
    Tenuto,
    /// Detached legato (tenuto + staccato)
    DetachedLegato,
    /// Marcato (^)
    Marcato,
    /// Fermata
    Fermata,
    /// Short fermata
    ShortFermata,
    /// Long fermata
    LongFermata,
    /// Breath mark
    BreathMark,
    /// Caesura
    Caesura,
    /// Up bow (string)
    UpBow,
    /// Down bow (string)
    DownBow,
    /// Harmonic
    Harmonic,
    /// Open string
    OpenString,
    /// Stopped (brass)
    Stopped,
    /// Snap pizzicato
    SnapPizzicato,
    /// Thumb position
    ThumbPosition,
    /// Pluck (guitar)
    Pluck,
    /// Double tongue
    DoubleTongue,
    /// Triple tongue
    TripleTongue,
    /// Heel (organ pedal)
    Heel,
    /// Toe (organ pedal)
    Toe,
}

impl ArticulationMark {
    /// Get the symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            ArticulationMark::Accent => ">",
            ArticulationMark::StrongAccent => "^",
            ArticulationMark::Staccato => ".",
            ArticulationMark::Staccatissimo => "▼",
            ArticulationMark::Tenuto => "-",
            ArticulationMark::DetachedLegato => "-.",
            ArticulationMark::Marcato => "^",
            ArticulationMark::Fermata => "𝄐",
            ArticulationMark::ShortFermata => "𝄑",
            ArticulationMark::LongFermata => "𝄒",
            ArticulationMark::BreathMark => ",",
            ArticulationMark::Caesura => "//",
            ArticulationMark::UpBow => "∨",
            ArticulationMark::DownBow => "∏",
            ArticulationMark::Harmonic => "○",
            ArticulationMark::OpenString => "○",
            ArticulationMark::Stopped => "+",
            ArticulationMark::SnapPizzicato => "⊙",
            ArticulationMark::ThumbPosition => "◯",
            ArticulationMark::Pluck => "i",
            ArticulationMark::DoubleTongue => "‥",
            ArticulationMark::TripleTongue => "…",
            ArticulationMark::Heel => "U",
            ArticulationMark::Toe => "^",
        }
    }

    /// Get the name
    pub fn name(&self) -> &'static str {
        match self {
            ArticulationMark::Accent => "accent",
            ArticulationMark::StrongAccent => "strong accent",
            ArticulationMark::Staccato => "staccato",
            ArticulationMark::Staccatissimo => "staccatissimo",
            ArticulationMark::Tenuto => "tenuto",
            ArticulationMark::DetachedLegato => "detached legato",
            ArticulationMark::Marcato => "marcato",
            ArticulationMark::Fermata => "fermata",
            ArticulationMark::ShortFermata => "short fermata",
            ArticulationMark::LongFermata => "long fermata",
            ArticulationMark::BreathMark => "breath mark",
            ArticulationMark::Caesura => "caesura",
            ArticulationMark::UpBow => "up bow",
            ArticulationMark::DownBow => "down bow",
            ArticulationMark::Harmonic => "harmonic",
            ArticulationMark::OpenString => "open string",
            ArticulationMark::Stopped => "stopped",
            ArticulationMark::SnapPizzicato => "snap pizzicato",
            ArticulationMark::ThumbPosition => "thumb position",
            ArticulationMark::Pluck => "pluck",
            ArticulationMark::DoubleTongue => "double tongue",
            ArticulationMark::TripleTongue => "triple tongue",
            ArticulationMark::Heel => "heel",
            ArticulationMark::Toe => "toe",
        }
    }

    /// Get the velocity multiplier
    pub fn velocity_multiplier(&self) -> f64 {
        match self {
            ArticulationMark::Accent => 1.2,
            ArticulationMark::StrongAccent | ArticulationMark::Marcato => 1.4,
            ArticulationMark::Staccato => 0.9,
            ArticulationMark::Staccatissimo => 1.1,
            ArticulationMark::Tenuto => 1.0,
            ArticulationMark::DetachedLegato => 0.95,
            _ => 1.0,
        }
    }

    /// Get the duration multiplier
    pub fn duration_multiplier(&self) -> f64 {
        match self {
            ArticulationMark::Staccato => 0.5,
            ArticulationMark::Staccatissimo => 0.25,
            ArticulationMark::Tenuto => 1.0,
            ArticulationMark::DetachedLegato => 0.75,
            _ => 1.0,
        }
    }

    /// Check if this affects note duration
    pub fn affects_duration(&self) -> bool {
        matches!(
            self,
            ArticulationMark::Staccato
                | ArticulationMark::Staccatissimo
                | ArticulationMark::DetachedLegato
        )
    }

    /// Check if this is a fermata type
    pub fn is_fermata(&self) -> bool {
        matches!(
            self,
            ArticulationMark::Fermata
                | ArticulationMark::ShortFermata
                | ArticulationMark::LongFermata
        )
    }
}

impl fmt::Display for ArticulationMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_articulation_mark() {
        let staccato = ArticulationMark::Staccato;
        assert_eq!(staccato.name(), "staccato");
        assert!(staccato.affects_duration());
        assert!(staccato.duration_multiplier() < 1.0);
    }

    #[test]
    fn test_accent_velocity() {
        let accent = ArticulationMark::Accent;
        assert!(accent.velocity_multiplier() > 1.0);
    }

    #[test]
    fn test_fermata() {
        assert!(ArticulationMark::Fermata.is_fermata());
        assert!(ArticulationMark::ShortFermata.is_fermata());
        assert!(!ArticulationMark::Staccato.is_fermata());
    }
}
