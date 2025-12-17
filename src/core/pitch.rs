//! Pitch representation
//!
//! A pitch represents a musical note with a specific frequency, combining:
//! - Step (C, D, E, F, G, A, B)
//! - Octave (0-10, where 4 is the middle octave)
//! - Accidental (sharp, flat, etc.)
//! - Optional microtone adjustment

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use super::accidental::{Accidental, Microtone};
use super::{Interval, ParseError};

/// The seven diatonic pitch steps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    C = 0,
    D = 1,
    E = 2,
    F = 3,
    G = 4,
    A = 5,
    B = 6,
}

impl Step {
    /// Get the pitch class (0-11) for this step without accidentals
    pub fn pitch_class(&self) -> u8 {
        match self {
            Step::C => 0,
            Step::D => 2,
            Step::E => 4,
            Step::F => 5,
            Step::G => 7,
            Step::A => 9,
            Step::B => 11,
        }
    }

    /// Get step from diatonic index (0-6)
    pub fn from_index(index: i32) -> Step {
        match index.rem_euclid(7) {
            0 => Step::C,
            1 => Step::D,
            2 => Step::E,
            3 => Step::F,
            4 => Step::G,
            5 => Step::A,
            6 => Step::B,
            _ => unreachable!(),
        }
    }

    /// Get the diatonic index (0-6)
    pub fn index(&self) -> i32 {
        *self as i32
    }

    /// Get the next step
    pub fn next(&self) -> Step {
        Step::from_index(self.index() + 1)
    }

    /// Get the previous step
    pub fn prev(&self) -> Step {
        Step::from_index(self.index() - 1)
    }

    /// Parse step from string
    pub fn from_str(s: &str) -> Result<Step, ParseError> {
        match s.to_uppercase().as_str() {
            "C" => Ok(Step::C),
            "D" => Ok(Step::D),
            "E" => Ok(Step::E),
            "F" => Ok(Step::F),
            "G" => Ok(Step::G),
            "A" => Ok(Step::A),
            "B" => Ok(Step::B),
            _ => Err(ParseError::InvalidStep(s.to_string())),
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Step::C => 'C',
            Step::D => 'D',
            Step::E => 'E',
            Step::F => 'F',
            Step::G => 'G',
            Step::A => 'A',
            Step::B => 'B',
        };
        write!(f, "{}", c)
    }
}

/// A musical pitch combining step, octave, and accidental
#[derive(Debug, Clone)]
pub struct Pitch {
    step: Step,
    octave: Option<i8>,
    accidental: Option<Accidental>,
    microtone: Option<Microtone>,
    /// Whether the spelling was algorithmically inferred
    spelling_is_inferred: bool,
}

impl Pitch {
    /// Create a new pitch with step and octave
    pub fn new(s: &str) -> Result<Pitch, ParseError> {
        s.parse()
    }

    /// Create a pitch from components
    pub fn from_parts(step: Step, octave: Option<i8>, accidental: Option<Accidental>) -> Pitch {
        Pitch {
            step,
            octave,
            accidental,
            microtone: None,
            spelling_is_inferred: false,
        }
    }

    /// Create a pitch from MIDI note number
    pub fn from_midi(midi: u8) -> Pitch {
        let octave = (midi as i8 / 12) - 1;
        let pc = midi % 12;

        // Default spelling for each pitch class
        let (step, accidental) = match pc {
            0 => (Step::C, None),
            1 => (Step::C, Some(Accidental::Sharp)),
            2 => (Step::D, None),
            3 => (Step::E, Some(Accidental::Flat)),
            4 => (Step::E, None),
            5 => (Step::F, None),
            6 => (Step::F, Some(Accidental::Sharp)),
            7 => (Step::G, None),
            8 => (Step::A, Some(Accidental::Flat)),
            9 => (Step::A, None),
            10 => (Step::B, Some(Accidental::Flat)),
            11 => (Step::B, None),
            _ => unreachable!(),
        };

        Pitch {
            step,
            octave: Some(octave),
            accidental,
            microtone: None,
            spelling_is_inferred: true,
        }
    }

