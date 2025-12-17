//! Key signature representation

use std::fmt;

use crate::core::Step;

/// Key mode (major/minor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KeyMode {
    #[default]
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
}

impl KeyMode {
    /// Get the mode name
    pub fn name(&self) -> &'static str {
        match self {
            KeyMode::Major => "major",
            KeyMode::Minor => "minor",
            KeyMode::Dorian => "dorian",
            KeyMode::Phrygian => "phrygian",
            KeyMode::Lydian => "lydian",
            KeyMode::Mixolydian => "mixolydian",
            KeyMode::Aeolian => "aeolian",
            KeyMode::Locrian => "locrian",
        }
    }
}

impl fmt::Display for KeyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A musical key (tonic + mode)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// The tonic pitch class
    tonic: Step,
    /// The mode
    mode: KeyMode,
}

impl Key {
    /// Create a new key
    pub fn new(tonic: Step, mode: KeyMode) -> Self {
        Self { tonic, mode }
    }

    /// Create a major key
    pub fn major(tonic: Step) -> Self {
        Self::new(tonic, KeyMode::Major)
    }

    /// Create a minor key
    pub fn minor(tonic: Step) -> Self {
        Self::new(tonic, KeyMode::Minor)
    }

    /// Get the tonic
    pub fn tonic(&self) -> Step {
        self.tonic
    }

    /// Get the mode
    pub fn mode(&self) -> KeyMode {
        self.mode
    }

    /// Get the relative major/minor
    pub fn relative(&self) -> Key {
        match self.mode {
            KeyMode::Major | KeyMode::Aeolian => {
                // Relative minor is 3 semitones down
                Key::new(Step::from_index(self.tonic.index() + 5), KeyMode::Minor)
            }
            KeyMode::Minor => {
                // Relative major is 3 semitones up
                Key::new(Step::from_index(self.tonic.index() + 2), KeyMode::Major)
            }
            _ => self.clone(),
        }
    }

    /// Get the parallel major/minor
    pub fn parallel(&self) -> Key {
        match self.mode {
            KeyMode::Major => Key::minor(self.tonic),
            KeyMode::Minor => Key::major(self.tonic),
            _ => self.clone(),
        }
    }

    /// Get the key name (e.g., "C major", "A minor")
    pub fn name(&self) -> String {
        format!("{} {}", self.tonic, self.mode)
    }
}

impl Default for Key {
    fn default() -> Self {
        Self::major(Step::C)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Key signature (number of sharps/flats)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeySignature {
    /// Number of sharps (positive) or flats (negative)
    sharps: i8,
    /// Whether this is a minor key
    minor: bool,
}

impl KeySignature {
    /// Create a new key signature
    pub fn new(sharps: i8, minor: bool) -> Self {
        Self { sharps, minor }
    }

    /// Create from sharps count (major)
    pub fn from_sharps(sharps: i8) -> Self {
        Self {
            sharps,
            minor: false,
        }
    }

    /// Create from flats count (major)
    pub fn from_flats(flats: u8) -> Self {
        Self {
            sharps: -(flats as i8),
            minor: false,
        }
    }

    /// Create C major key signature
    pub fn c_major() -> Self {
        Self::new(0, false)
    }

    /// Create A minor key signature
    pub fn a_minor() -> Self {
        Self::new(0, true)
    }

    /// Create G major key signature
    pub fn g_major() -> Self {
        Self::new(1, false)
    }

    /// Create F major key signature
    pub fn f_major() -> Self {
        Self::new(-1, false)
    }

    /// Get the number of sharps (positive) or flats (negative)
    pub fn sharps(&self) -> i8 {
        self.sharps
    }

    /// Get the number of flats (0 if sharps)
    pub fn flats(&self) -> u8 {
        if self.sharps < 0 {
            (-self.sharps) as u8
        } else {
            0
        }
    }

    /// Check if minor
    pub fn is_minor(&self) -> bool {
        self.minor
    }

    /// Check if major
    pub fn is_major(&self) -> bool {
        !self.minor
    }

    /// Get the tonic step
    pub fn tonic(&self) -> Step {
        // Circle of fifths for major keys
        let major_tonics = [
            Step::C, Step::G, Step::D, Step::A, Step::E, Step::B, Step::F,
        ];
        let flat_tonics = [
            Step::C, Step::F, Step::B, Step::E, Step::A, Step::D, Step::G,
        ];

        let tonic = if self.sharps >= 0 {
            major_tonics[self.sharps as usize % 7]
        } else {
            flat_tonics[(-self.sharps) as usize % 7]
        };

        if self.minor {
            // Relative minor is 3 steps below
            Step::from_index(tonic.index() + 5)
        } else {
            tonic
        }
    }

    /// Get the altered pitches
    pub fn altered_pitches(&self) -> Vec<Step> {
        let sharp_order = [Step::F, Step::C, Step::G, Step::D, Step::A, Step::E, Step::B];
        let flat_order = [Step::B, Step::E, Step::A, Step::D, Step::G, Step::C, Step::F];

        if self.sharps > 0 {
            sharp_order[..self.sharps as usize].to_vec()
        } else if self.sharps < 0 {
            flat_order[..(-self.sharps) as usize].to_vec()
        } else {
            vec![]
        }
    }

    /// Check if a pitch is altered in this key
    pub fn is_altered(&self, step: Step) -> bool {
        self.altered_pitches().contains(&step)
    }

    /// Get the accidental for a step in this key
    pub fn accidental_for(&self, step: Step) -> Option<crate::core::Accidental> {
        if self.is_altered(step) {
            if self.sharps > 0 {
                Some(crate::core::Accidental::Sharp)
            } else {
                Some(crate::core::Accidental::Flat)
            }
        } else {
            None
        }
    }

    /// Convert to Key
    pub fn to_key(&self) -> Key {
        let mode = if self.minor {
            KeyMode::Minor
        } else {
            KeyMode::Major
        };
        Key::new(self.tonic(), mode)
    }
}

impl Default for KeySignature {
    fn default() -> Self {
        Self::c_major()
    }
}

impl fmt::Display for KeySignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sharps == 0 {
            write!(f, "no sharps/flats")
        } else if self.sharps > 0 {
            write!(f, "{} sharp(s)", self.sharps)
        } else {
            write!(f, "{} flat(s)", -self.sharps)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_creation() {
        let key = Key::major(Step::C);
        assert_eq!(key.tonic(), Step::C);
        assert_eq!(key.mode(), KeyMode::Major);
    }

    #[test]
    fn test_key_relative() {
        let c_major = Key::major(Step::C);
        let relative = c_major.relative();
        assert_eq!(relative.tonic(), Step::A);
        assert_eq!(relative.mode(), KeyMode::Minor);
    }

    #[test]
    fn test_key_signature() {
        let ks = KeySignature::g_major();
        assert_eq!(ks.sharps(), 1);
        assert!(!ks.is_minor());
        assert_eq!(ks.tonic(), Step::G);
    }

    #[test]
    fn test_key_signature_altered() {
        let g_major = KeySignature::g_major();
        assert!(g_major.is_altered(Step::F));
        assert!(!g_major.is_altered(Step::C));

        let f_major = KeySignature::f_major();
        assert!(f_major.is_altered(Step::B));
    }
}
