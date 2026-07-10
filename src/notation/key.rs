//! Key signature representation

use std::fmt;

use crate::core::{Accidental, Interval, Pitch, Step};

use super::Scale;

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

    /// Whether this mode is conventionally displayed/spelled like a minor
    /// key (lowercase tonic name, relative-minor-style spelling): minor and
    /// aeolian are the "authentic" minor-like modes; dorian/phrygian/locrian
    /// are also minor-quality modes by their third, and are treated the same
    /// way for display purposes.
    fn is_minor_like(&self) -> bool {
        matches!(
            self,
            KeyMode::Minor | KeyMode::Aeolian | KeyMode::Dorian | KeyMode::Phrygian | KeyMode::Locrian
        )
    }
}

impl fmt::Display for KeyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// (step, accidental) tonics for major keys, indexed by `sharps + 7`
/// (sharps ranges -7..=7, positive = sharps, negative = flats).
const MAJOR_TONICS: [(Step, Option<Accidental>); 15] = [
    (Step::C, Some(Accidental::Flat)),  // -7: Cb
    (Step::G, Some(Accidental::Flat)),  // -6: Gb
    (Step::D, Some(Accidental::Flat)),  // -5: Db
    (Step::A, Some(Accidental::Flat)),  // -4: Ab
    (Step::E, Some(Accidental::Flat)),  // -3: Eb
    (Step::B, Some(Accidental::Flat)),  // -2: Bb
    (Step::F, None),                    // -1: F
    (Step::C, None),                    //  0: C
    (Step::G, None),                    //  1: G
    (Step::D, None),                    //  2: D
    (Step::A, None),                    //  3: A
    (Step::E, None),                    //  4: E
    (Step::B, None),                    //  5: B
    (Step::F, Some(Accidental::Sharp)), //  6: F#
    (Step::C, Some(Accidental::Sharp)), //  7: C#
];

/// (step, accidental) tonics for minor keys, indexed the same way as
/// `MAJOR_TONICS` (by the shared key signature's sharps count).
const MINOR_TONICS: [(Step, Option<Accidental>); 15] = [
    (Step::A, Some(Accidental::Flat)),  // -7: Ab
    (Step::E, Some(Accidental::Flat)),  // -6: Eb
    (Step::B, Some(Accidental::Flat)),  // -5: Bb
    (Step::F, None),                    // -4: F
    (Step::C, None),                    // -3: C
    (Step::G, None),                    // -2: G
    (Step::D, None),                    // -1: D
    (Step::A, None),                    //  0: A
    (Step::E, None),                    //  1: E
    (Step::B, None),                    //  2: B
    (Step::F, Some(Accidental::Sharp)), //  3: F#
    (Step::C, Some(Accidental::Sharp)), //  4: C#
    (Step::G, Some(Accidental::Sharp)), //  5: G#
    (Step::D, Some(Accidental::Sharp)), //  6: D#
    (Step::A, Some(Accidental::Sharp)), //  7: A#
];

/// Convert a sharps count (-7..=7; positive = sharps, negative = flats) and
/// major/minor-ness into the standard tonic `Pitch` for that key signature —
/// e.g. `sharps_to_pitch(3, false)` is A, `sharps_to_pitch(3, true)` is F#.
/// Out-of-range sharps counts are clamped to -7..=7. Mirrors music21's
/// `key.sharpsToPitch` (generalized here to also take the mode).
pub fn sharps_to_pitch(sharps: i8, minor: bool) -> Pitch {
    let idx = (sharps.clamp(-7, 7) + 7) as usize;
    let (step, accidental) = if minor { MINOR_TONICS[idx] } else { MAJOR_TONICS[idx] };
    Pitch::from_parts(step, None, accidental)
}

/// Inverse of `sharps_to_pitch`: given a tonic pitch and mode, find the
/// matching sharps count for one of the 15 standard (-7..=7) key
/// signatures, or `None` if the pitch doesn't correspond to any of them in
/// that mode (e.g. a double-sharp tonic). Mirrors music21's
/// `key.pitchToSharps`.
pub fn pitch_to_sharps(pitch: &Pitch, minor: bool) -> Option<i8> {
    (-7..=7i8).find(|&s| {
        let candidate = sharps_to_pitch(s, minor);
        candidate.step() == pitch.step() && candidate.accidental() == pitch.accidental()
    })
}

