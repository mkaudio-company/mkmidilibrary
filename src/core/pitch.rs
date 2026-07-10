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

use super::accidental::{Accidental, AccidentalDisplay, AccidentalDisplayType, Microtone};
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
}

impl FromStr for Step {
    type Err = ParseError;

    /// Parse step from string
    fn from_str(s: &str) -> Result<Step, ParseError> {
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

    /// The accidental (and its signed alteration) needed to respell this
    /// pitch's pitch class using `new_step` as the letter name. `None`
    /// (rather than `Some(Accidental::Natural)`) represents no alteration,
    /// matching this crate's convention elsewhere (e.g. `from_midi`).
    fn respell_as(&self, new_step: Step) -> (Option<Accidental>, i32) {
        let pc = self.pitch_class() as i32;
        let natural_pc = new_step.pitch_class() as i32;
        let mut alter = (pc - natural_pc).rem_euclid(12);
        if alter > 6 {
            alter -= 12;
        }
        let accidental = if alter == 0 {
            None
        } else {
            Accidental::from_alter(alter as f64)
        };
        (accidental, alter)
    }

    /// The octave `new_step` would land in when moving one letter name up
    /// (`is_next = true`) or down (`is_next = false`) from this pitch's own
    /// step, accounting for the B/C octave boundary.
    fn octave_for_step_change(&self, is_next: bool) -> Option<i8> {
        self.octave.map(|oct| {
            if is_next && self.step == Step::B {
                oct + 1
            } else if !is_next && self.step == Step::C {
                oct - 1
            } else {
                oct
            }
        })
    }

    /// Respell this pitch using the next letter name up (e.g. C# -> Dbb),
    /// regardless of whether that's the simplest respelling — see
    /// `enharmonic()` for the "pick whichever is simpler" version.
    pub fn get_higher_enharmonic(&self) -> Pitch {
        let new_step = self.step.next();
        let (accidental, _) = self.respell_as(new_step);
        Pitch {
            step: new_step,
            octave: self.octave_for_step_change(true),
            accidental,
            microtone: self.microtone,
            spelling_is_inferred: false,
        }
    }

    /// Respell this pitch using the next letter name down (e.g. C# -> B##).
    pub fn get_lower_enharmonic(&self) -> Pitch {
        let new_step = self.step.prev();
        let (accidental, _) = self.respell_as(new_step);
        Pitch {
            step: new_step,
            octave: self.octave_for_step_change(false),
            accidental,
            microtone: self.microtone,
            spelling_is_inferred: false,
        }
    }

    /// Get an enharmonic equivalent: whichever of the adjacent-letter
    /// respellings (`get_higher_enharmonic`/`get_lower_enharmonic`) needs
    /// the smaller accidental (ties favor the higher letter name). Unlike
    /// the previous implementation, this always returns a genuinely
    /// different spelling — including for naturals like C/D/G/A, which
    /// previously fell through to a no-op returning the same pitch.
    pub fn enharmonic(&self) -> Pitch {
        let (_, alter_next) = self.respell_as(self.step.next());
        let (_, alter_prev) = self.respell_as(self.step.prev());

        if alter_next.abs() <= alter_prev.abs() {
            self.get_higher_enharmonic()
        } else {
            self.get_lower_enharmonic()
        }
    }

    /// Get all "common" enharmonic respellings of this pitch — the
    /// adjacent-letter-name respellings (see `get_higher_enharmonic`/
    /// `get_lower_enharmonic`) whose required accidental's alteration
    /// magnitude is at most `alter_limit`. Respellings two or more letters
    /// away (which would need 3+ accidentals) are not considered "common"
    /// and are not included.
    pub fn get_all_common_enharmonics(&self, alter_limit: i32) -> Vec<Pitch> {
        let mut results = Vec::new();

        let (_, alter_next) = self.respell_as(self.step.next());
        if alter_next.abs() <= alter_limit {
            results.push(self.get_higher_enharmonic());
        }

        let (_, alter_prev) = self.respell_as(self.step.prev());
        if alter_prev.abs() <= alter_limit {
            results.push(self.get_lower_enharmonic());
        }

        results
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

    /// Get the pitch name using Unicode accidental symbols (♯/♭/♮/𝄪/𝄫)
    /// instead of ASCII (#/b).
    pub fn unicode_name(&self) -> String {
        format!(
            "{}{}",
            self.step,
            self.accidental.map(|a| a.unicode()).unwrap_or("")
        )
    }

    /// Get the full Unicode pitch name with octave.
    pub fn unicode_name_with_octave(&self) -> String {
        match self.octave {
            Some(oct) => format!("{}{}", self.unicode_name(), oct),
            None => self.unicode_name(),
        }
    }

    /// The base solfège syllable for this pitch's letter name (shared by
    /// the Italian/French/Spanish naming conventions, which mainly differ
    /// in how they name the accidental).
    fn solfege_syllable(&self) -> &'static str {
        match self.step {
            Step::C => "Do",
            Step::D => "Re",
            Step::E => "Mi",
            Step::F => "Fa",
            Step::G => "Sol",
            Step::A => "La",
            Step::B => "Si",
        }
    }

    /// Get the Italian solfège name (e.g. "Fa diesis" for F#).
    pub fn italian(&self) -> String {
        let base = self.solfege_syllable();
        match self.accidental {
            Some(Accidental::Sharp) => format!("{base} diesis"),
            Some(Accidental::DoubleSharp) => format!("{base} doppio diesis"),
            Some(Accidental::Flat) => format!("{base} bemolle"),
            Some(Accidental::DoubleFlat) => format!("{base} doppio bemolle"),
            _ => base.to_string(),
        }
    }

    /// Get the French solfège name (e.g. "Fa dièse" for F#).
    pub fn french(&self) -> String {
        let base = self.solfege_syllable();
        match self.accidental {
            Some(Accidental::Sharp) => format!("{base} di\u{e8}se"),
            Some(Accidental::DoubleSharp) => format!("{base} double di\u{e8}se"),
            Some(Accidental::Flat) => format!("{base} b\u{e9}mol"),
            Some(Accidental::DoubleFlat) => format!("{base} double b\u{e9}mol"),
            _ => base.to_string(),
        }
    }

    /// Get the Spanish solfège name (e.g. "Fa sostenido" for F#).
    pub fn spanish(&self) -> String {
        let base = self.solfege_syllable();
        match self.accidental {
            Some(Accidental::Sharp) => format!("{base} sostenido"),
            Some(Accidental::DoubleSharp) => format!("{base} doble sostenido"),
            Some(Accidental::Flat) => format!("{base} bemol"),
            Some(Accidental::DoubleFlat) => format!("{base} doble bemol"),
            _ => base.to_string(),
        }
    }

    /// Get the absolute diatonic step count (letter name only, ignoring
    /// accidental) — a self-consistent numbering where each octave spans 7
    /// consecutive numbers and `set_diatonic_note_num` is its exact
    /// inverse. Useful for diatonic (letter-name-based) arithmetic that
    /// should ignore accidentals, e.g. counting scale-step distances.
    pub fn diatonic_note_num(&self) -> i32 {
        (self.implicit_octave() as i32 + 1) * 7 + self.step.index() + 1
    }

    /// Set this pitch's step and octave from an absolute diatonic step
    /// count (see `diatonic_note_num`). Leaves the accidental unchanged.
    pub fn set_diatonic_note_num(&mut self, num: i32) {
        let n = num - 1;
        self.octave = Some((n.div_euclid(7) - 1) as i8);
        self.step = Step::from_index(n.rem_euclid(7));
        self.spelling_is_inferred = false;
    }

    /// Get the pitch of the `number`th harmonic above this pitch treated as
    /// a fundamental (e.g. the 2nd harmonic is one octave up, the 3rd is an
    /// octave and a fifth up).
    pub fn get_harmonic(&self, number: u32) -> Pitch {
        Pitch::from_frequency(self.frequency() * number.max(1) as f64)
    }

    /// Given a candidate fundamental, determine which harmonic number this
    /// pitch most closely represents, and how many cents sharp/flat it is
    /// from that harmonic's exact frequency.
    pub fn harmonic_from_fundamental(&self, fundamental: &Pitch) -> (u32, f64) {
        let ratio = self.frequency() / fundamental.frequency();
        let n = ratio.round().max(1.0) as u32;
        let exact_freq = fundamental.frequency() * n as f64;
        let cents = 1200.0 * (self.frequency() / exact_freq).log2();
        (n, cents)
    }

    /// Get the fundamental pitch such that this pitch is its `number`th
    /// harmonic.
    pub fn fundamental_from_harmonic(&self, number: u32) -> Pitch {
        Pitch::from_frequency(self.frequency() / number.max(1) as f64)
    }

    /// Describe this pitch's relationship to a candidate fundamental as a
    /// human-readable harmonic label, e.g. "5th harmonic" or "5th harmonic
    /// (+14 cents)" if it deviates from the exact harmonic frequency.
    pub fn harmonic_string(&self, fundamental: &Pitch) -> String {
        let (n, cents) = self.harmonic_from_fundamental(fundamental);
        let suffix = match n % 10 {
            1 if n % 100 != 11 => "st",
            2 if n % 100 != 12 => "nd",
            3 if n % 100 != 13 => "rd",
            _ => "th",
        };
        if cents.abs() < 0.5 {
            format!("{n}{suffix} harmonic")
        } else {
            format!("{n}{suffix} harmonic ({cents:+.0} cents)")
        }
    }

    /// Convert a quarter-tone accidental (quarter/three-quarter sharp or
    /// flat) into an equivalent standard accidental plus a Microtone
    /// adjustment (e.g. quarter-sharp -> natural + 50 cents). Pitches
    /// without a quarter-tone accidental are returned unchanged.
    pub fn convert_quarter_tones_to_microtones(&self) -> Pitch {
        let (new_accidental, extra_cents) = match self.accidental {
            Some(Accidental::QuarterSharp) => (None, 50.0),
            Some(Accidental::QuarterFlat) => (None, -50.0),
            Some(Accidental::ThreeQuarterSharp) => (Some(Accidental::Sharp), 50.0),
            Some(Accidental::ThreeQuarterFlat) => (Some(Accidental::Flat), -50.0),
            _ => return self.clone(),
        };
        let existing_cents = self.microtone.map(|m| m.cents()).unwrap_or(0.0);
        Pitch {
            step: self.step,
            octave: self.octave,
            accidental: new_accidental,
            microtone: Some(Microtone::new(existing_cents + extra_cents)),
            spelling_is_inferred: false,
        }
    }

    /// Inverse of `convert_quarter_tones_to_microtones`: fold a ±50-cent
    /// microtone on top of a natural/sharp/flat accidental into the
    /// equivalent quarter-tone accidental, clearing the microtone. Any
    /// other microtone/accidental combination is left unchanged (no
    /// quarter-tone accidental can represent it).
    pub fn convert_microtones_to_quarter_tones(&self) -> Pitch {
        let cents = match self.microtone {
            Some(m) => m.cents(),
            None => return self.clone(),
        };
        let base = self.accidental.unwrap_or(Accidental::Natural);
        let new_accidental = match (base, cents) {
            (Accidental::Natural, c) if (c - 50.0).abs() < 0.5 => Accidental::QuarterSharp,
            (Accidental::Natural, c) if (c + 50.0).abs() < 0.5 => Accidental::QuarterFlat,
            (Accidental::Sharp, c) if (c - 50.0).abs() < 0.5 => Accidental::ThreeQuarterSharp,
            (Accidental::Flat, c) if (c + 50.0).abs() < 0.5 => Accidental::ThreeQuarterFlat,
            _ => return self.clone(),
        };
        Pitch {
            step: self.step,
            octave: self.octave,
            accidental: Some(new_accidental),
            microtone: None,
            spelling_is_inferred: false,
        }
    }

    /// Adjust this pitch's octave (keeping step/accidental fixed) to the
    /// highest octave at which it falls at or below `target` in pitch
    /// space.
    pub fn transpose_below_target(&self, target: &Pitch) -> Pitch {
        let mut result = self.clone();
        while result.ps() > target.ps() {
            result.octave = Some(result.implicit_octave() - 1);
        }
        while result.ps() + 12.0 <= target.ps() {
            result.octave = Some(result.implicit_octave() + 1);
        }
        result
    }

    /// Adjust this pitch's octave (keeping step/accidental fixed) to the
    /// lowest octave at which it falls at or above `target` in pitch space.
    pub fn transpose_above_target(&self, target: &Pitch) -> Pitch {
        let mut result = self.clone();
        while result.ps() < target.ps() {
            result.octave = Some(result.implicit_octave() + 1);
        }
        while result.ps() - 12.0 >= target.ps() {
            result.octave = Some(result.implicit_octave() - 1);
        }
        result
    }
}

/// Compute accidental display status for a sequence of pitches within a
/// single measure, given the prevailing key signature. Mirrors music21's
/// `Pitch.updateAccidentalDisplay` (invoked internally by
/// `Stream.makeAccidentals` for a whole measure — see the stream-level
/// notation pipeline for how this feeds into a real score).
///
/// An accidental is only shown if it isn't already implied by the key
/// signature, or if an earlier note in the same measure displayed a
/// *different* accidental for the same step (requiring a new symbol —
/// including a natural sign — to cancel it). Once a step's accidental has
/// been shown in the measure, later notes with the same accidental on that
/// step don't need to redisplay it.
///
/// A single `KeySignature` is used for the whole slice; if it changes
/// mid-measure, call this separately for each sub-range.
pub fn update_accidental_display(
    pitches: &[Pitch],
    key_signature: &crate::notation::KeySignature,
) -> Vec<AccidentalDisplay> {
    let mut last_shown: std::collections::HashMap<Step, Option<Accidental>> =
        std::collections::HashMap::new();
    let mut results = Vec::with_capacity(pitches.len());

    for pitch in pitches {
        let step = pitch.step();
        let implied = key_signature.accidental_for(step);
        let effective_previous = last_shown.get(&step).copied().unwrap_or(implied);
        let needs_display = pitch.accidental() != effective_previous;

        let mut display = AccidentalDisplay::new(pitch.accidental().unwrap_or(Accidental::Natural));
        display.display_status = Some(needs_display);
        display.display_type = AccidentalDisplayType::IfNeeded;
        results.push(display);

        last_shown.insert(step, pitch.accidental());
    }

    results
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
    fn test_pitch_enharmonic_naturals_are_not_no_ops() {
        // Regression test: enharmonic() used to silently fall through to a
        // same-pitch-class-same-spelling result for naturals like D/G/A
        // (and any double-sharp/flat pitch), returning the identical
        // pitch instead of a genuine respelling.
        for step in [Step::C, Step::D, Step::G, Step::A] {
            let p = Pitch::from_parts(step, Some(4), None);
            let enh = p.enharmonic();
            assert_ne!(
                (enh.step(), enh.accidental()),
                (p.step(), p.accidental()),
                "enharmonic() of {step} was a no-op"
            );
            assert!(p.is_enharmonic(&enh));
        }
    }

    #[test]
    fn test_pitch_enharmonic_double_sharp_simplifies() {
        // C## and D (natural) are the same pitch class; enharmonic() should
        // find this exact, zero-alteration respelling.
        let css = Pitch::from_parts(Step::C, Some(4), Some(Accidental::DoubleSharp));
        let enh = css.enharmonic();
        assert_eq!(enh.step(), Step::D);
        assert_eq!(enh.accidental(), None);
    }

    #[test]
    fn test_get_higher_lower_enharmonic() {
        let cs = Pitch::from_parts(Step::C, Some(4), Some(Accidental::Sharp));
        let higher = cs.get_higher_enharmonic();
        assert_eq!(higher.step(), Step::D);
        assert_eq!(higher.accidental(), Some(Accidental::Flat));

        let lower = cs.get_lower_enharmonic();
        assert_eq!(lower.step(), Step::B);
        assert_eq!(lower.accidental(), Some(Accidental::DoubleSharp));

        // B4 -> C5 crosses the octave boundary upward.
        let b4 = Pitch::from_parts(Step::B, Some(4), None);
        assert_eq!(b4.get_higher_enharmonic().octave(), Some(5));
        // C4 -> B3 crosses the octave boundary downward.
        let c4 = Pitch::from_parts(Step::C, Some(4), None);
        assert_eq!(c4.get_lower_enharmonic().octave(), Some(3));
    }

    #[test]
    fn test_get_all_common_enharmonics() {
        let cs = Pitch::from_parts(Step::C, Some(4), Some(Accidental::Sharp));
        let common = cs.get_all_common_enharmonics(1);
        assert_eq!(common.len(), 1); // only Db (alter -1); B## has alter +2, excluded
        assert_eq!(common[0].step(), Step::D);

        let common_wide = cs.get_all_common_enharmonics(2);
        assert_eq!(common_wide.len(), 2);
    }

    #[test]
    fn test_unicode_name() {
        let cs = Pitch::from_parts(Step::C, Some(4), Some(Accidental::Sharp));
        assert_eq!(cs.unicode_name(), "C\u{266F}");
        assert_eq!(cs.unicode_name_with_octave(), "C\u{266F}4");
    }

    #[test]
    fn test_foreign_names() {
        let fs = Pitch::from_parts(Step::F, Some(4), Some(Accidental::Sharp));
        assert_eq!(fs.italian(), "Fa diesis");
        assert_eq!(fs.french(), "Fa di\u{e8}se");
        assert_eq!(fs.spanish(), "Fa sostenido");

        let c = Pitch::from_parts(Step::C, Some(4), None);
        assert_eq!(c.italian(), "Do");
    }

    #[test]
    fn test_diatonic_note_num_roundtrip() {
        let c4 = Pitch::from_parts(Step::C, Some(4), Some(Accidental::Sharp));
        let num = c4.diatonic_note_num();

        let mut p = Pitch::from_parts(Step::C, Some(0), None);
        p.set_diatonic_note_num(num);
        assert_eq!(p.step(), Step::C);
        assert_eq!(p.octave(), Some(4));
        // Accidental is preserved (set_diatonic_note_num only touches step/octave).
        assert_eq!(p.accidental(), None);
    }

    #[test]
    fn test_harmonics() {
        let c4 = Pitch::from_parts(Step::C, Some(4), None);
        let second_harmonic = c4.get_harmonic(2);
        // 2nd harmonic of a fundamental is exactly one octave up.
        assert!((second_harmonic.frequency() - c4.frequency() * 2.0).abs() < 0.01);

        let (n, cents) = second_harmonic.harmonic_from_fundamental(&c4);
        assert_eq!(n, 2);
        assert!(cents.abs() < 0.1);

        let fundamental = second_harmonic.fundamental_from_harmonic(2);
        assert!((fundamental.frequency() - c4.frequency()).abs() < 0.01);

        assert_eq!(second_harmonic.harmonic_string(&c4), "2nd harmonic");
    }

    #[test]
    fn test_quarter_tone_conversion_roundtrip() {
        let qs = Pitch::from_parts(Step::C, Some(4), Some(Accidental::QuarterSharp));
        let converted = qs.convert_quarter_tones_to_microtones();
        assert_eq!(converted.accidental(), None);
        assert_eq!(converted.microtone().unwrap().cents(), 50.0);

        let back = converted.convert_microtones_to_quarter_tones();
        assert_eq!(back.accidental(), Some(Accidental::QuarterSharp));
        assert_eq!(back.microtone(), None);
    }

    #[test]
    fn test_transpose_below_above_target() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let target = Pitch::from_parts(Step::A, Some(2), None);

        let below = c.transpose_below_target(&target);
        assert!(below.ps() <= target.ps());
        assert!(below.ps() + 12.0 > target.ps());

        let above = c.transpose_above_target(&target);
        assert!(above.ps() >= target.ps());
        assert!(above.ps() - 12.0 < target.ps());
    }