    /// Create a pitch from frequency in Hz
    pub fn from_frequency(freq: f64) -> Pitch {
        let midi = 69.0 + 12.0 * (freq / 440.0).log2();
        let midi_rounded = midi.round() as u8;
        let mut pitch = Pitch::from_midi(midi_rounded);

        // Add microtone adjustment if needed
        let cents = (midi - midi_rounded as f64) * 100.0;
        if cents.abs() > 0.5 {
            pitch.microtone = Some(Microtone::new(cents));
        }

        pitch
    }

    /// Get the step (C, D, E, F, G, A, B)
    pub fn step(&self) -> Step {
        self.step
    }

    /// Set the step
    pub fn set_step(&mut self, step: Step) {
        self.step = step;
        self.spelling_is_inferred = false;
    }

    /// Get the octave (None for octaveless pitches)
    pub fn octave(&self) -> Option<i8> {
        self.octave
    }

    /// Get the implicit octave (defaults to 4 if not set)
    pub fn implicit_octave(&self) -> i8 {
        self.octave.unwrap_or(4)
    }

    /// Set the octave
    pub fn set_octave(&mut self, octave: Option<i8>) {
        self.octave = octave;
    }

    /// Get the accidental
    pub fn accidental(&self) -> Option<Accidental> {
        self.accidental
    }

    /// Set the accidental
    pub fn set_accidental(&mut self, accidental: Option<Accidental>) {
        self.accidental = accidental;
        self.spelling_is_inferred = false;
    }

    /// Get the microtone adjustment
    pub fn microtone(&self) -> Option<&Microtone> {
        self.microtone.as_ref()
    }

    /// Set the microtone adjustment
    pub fn set_microtone(&mut self, microtone: Option<Microtone>) {
        self.microtone = microtone;
    }

    /// Get the total alteration in semitones (accidental + microtone)
    pub fn alter(&self) -> f64 {
        let acc_alter = self.accidental.map(|a| a.alter()).unwrap_or(0.0);
        let micro_alter = self.microtone.map(|m| m.alter()).unwrap_or(0.0);
        acc_alter + micro_alter
    }

    /// Get the pitch space value (60.0 = middle C)
    pub fn ps(&self) -> f64 {
        let octave = self.implicit_octave() as f64;
        let base = (octave + 1.0) * 12.0;
        base + self.step.pitch_class() as f64 + self.alter()
    }

    /// Get the MIDI note number (0-127)
    pub fn midi(&self) -> u8 {
        self.ps().round().clamp(0.0, 127.0) as u8
    }

    /// Get the pitch class (0-11)
    pub fn pitch_class(&self) -> u8 {
        (self.ps().round() as i32).rem_euclid(12) as u8
    }

    /// Get the frequency in Hz (A4 = 440 Hz)
    pub fn frequency(&self) -> f64 {
        self.frequency_with_a4(440.0)
    }

    /// Get the frequency with custom A4 reference
    pub fn frequency_with_a4(&self, a4: f64) -> f64 {
        a4 * 2.0_f64.powf((self.ps() - 69.0) / 12.0)
    }

    /// Get the pitch name (step + accidental)
    pub fn name(&self) -> String {
        format!(
            "{}{}",
            self.step,
            self.accidental.map(|a| a.ascii()).unwrap_or("")
        )
    }

    /// Get the full name with octave
    pub fn name_with_octave(&self) -> String {
        match self.octave {
            Some(oct) => format!("{}{}", self.name(), oct),
            None => self.name(),
        }
    }

    /// Transpose by an interval
    pub fn transpose(&self, interval: &Interval) -> Pitch {
        let new_diatonic = self.step.index() + interval.generic();
        let new_step = Step::from_index(new_diatonic);

        // Calculate octave change
        let octave_change = new_diatonic.div_euclid(7);
        let new_octave = self.octave.map(|o| o + octave_change as i8);

        // Calculate the needed alteration
        let current_pc = self.pitch_class() as i32;
        let target_pc = (current_pc + interval.semitones()).rem_euclid(12);
        let new_natural_pc = new_step.pitch_class() as i32;
        let alter_needed = (target_pc - new_natural_pc).rem_euclid(12);

        // Normalize alteration to be in range [-6, 5]
        let alter = if alter_needed > 6 {
            alter_needed - 12
        } else {
            alter_needed
        };

        let accidental = Accidental::from_alter(alter as f64);

        Pitch {
            step: new_step,
            octave: new_octave,
            accidental,
            microtone: self.microtone,
            spelling_is_inferred: false,
        }
    }