/// A musical key (tonic + mode)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// The tonic pitch (letter name + accidental)
    tonic: Pitch,
    /// The mode
    mode: KeyMode,
}

impl Key {
    /// Create a new key from an explicit tonic pitch (letter + accidental)
    /// and mode, e.g. `Key::new(Pitch::from_parts(Step::F, None,
    /// Some(Accidental::Sharp)), KeyMode::Minor)` for F# minor.
    pub fn new(tonic: Pitch, mode: KeyMode) -> Self {
        Self { tonic, mode }
    }

    /// Create a major key from a natural-letter tonic (no accidental). For
    /// keys with an altered tonic (e.g. F# major), use `Key::new` with a
    /// full `Pitch`.
    pub fn major(tonic: Step) -> Self {
        Self::new(Pitch::from_parts(tonic, None, None), KeyMode::Major)
    }

    /// Create a minor key from a natural-letter tonic (no accidental). For
    /// keys with an altered tonic (e.g. F# minor), use `Key::new` with a
    /// full `Pitch`.
    pub fn minor(tonic: Step) -> Self {
        Self::new(Pitch::from_parts(tonic, None, None), KeyMode::Minor)
    }

    /// Get the tonic pitch
    pub fn tonic(&self) -> &Pitch {
        &self.tonic
    }

    /// Get the mode
    pub fn mode(&self) -> KeyMode {
        self.mode
    }

    /// Get the relative major/minor. Only defined (as a real transformation)
    /// for `Major`/`Aeolian` <-> `Minor`; other modes return a clone of self.
    /// Returns a clone of self if the tonic doesn't correspond to one of the
    /// 15 standard key signatures.
    pub fn relative(&self) -> Key {
        match self.mode {
            KeyMode::Major | KeyMode::Aeolian => match pitch_to_sharps(&self.tonic, false) {
                Some(sharps) => Key::new(sharps_to_pitch(sharps, true), KeyMode::Minor),
                None => self.clone(),
            },
            KeyMode::Minor => match pitch_to_sharps(&self.tonic, true) {
                Some(sharps) => Key::new(sharps_to_pitch(sharps, false), KeyMode::Major),
                None => self.clone(),
            },
            _ => self.clone(),
        }
    }

    /// Get the parallel major/minor (same tonic, opposite mode).
    pub fn parallel(&self) -> Key {
        match self.mode {
            KeyMode::Major => Key::new(self.tonic.clone(), KeyMode::Minor),
            KeyMode::Minor => Key::new(self.tonic.clone(), KeyMode::Major),
            _ => self.clone(),
        }
    }

    /// Transpose this key by an interval, keeping the same mode.
    pub fn transpose(&self, interval: &Interval) -> Key {
        Key::new(self.tonic.transpose(interval), self.mode)
    }

    /// Get the key name (e.g., "C major", "F# minor")
    pub fn name(&self) -> String {
        format!("{} {}", self.tonic.name(), self.mode)
    }

    /// Get the tonic pitch name with conventional case (lowercase for
    /// minor-quality modes, e.g. "f#" for F# minor; uppercase otherwise,
    /// e.g. "C" for C major).
    pub fn tonic_pitch_name_with_case(&self) -> String {
        let name = self.tonic.name();
        if self.mode.is_minor_like() {
            name.to_lowercase()
        } else {
            name
        }
    }