    #[test]
    fn test_update_accidental_display_basic() {
        use crate::notation::KeySignature;

        // G major (1 sharp: F#). An F# matches the key signature and
        // shouldn't display; a plain F must display a natural to cancel.
        let g_major = KeySignature::g_major();
        let pitches = vec![
            Pitch::from_parts(Step::F, Some(4), Some(Accidental::Sharp)),
            Pitch::from_parts(Step::F, Some(4), None),
        ];
        let display = update_accidental_display(&pitches, &g_major);

        assert_eq!(display[0].display_status, Some(false)); // F# matches key sig
        assert_eq!(display[1].display_status, Some(true)); // natural cancels it
    }

    #[test]
    fn test_update_accidental_display_repeated_accidental_not_redisplayed() {
        use crate::notation::KeySignature;

        // C major (no sharps/flats): first F# must display; a second F#
        // later in the same measure shouldn't need to redisplay it.
        let c_major = KeySignature::c_major();
        let pitches = vec![
            Pitch::from_parts(Step::F, Some(4), Some(Accidental::Sharp)),
            Pitch::from_parts(Step::C, Some(4), None), // unrelated note in between
            Pitch::from_parts(Step::F, Some(5), Some(Accidental::Sharp)),
        ];
        let display = update_accidental_display(&pitches, &c_major);

        assert_eq!(display[0].display_status, Some(true));
        assert_eq!(display[2].display_status, Some(false));
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