    /// Transpose by semitones
    pub fn transpose_semitones(&self, semitones: i32) -> Pitch {
        let new_midi = (self.midi() as i32 + semitones).clamp(0, 127) as u8;
        Pitch::from_midi(new_midi)
    }

    /// Get an enharmonic equivalent
    pub fn enharmonic(&self) -> Pitch {
        let pc = self.pitch_class();

        // Find alternative spelling
        let (new_step, new_acc) = match (self.step, self.accidental) {
            (Step::C, Some(Accidental::Sharp)) => (Step::D, Some(Accidental::Flat)),
            (Step::D, Some(Accidental::Flat)) => (Step::C, Some(Accidental::Sharp)),
            (Step::D, Some(Accidental::Sharp)) => (Step::E, Some(Accidental::Flat)),
            (Step::E, Some(Accidental::Flat)) => (Step::D, Some(Accidental::Sharp)),
            (Step::E, None) => (Step::F, Some(Accidental::Flat)),
            (Step::F, Some(Accidental::Flat)) => (Step::E, None),
            (Step::F, Some(Accidental::Sharp)) => (Step::G, Some(Accidental::Flat)),
            (Step::G, Some(Accidental::Flat)) => (Step::F, Some(Accidental::Sharp)),
            (Step::G, Some(Accidental::Sharp)) => (Step::A, Some(Accidental::Flat)),
            (Step::A, Some(Accidental::Flat)) => (Step::G, Some(Accidental::Sharp)),
            (Step::A, Some(Accidental::Sharp)) => (Step::B, Some(Accidental::Flat)),
            (Step::B, Some(Accidental::Flat)) => (Step::A, Some(Accidental::Sharp)),
            (Step::B, None) => (Step::C, Some(Accidental::Flat)),
            (Step::C, Some(Accidental::Flat)) => (Step::B, None),
            // No simple enharmonic, return a pitch with the same pitch class
            _ => return Pitch::from_midi(pc + 12 * (self.implicit_octave() as u8 + 1)),
        };

        // Adjust octave if crossing B/C boundary
        let mut new_octave = self.octave;
        if let Some(oct) = new_octave {
            if self.step == Step::B && new_step == Step::C {
                new_octave = Some(oct + 1);
            } else if self.step == Step::C && new_step == Step::B {
                new_octave = Some(oct - 1);
            }
        }

        Pitch {
            step: new_step,
            octave: new_octave,
            accidental: new_acc,
            microtone: self.microtone,
            spelling_is_inferred: false,
        }
    }

    /// Check if this pitch is enharmonic with another
    pub fn is_enharmonic(&self, other: &Pitch) -> bool {
        (self.ps() - other.ps()).abs() < 0.01
    }

    /// Simplify the enharmonic spelling (reduce accidentals)
    pub fn simplify_enharmonic(&self) -> Pitch {
        match self.accidental {
            Some(Accidental::DoubleSharp) | Some(Accidental::DoubleFlat) => self.enharmonic(),
            Some(Accidental::Sharp) if self.step == Step::E || self.step == Step::B => {
                self.enharmonic()
            }
            Some(Accidental::Flat) if self.step == Step::F || self.step == Step::C => {
                self.enharmonic()
            }
            _ => self.clone(),
        }
    }

    /// Check if spelling was inferred algorithmically
    pub fn spelling_is_inferred(&self) -> bool {
        self.spelling_is_inferred
    }

    /// Get German pitch name
    pub fn german(&self) -> String {
        let base = match self.step {
            Step::B => "H",
            _ => &self.step.to_string(),
        };

        let suffix = match self.accidental {
            Some(Accidental::Sharp) => "is",
            Some(Accidental::DoubleSharp) => "isis",
            Some(Accidental::Flat) if self.step == Step::B => "",
            Some(Accidental::Flat) => "es",
            Some(Accidental::DoubleFlat) if self.step == Step::B => "es",
            Some(Accidental::DoubleFlat) => "eses",
            _ => "",
        };

        // Special case: B-flat is "B" in German
        if self.step == Step::B && self.accidental == Some(Accidental::Flat) {
            return "B".to_string();
        }

        format!("{}{}", base, suffix)
    }
}