    /// Construct a `Key` (same mode as `self`) such that `pitch_ref` is the
    /// given scale degree (1-indexed) of the resulting key. Used e.g. for
    /// secondary-dominant roman numeral resolution — finding the temporary
    /// tonic that a figure like "V/V" tonicizes. Returns `None` for
    /// `degree` outside 1..=7.
    pub fn derive_by_degree(&self, degree: u8, pitch_ref: &Pitch) -> Option<Key> {
        if degree == 0 || degree > 7 {
            return None;
        }
        let intervals = Scale::intervals_for_mode(self.mode);
        let offset = intervals[(degree - 1) as usize];
        let tonic_pc = (pitch_ref.pitch_class() as i32 - offset).rem_euclid(12);
        // Pick a reasonable default spelling for the derived tonic; exact
        // traditional spelling (matching the key signature convention)
        // would require the same sharps/flats table as `sharps_to_pitch`,
        // which only covers the 15 standard signatures, not arbitrary
        // tonics reachable via this method.
        let tonic_pitch = Pitch::from_midi(60 + tonic_pc as u8);
        Some(Key::new(tonic_pitch, self.mode))
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

    /// Whether this signature falls outside the 15 standard traditional
    /// signatures (-7..=7 sharps). Signatures built via `new`/`from_sharps`/
    /// `from_flats` with a count outside that range are non-traditional
    /// (e.g. mixed sharps/flats or microtonal signatures aren't
    /// representable by this struct at all, so this only catches
    /// out-of-range counts).
    pub fn is_non_traditional(&self) -> bool {
        !(-7..=7).contains(&self.sharps)
    }

    /// Get the tonic pitch (letter name + accidental) for this key
    /// signature. For all 15 standard signatures (-7..=7 sharps) this
    /// returns the correct accidental for minor keys too — e.g. the tonic
    /// of a 3-sharp minor signature is F# (the actual relative minor of A
    /// major), not the bare letter "F".
    pub fn tonic(&self) -> Pitch {
        sharps_to_pitch(self.sharps, self.minor)
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
    pub fn accidental_for(&self, step: Step) -> Option<Accidental> {
        if self.is_altered(step) {
            if self.sharps > 0 {
                Some(Accidental::Sharp)
            } else {
                Some(Accidental::Flat)
            }
        } else {
            None
        }
    }

    /// Convert to a `Key` using this signature's own major/minor flag.
    pub fn to_key(&self) -> Key {
        let mode = if self.minor {
            KeyMode::Minor
        } else {
            KeyMode::Major
        };
        Key::new(self.tonic(), mode)
    }

    /// Build a `Key` for an explicit mode built on this key signature.
    /// Major/minor are exact (using the same sharps table as `tonic`).
    /// Other modes (dorian, phrygian, etc.) currently approximate using the
    /// major tonic — full scale-degree-correct modal tonics require a scale
    /// abstraction and are not yet implemented.
    pub fn as_key(&self, mode: KeyMode) -> Key {
        match mode {
            KeyMode::Major => Key::new(sharps_to_pitch(self.sharps, false), KeyMode::Major),
            KeyMode::Minor => Key::new(sharps_to_pitch(self.sharps, true), KeyMode::Minor),
            other => Key::new(sharps_to_pitch(self.sharps, false), other),
        }
    }

    /// Build the `Scale` for this key signature in a given mode.
    pub fn get_scale(&self, mode: KeyMode) -> Scale {
        let key = self.as_key(mode);
        Scale::new(key.tonic().clone(), mode)
    }

    /// Transpose a pitch by the same interval that separates C from this
    /// signature's major tonic (e.g. for a 2-sharp signature, D major,
    /// transposing "C" gives "D").
    pub fn transpose_pitch_from_c(&self, pitch: &Pitch) -> Pitch {
        let major_tonic = sharps_to_pitch(self.sharps, false);
        let c = Pitch::from_parts(Step::C, None, None);
        let shift = (major_tonic.pitch_class() as i32 - c.pitch_class() as i32).rem_euclid(12);
        pitch.transpose_semitones(shift)
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
        assert_eq!(key.tonic().step(), Step::C);
        assert_eq!(key.tonic().accidental(), None);
        assert_eq!(key.mode(), KeyMode::Major);
    }

    #[test]
    fn test_key_relative() {
        let c_major = Key::major(Step::C);
        let relative = c_major.relative();
        assert_eq!(relative.tonic().step(), Step::A);
        assert_eq!(relative.tonic().accidental(), None);
        assert_eq!(relative.mode(), KeyMode::Minor);
    }

    #[test]
    fn test_sharp_minor_key_is_correctly_spelled() {
        // Regression test: the relative minor of A major (3 sharps) is F#
        // minor, not "F minor" (a bare, unaltered letter). Both the Key
        // API and KeySignature::tonic() must agree.
        let a_major = Key::major(Step::A);
        let relative = a_major.relative();
        assert_eq!(relative.tonic().step(), Step::F);
        assert_eq!(relative.tonic().accidental(), Some(Accidental::Sharp));
        assert_eq!(relative.name(), "F# minor");

        let three_sharps_minor = KeySignature::new(3, true);
        let tonic = three_sharps_minor.tonic();
        assert_eq!(tonic.step(), Step::F);
        assert_eq!(tonic.accidental(), Some(Accidental::Sharp));
    }

    #[test]
    fn test_can_construct_previously_unrepresentable_keys() {
        // Before this fix, Key::tonic was a bare Step, so keys like F#
        // minor, Bb major, C# major, Db major could not be constructed at
        // all (only the 7 natural letters were representable).
        let f_sharp_minor = Key::new(
            Pitch::from_parts(Step::F, None, Some(Accidental::Sharp)),
            KeyMode::Minor,
        );
        assert_eq!(f_sharp_minor.name(), "F# minor");

        let b_flat_major = Key::new(
            Pitch::from_parts(Step::B, None, Some(Accidental::Flat)),
            KeyMode::Major,
        );
        assert_eq!(b_flat_major.name(), "Bb major");
    }

    #[test]
    fn test_sharps_to_pitch_and_back_roundtrip() {
        for sharps in -7..=7i8 {
            for minor in [false, true] {
                let pitch = sharps_to_pitch(sharps, minor);
                assert_eq!(pitch_to_sharps(&pitch, minor), Some(sharps));
            }
        }
    }

    #[test]
    fn test_key_signature() {
        let ks = KeySignature::g_major();
        assert_eq!(ks.sharps(), 1);
        assert!(!ks.is_minor());
        assert_eq!(ks.tonic().step(), Step::G);
        assert_eq!(ks.tonic().accidental(), None);
    }

    #[test]
    fn test_key_signature_altered() {
        let g_major = KeySignature::g_major();
        assert!(g_major.is_altered(Step::F));
        assert!(!g_major.is_altered(Step::C));

        let f_major = KeySignature::f_major();
        assert!(f_major.is_altered(Step::B));
    }

    #[test]
    fn test_key_signature_as_key_and_non_traditional() {
        let three_sharps = KeySignature::new(3, false);
        assert!(!three_sharps.is_non_traditional());
        assert_eq!(three_sharps.as_key(KeyMode::Minor).name(), "F# minor");

        let out_of_range = KeySignature::new(10, false);
        assert!(out_of_range.is_non_traditional());
    }

    #[test]
    fn test_tonic_pitch_name_with_case() {
        let f_sharp_minor = Key::new(
            Pitch::from_parts(Step::F, None, Some(Accidental::Sharp)),
            KeyMode::Minor,
        );
        assert_eq!(f_sharp_minor.tonic_pitch_name_with_case(), "f#");

        let c_major = Key::major(Step::C);
        assert_eq!(c_major.tonic_pitch_name_with_case(), "C");
    }

    #[test]
    fn test_transpose_pitch_from_c() {
        let d_major_sig = KeySignature::new(2, false);
        let c = Pitch::from_parts(Step::C, None, None);
        let transposed = d_major_sig.transpose_pitch_from_c(&c);
        assert_eq!(transposed.step(), Step::D);
    }

    #[test]
    fn test_key_signature_get_scale() {
        let g_major_sig = KeySignature::g_major();
        let scale = g_major_sig.get_scale(KeyMode::Major);
        let names: Vec<String> = scale.pitches().iter().map(|p| p.name()).collect();
        assert_eq!(names, vec!["G", "A", "B", "C", "D", "E", "F#"]);
    }

    #[test]
    fn test_key_derive_by_degree() {
        // Secondary dominant use case: what key has D as its 5th degree
        // (i.e. what key's dominant is D, so "V" of that key resolves via
        // D)? That's G major.
        let major = Key::major(Step::C);
        let d = Pitch::from_parts(Step::D, None, None);
        let derived = major.derive_by_degree(5, &d).unwrap();
        assert_eq!(derived.tonic().step(), Step::G);
        assert_eq!(derived.mode(), KeyMode::Major);

        assert!(major.derive_by_degree(0, &d).is_none());
        assert!(major.derive_by_degree(8, &d).is_none());
    }
}
