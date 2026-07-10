//! Clef representation

use std::fmt;

use crate::core::Pitch;

/// Clef sign
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClefSign {
    G,
    F,
    C,
    Percussion,
    Tab,
    /// No clef at all (unpitched staff with no reference line).
    None,
    /// Jianpu (numbered musical notation), which has no staff/clef of its
    /// own but is represented here so a `Clef` can still be attached to a
    /// part using this notation style.
    Jianpu,
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
            ClefSign::None => 'N',
            ClefSign::Jianpu => 'J',
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

    /// Create French violin clef (G clef on line 1 — rare, historically
    /// used for the violin/flute before treble clef became standard).
    pub fn french_violin() -> Self {
        Self::new(ClefSign::G, 1)
    }

    /// Create sub-bass clef (F clef on line 5 — used for very low voices,
    /// e.g. contrabass).
    pub fn sub_bass() -> Self {
        Self::new(ClefSign::F, 5)
    }

    /// Create a "no clef" marker (unpitched staff with no reference line).
    pub fn no_clef() -> Self {
        Self::new(ClefSign::None, 3)
    }

    /// Create a Jianpu (numbered musical notation) marker.
    pub fn jianpu() -> Self {
        Self::new(ClefSign::Jianpu, 3)
    }

    /// Parse a clef from a string. Accepts common names (`"treble"`,
    /// `"bass"`, `"alto"`, `"tenor"`, `"soprano"`, `"mezzo-soprano"`,
    /// `"baritone"`, `"french violin"`, `"sub-bass"`, `"percussion"`,
    /// `"tab"`, `"none"`, `"jianpu"`, the 8va/8vb treble/bass variants),
    /// case-insensitively, as well as the generic `"<sign><line>"` form
    /// used by formats like MusicXML/music21 (e.g. `"G2"` = treble,
    /// `"F4"` = bass, `"C3"` = alto). Returns `None` for unrecognized
    /// input; does not attempt to parse octave-shift suffixes on the
    /// generic form (use the named 8va/8vb variants for those).
    pub fn from_string(s: &str) -> Option<Clef> {
        let trimmed = s.trim();
        match trimmed.to_lowercase().as_str() {
            "treble" | "g2" => return Some(Clef::treble()),
            "treble8va" | "treble 8va" => return Some(Clef::treble_8va()),
            "treble8vb" | "treble 8vb" => return Some(Clef::treble_8vb()),
            "bass" | "f4" => return Some(Clef::bass()),
            "bass8vb" | "bass 8vb" => return Some(Clef::bass_8vb()),
            "alto" | "c3" => return Some(Clef::alto()),
            "tenor" | "c4" => return Some(Clef::tenor()),
            "soprano" | "c1" => return Some(Clef::soprano()),
            "mezzo-soprano" | "mezzosoprano" | "c2" => return Some(Clef::mezzo_soprano()),
            "baritone" | "f3" => return Some(Clef::baritone()),
            "french violin" | "frenchviolin" | "g1" => return Some(Clef::french_violin()),
            "sub-bass" | "subbass" | "f5" => return Some(Clef::sub_bass()),
            "percussion" => return Some(Clef::percussion()),
            "tab" | "tablature" => return Some(Clef::tab()),
            "none" | "no clef" | "noclef" => return Some(Clef::no_clef()),
            "jianpu" => return Some(Clef::jianpu()),
            _ => {}
        }

        let mut chars = trimmed.chars();
        let sign = match chars.next()?.to_ascii_uppercase() {
            'G' => ClefSign::G,
            'F' => ClefSign::F,
            'C' => ClefSign::C,
            'P' => ClefSign::Percussion,
            'T' => ClefSign::Tab,
            _ => return None,
        };
        let line: u8 = chars.as_str().parse().ok()?;
        Some(Clef::new(sign, line))
    }

    /// A simple treble-or-bass clef recommendation for a set of pitches,
    /// based on their average pitch relative to middle C. Mirrors (a
    /// simplified form of) music21's `bestClef`, which more fully weighs
    /// how many notes would need ledger lines under each clef choice
    /// (including alto as a third option); this only chooses between
    /// treble and bass.
    pub fn best_clef_for_pitches(pitches: &[Pitch]) -> Clef {
        if pitches.is_empty() {
            return Clef::treble();
        }
        let average: f64 =
            pitches.iter().map(|p| p.midi() as f64).sum::<f64>() / pitches.len() as f64;
        if average >= 60.0 {
            Clef::treble()
        } else {
            Clef::bass()
        }
    }

    /// A stem-direction recommendation for a set of pitches notated on
    /// this clef: notes averaging below the staff's middle line get
    /// stems up, at or above it get stems down (the standard notation
    /// convention).
    pub fn stem_direction_for_pitches(&self, pitches: &[Pitch]) -> crate::core::StemDirection {
        if pitches.is_empty() {
            return crate::core::StemDirection::Up;
        }
        let middle_line_pitch = self.pitch_at_position(8) as f64;
        let average: f64 =
            pitches.iter().map(|p| p.midi() as f64).sum::<f64>() / pitches.len() as f64;
        if average < middle_line_pitch {
            crate::core::StemDirection::Up
        } else {
            crate::core::StemDirection::Down
        }
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
            (ClefSign::G, 1, 0) => "french violin",
            (ClefSign::F, 5, 0) => "sub-bass",
            (ClefSign::Percussion, _, _) => "percussion",
            (ClefSign::Tab, _, _) => "tab",
            (ClefSign::None, _, _) => "none",
            (ClefSign::Jianpu, _, _) => "jianpu",
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
            ClefSign::None => 60,
            ClefSign::Jianpu => 60,
        };
        (base as i8 + self.octave_change * 12) as u8
    }

    /// The diatonic scale degree (0 = C, 1 = D, ... 6 = B) of this clef's
    /// reference pitch's letter name, used to correctly re-anchor the
    /// C-major semitone table in `pitch_at_position` for clefs whose
    /// reference note isn't C.
    fn reference_degree(&self) -> i32 {
        match self.sign {
            ClefSign::G => 4, // G
            ClefSign::F => 3, // F
            ClefSign::C | ClefSign::Percussion | ClefSign::Tab | ClefSign::None | ClefSign::Jianpu => 0, // C
        }
    }

    /// Get the MIDI pitch of a note at a given staff position (position 0
    /// = bottom ledger line below staff, 4 = bottom staff line, +2 per
    /// line/space moving up).
    pub fn pitch_at_position(&self, position: i8) -> u8 {
        // Convert the clef's own line number to the same position scale
        // (line 1 = position 4, +2 per line moving up), and find how many
        // diatonic steps `position` is from the clef's reference line.
        let clef_position = 4 + (self.line as i8 - 1) * 2;
        let steps = (position - clef_position) as i32;

        // Diatonic semitone offsets from C, indexed by scale degree.
        let semitones_from_c = [0, 2, 4, 5, 7, 9, 11];

        // Regression fix: this used to index `semitones_from_c` directly
        // by the raw step count, implicitly treating the reference pitch
        // as if it were always C — correct only when it actually is (the
        // C-clef family), and silently wrong for the G/F clefs wherever a
        // query crossed the point where the true diatonic half-step
        // (e.g. G major's B-C at the 7th degree) diverges from the
        // C-major table's own half-step location. Re-anchoring via the
        // reference pitch's own scale degree (and the "C" pitch that sits
        // in the same octave as it) makes every clef consistent.
        let reference_degree = self.reference_degree();
        let c_in_reference_octave = self.reference_pitch() as i32 - semitones_from_c[reference_degree as usize];

        let total_degree = reference_degree + steps;
        let octave_shift = total_degree.div_euclid(7);
        let degree_in_octave = total_degree.rem_euclid(7) as usize;

        (c_in_reference_octave + octave_shift * 12 + semitones_from_c[degree_in_octave]) as u8
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

    #[test]
    fn test_pitch_at_position_treble_staff_lines() {
        // Regression: pitch_at_position used to index the C-major
        // semitone table directly by raw step count, which is only
        // correct when the clef's own reference pitch is C. For treble
        // (reference G), this made positions beyond a certain point
        // silently return the wrong (G-mixolydian-shaped) pitch instead
        // of true G-major-relative diatonic steps. All 5 lines of the
        // treble staff are checked against their real, well-known notes.
        let treble = Clef::treble();
        assert_eq!(treble.pitch_at_position(4), 64); // line 1 (bottom): E4
        assert_eq!(treble.pitch_at_position(6), 67); // line 2: G4 (reference)
        assert_eq!(treble.pitch_at_position(8), 71); // line 3: B4
        assert_eq!(treble.pitch_at_position(10), 74); // line 4: D5
        assert_eq!(treble.pitch_at_position(12), 77); // line 5 (top): F5
    }

    #[test]
    fn test_pitch_at_position_bass_staff_lines() {
        let bass = Clef::bass();
        assert_eq!(bass.pitch_at_position(4), 43); // line 1 (bottom): G2
        assert_eq!(bass.pitch_at_position(10), 53); // line 4: F3 (reference)
        assert_eq!(bass.pitch_at_position(12), 57); // line 5 (top): A3
    }

    #[test]
    fn test_pitch_at_position_alto_staff_lines() {
        let alto = Clef::alto();
        assert_eq!(alto.pitch_at_position(4), 53); // line 1 (bottom): F3
        assert_eq!(alto.pitch_at_position(8), 60); // line 3: C4 (reference)
        assert_eq!(alto.pitch_at_position(12), 67); // line 5 (top): G4
    }

    #[test]
    fn test_pitch_at_position_tenor_staff_lines() {
        let tenor = Clef::tenor();
        assert_eq!(tenor.pitch_at_position(4), 50); // line 1 (bottom): D3
        assert_eq!(tenor.pitch_at_position(10), 60); // line 4: C4 (reference)
    }

    #[test]
    fn test_clef_from_string_named() {
        assert_eq!(Clef::from_string("treble"), Some(Clef::treble()));
        assert_eq!(Clef::from_string("Bass"), Some(Clef::bass()));
        assert_eq!(Clef::from_string("ALTO"), Some(Clef::alto()));
        assert_eq!(Clef::from_string("mezzo-soprano"), Some(Clef::mezzo_soprano()));
        assert_eq!(Clef::from_string("french violin"), Some(Clef::french_violin()));
        assert_eq!(Clef::from_string("sub-bass"), Some(Clef::sub_bass()));
        assert_eq!(Clef::from_string("none"), Some(Clef::no_clef()));
        assert_eq!(Clef::from_string("jianpu"), Some(Clef::jianpu()));
        assert_eq!(Clef::from_string("not a clef"), None);
    }

    #[test]
    fn test_clef_from_string_sign_line_form() {
        assert_eq!(Clef::from_string("G2"), Some(Clef::treble()));
        assert_eq!(Clef::from_string("F4"), Some(Clef::bass()));
        assert_eq!(Clef::from_string("C3"), Some(Clef::alto()));
        assert_eq!(Clef::from_string("C4"), Some(Clef::tenor()));
        assert_eq!(Clef::from_string("X9"), None);
    }

    #[test]
    fn test_best_clef_for_pitches() {
        use crate::core::{Pitch, Step};
        let high = vec![
            Pitch::from_parts(Step::C, Some(5), None),
            Pitch::from_parts(Step::E, Some(5), None),
        ];
        assert_eq!(Clef::best_clef_for_pitches(&high), Clef::treble());

        let low = vec![
            Pitch::from_parts(Step::C, Some(2), None),
            Pitch::from_parts(Step::E, Some(2), None),
        ];
        assert_eq!(Clef::best_clef_for_pitches(&low), Clef::bass());
    }

    #[test]
    fn test_stem_direction_for_pitches() {
        use crate::core::{Pitch, Step, StemDirection};
        let treble = Clef::treble();

        let low_note = vec![Pitch::from_parts(Step::C, Some(4), None)];
        assert_eq!(treble.stem_direction_for_pitches(&low_note), StemDirection::Up);

        let high_note = vec![Pitch::from_parts(Step::C, Some(6), None)];
        assert_eq!(treble.stem_direction_for_pitches(&high_note), StemDirection::Down);
    }

    #[test]
    fn test_new_clef_variant_names() {
        assert_eq!(Clef::french_violin().name(), "french violin");
        assert_eq!(Clef::sub_bass().name(), "sub-bass");
        assert_eq!(Clef::no_clef().name(), "none");
        assert_eq!(Clef::jianpu().name(), "jianpu");
    }
}
