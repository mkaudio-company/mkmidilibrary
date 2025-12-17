//! Clef representation

use std::fmt;

/// Clef sign
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClefSign {
    G,
    F,
    C,
    Percussion,
    Tab,
}

impl ClefSign {
    /// Get the sign character
    pub fn char(&self) -> char {
        match self {
            ClefSign::G => 'G',
            ClefSign::F => 'F',
            ClefSign::C => 'C',
            ClefSign::Percussion => 'P',
            ClefSign::Tab => 'T',
        }
    }
}

impl fmt::Display for ClefSign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.char())
    }
}

/// A musical clef
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Clef {
    /// Clef sign
    sign: ClefSign,
    /// Line number (1 = bottom line)
    line: u8,
    /// Octave change (positive = up, negative = down)
    octave_change: i8,
}

impl Clef {
    /// Create a new clef
    pub fn new(sign: ClefSign, line: u8) -> Self {
        Self {
            sign,
            line,
            octave_change: 0,
        }
    }

    /// Create a new clef with octave change
    pub fn with_octave(sign: ClefSign, line: u8, octave_change: i8) -> Self {
        Self {
            sign,
            line,
            octave_change,
        }
    }

    /// Create treble clef (G clef on line 2)
    pub fn treble() -> Self {
        Self::new(ClefSign::G, 2)
    }

    /// Create bass clef (F clef on line 4)
    pub fn bass() -> Self {
        Self::new(ClefSign::F, 4)
    }

    /// Create alto clef (C clef on line 3)
    pub fn alto() -> Self {
        Self::new(ClefSign::C, 3)
    }

    /// Create tenor clef (C clef on line 4)
    pub fn tenor() -> Self {
        Self::new(ClefSign::C, 4)
    }

    /// Create soprano clef (C clef on line 1)
    pub fn soprano() -> Self {
        Self::new(ClefSign::C, 1)
    }

    /// Create mezzo-soprano clef (C clef on line 2)
    pub fn mezzo_soprano() -> Self {
        Self::new(ClefSign::C, 2)
    }

    /// Create baritone clef (F clef on line 3 or C clef on line 5)
    pub fn baritone() -> Self {
        Self::new(ClefSign::F, 3)
    }

    /// Create treble clef 8va (octave up)
    pub fn treble_8va() -> Self {
        Self::with_octave(ClefSign::G, 2, 1)
    }

    /// Create treble clef 8vb (octave down)
    pub fn treble_8vb() -> Self {
        Self::with_octave(ClefSign::G, 2, -1)
    }

    /// Create bass clef 8vb (octave down)
    pub fn bass_8vb() -> Self {
        Self::with_octave(ClefSign::F, 4, -1)
    }

    /// Create percussion clef
    pub fn percussion() -> Self {
        Self::new(ClefSign::Percussion, 3)
    }

    /// Create tab clef
    pub fn tab() -> Self {
        Self::new(ClefSign::Tab, 3)
    }

    /// Get the sign
    pub fn sign(&self) -> ClefSign {
        self.sign
    }

    /// Get the line
    pub fn line(&self) -> u8 {
        self.line
    }

    /// Get the octave change
    pub fn octave_change(&self) -> i8 {
        self.octave_change
    }

    /// Get the clef name
    pub fn name(&self) -> &'static str {
        match (self.sign, self.line, self.octave_change) {
            (ClefSign::G, 2, 0) => "treble",
            (ClefSign::G, 2, 1) => "treble 8va",
            (ClefSign::G, 2, -1) => "treble 8vb",
            (ClefSign::F, 4, 0) => "bass",
            (ClefSign::F, 4, -1) => "bass 8vb",
            (ClefSign::F, 3, 0) => "baritone",
            (ClefSign::C, 3, 0) => "alto",
            (ClefSign::C, 4, 0) => "tenor",
            (ClefSign::C, 1, 0) => "soprano",
            (ClefSign::C, 2, 0) => "mezzo-soprano",
            (ClefSign::Percussion, _, _) => "percussion",
            (ClefSign::Tab, _, _) => "tab",
            _ => "custom",
        }
    }

    /// Get the MIDI pitch of the clef reference line
    /// (e.g., treble clef: G4 = 67, bass clef: F3 = 53)
    pub fn reference_pitch(&self) -> u8 {
        let base = match self.sign {
            ClefSign::G => 67, // G4
            ClefSign::F => 53, // F3
            ClefSign::C => 60, // C4
            ClefSign::Percussion => 60,
            ClefSign::Tab => 60,
        };
        (base as i8 + self.octave_change * 12) as u8
    }

    /// Get the MIDI pitch of a note at a given staff position
    /// (position 0 = bottom ledger line below staff, 4 = bottom staff line)
    pub fn pitch_at_position(&self, position: i8) -> u8 {
        // Each staff position is a diatonic step
        // Position relative to clef line
        let clef_position = (self.line as i8 - 1) * 2; // Convert to ledger-based position
        let ref_pitch = self.reference_pitch();

        // Calculate pitch offset (each position is ~2 semitones on average)
        let steps = position - clef_position;
        let octaves = steps / 7;
        let step_in_octave = steps.rem_euclid(7);

        // Diatonic semitone offsets from C
        let semitones = [0, 2, 4, 5, 7, 9, 11];

        let base_octave_offset = octaves * 12;
        let step_offset = semitones[step_in_octave as usize];

        ((ref_pitch as i8) + base_octave_offset + step_offset as i8) as u8
    }
}

impl Default for Clef {
    fn default() -> Self {
        Self::treble()
    }
}

impl fmt::Display for Clef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clef_creation() {
        let treble = Clef::treble();
        assert_eq!(treble.sign(), ClefSign::G);
        assert_eq!(treble.line(), 2);
        assert_eq!(treble.name(), "treble");
    }

    #[test]
    fn test_clef_types() {
        assert_eq!(Clef::bass().name(), "bass");
        assert_eq!(Clef::alto().name(), "alto");
        assert_eq!(Clef::tenor().name(), "tenor");
    }

    #[test]
    fn test_clef_octave() {
        let treble_8vb = Clef::treble_8vb();
        assert_eq!(treble_8vb.octave_change(), -1);
        assert_eq!(treble_8vb.name(), "treble 8vb");
    }

    #[test]
    fn test_reference_pitch() {
        assert_eq!(Clef::treble().reference_pitch(), 67); // G4
        assert_eq!(Clef::bass().reference_pitch(), 53); // F3
        assert_eq!(Clef::alto().reference_pitch(), 60); // C4
    }
}