impl FromStr for Pitch {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::InvalidPitch(s.to_string()));
        }

        let s = s.trim();
        let mut chars = s.chars().peekable();

        // Parse step
        let step_char = chars.next().ok_or_else(|| ParseError::InvalidPitch(s.to_string()))?;
        let step = Step::from_str(&step_char.to_string())?;

        // Parse accidental
        let mut accidental_str = String::new();
        while let Some(&c) = chars.peek() {
            if c == '#' || c == 'b' || c == '-' || c == 'x' || c == '~' || c == '`' {
                accidental_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        let accidental = if accidental_str.is_empty() {
            None
        } else {
            Some(Accidental::from_str(&accidental_str)?)
        };

        // Parse octave
        let octave_str: String = chars.collect();
        let octave = if octave_str.is_empty() {
            None
        } else {
            Some(
                octave_str
                    .parse::<i8>()
                    .map_err(|_| ParseError::InvalidOctave(octave_str))?,
            )
        };

        Ok(Pitch {
            step,
            octave,
            accidental,
            microtone: None,
            spelling_is_inferred: false,
        })
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name_with_octave())
    }
}

impl PartialEq for Pitch {
    fn eq(&self, other: &Self) -> bool {
        // Equality requires identical spelling
        self.step == other.step
            && self.octave == other.octave
            && self.accidental == other.accidental
            && self.microtone.map(|m| m.cents()) == other.microtone.map(|m| m.cents())
    }
}

impl Eq for Pitch {}

impl Hash for Pitch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.step.hash(state);
        self.octave.hash(state);
        self.accidental.hash(state);
        // Microtone is not hashable due to f64, so we hash the bits
        if let Some(m) = &self.microtone {
            m.cents().to_bits().hash(state);
        }
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pitch {
    fn cmp(&self, other: &Self) -> Ordering {
        // Ordering based on pitch space (enharmonic equivalents are equal in ordering)
        self.ps().partial_cmp(&other.ps()).unwrap_or(Ordering::Equal)
    }
}

impl Default for Pitch {
    fn default() -> Self {
        Pitch::from_parts(Step::C, Some(4), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_parse() {
        let p = Pitch::new("C4").unwrap();
        assert_eq!(p.step(), Step::C);
        assert_eq!(p.octave(), Some(4));
        assert_eq!(p.accidental(), None);

        let p = Pitch::new("F#5").unwrap();
        assert_eq!(p.step(), Step::F);
        assert_eq!(p.octave(), Some(5));
        assert_eq!(p.accidental(), Some(Accidental::Sharp));

        let p = Pitch::new("Bb3").unwrap();
        assert_eq!(p.step(), Step::B);
        assert_eq!(p.octave(), Some(3));
        assert_eq!(p.accidental(), Some(Accidental::Flat));
    }

    #[test]
    fn test_pitch_midi() {
        assert_eq!(Pitch::new("C4").unwrap().midi(), 60);
        assert_eq!(Pitch::new("A4").unwrap().midi(), 69);
        assert_eq!(Pitch::new("C5").unwrap().midi(), 72);
        assert_eq!(Pitch::new("C#4").unwrap().midi(), 61);
    }

    #[test]
    fn test_pitch_from_midi() {
        let p = Pitch::from_midi(60);
        assert_eq!(p.step(), Step::C);
        assert_eq!(p.octave(), Some(4));

        let p = Pitch::from_midi(61);
        assert_eq!(p.midi(), 61);
    }

    #[test]
    fn test_pitch_frequency() {
        let a4 = Pitch::new("A4").unwrap();
        assert!((a4.frequency() - 440.0).abs() < 0.01);

        let c4 = Pitch::new("C4").unwrap();
        assert!((c4.frequency() - 261.63).abs() < 0.1);
    }

    #[test]
    fn test_pitch_enharmonic() {
        let cs = Pitch::new("C#4").unwrap();
        let db = cs.enharmonic();
        assert_eq!(db.step(), Step::D);
        assert_eq!(db.accidental(), Some(Accidental::Flat));
        assert!(cs.is_enharmonic(&db));
    }

    #[test]
    fn test_pitch_ordering() {
        let c4 = Pitch::new("C4").unwrap();
        let d4 = Pitch::new("D4").unwrap();
        let c5 = Pitch::new("C5").unwrap();

        assert!(c4 < d4);
        assert!(d4 < c5);
        assert!(c4 < c5);
    }
}
