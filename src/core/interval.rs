//! Musical intervals
//!
//! An interval represents the distance between two pitches,
//! described by both a generic (diatonic) size and a quality.

use std::fmt;
use std::str::FromStr;

use super::{Note, ParseError, Pitch, Step};

/// Interval quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntervalQuality {
    /// Diminished (can be multiply diminished)
    Diminished(u8),
    /// Minor (only for 2nd, 3rd, 6th, 7th)
    Minor,
    /// Perfect (only for unison, 4th, 5th, octave)
    Perfect,
    /// Major (only for 2nd, 3rd, 6th, 7th)
    Major,
    /// Augmented (can be multiply augmented)
    Augmented(u8),
}

impl IntervalQuality {
    /// Get the quality abbreviation
    pub fn abbreviation(&self) -> String {
        match self {
            IntervalQuality::Diminished(n) => "d".repeat(*n as usize),
            IntervalQuality::Minor => "m".to_string(),
            IntervalQuality::Perfect => "P".to_string(),
            IntervalQuality::Major => "M".to_string(),
            IntervalQuality::Augmented(n) => "A".repeat(*n as usize),
        }
    }

    /// Get the full name
    pub fn name(&self) -> String {
        match self {
            IntervalQuality::Diminished(1) => "diminished".to_string(),
            IntervalQuality::Diminished(n) => format!("{}-diminished", n),
            IntervalQuality::Minor => "minor".to_string(),
            IntervalQuality::Perfect => "perfect".to_string(),
            IntervalQuality::Major => "major".to_string(),
            IntervalQuality::Augmented(1) => "augmented".to_string(),
            IntervalQuality::Augmented(n) => format!("{}-augmented", n),
        }
    }
}

impl fmt::Display for IntervalQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// A musical interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Interval {
    /// Generic (diatonic) interval: 0 = unison, 1 = second, etc.
    generic: i32,
    /// Chromatic semitones
    semitones: i32,
}

impl Interval {
    /// Create an interval from generic size and semitones
    pub fn new(generic: i32, semitones: i32) -> Self {
        Self { generic, semitones }
    }

    /// Create an interval from quality and generic size. Returns an error
    /// for an invalid quality/generic-size combination (e.g. a "minor
    /// fourth", since fourths are a perfect-type interval and cannot be
    /// minor/major) rather than panicking, matching this crate's
    /// fallible-parsing convention used elsewhere (`ParseError`).
    pub fn from_quality(quality: IntervalQuality, generic: i32) -> Result<Self, ParseError> {
        let abs_generic = generic.abs();
        let direction = if generic >= 0 { 1 } else { -1 };

        // Normalize to within an octave for calculation
        let simple_generic = abs_generic % 7;
        let octaves = abs_generic / 7;

        // Base semitones for each generic interval
        let base_semitones = match simple_generic {
            0 => 0,  // unison
            1 => 2,  // major second
            2 => 4,  // major third
            3 => 5,  // perfect fourth
            4 => 7,  // perfect fifth
            5 => 9,  // major sixth
            6 => 11, // major seventh
            _ => unreachable!(),
        };

        // Determine if this is a "perfect" interval
        let is_perfect_type = matches!(simple_generic, 0 | 3 | 4);

        // Calculate semitone adjustment based on quality
        let adjustment = match quality {
            IntervalQuality::Diminished(n) => {
                if is_perfect_type {
                    -(n as i32)
                } else {
                    -(n as i32 + 1)
                }
            }
            IntervalQuality::Minor => {
                if is_perfect_type {
                    return Err(ParseError::InvalidInterval(
                        "perfect intervals (unison/4th/5th/octave) cannot be minor".to_string(),
                    ));
                } else {
                    -1
                }
            }
            IntervalQuality::Perfect => {
                if is_perfect_type {
                    0
                } else {
                    return Err(ParseError::InvalidInterval(
                        "2nd/3rd/6th/7th intervals cannot be perfect".to_string(),
                    ));
                }
            }
            IntervalQuality::Major => {
                if is_perfect_type {
                    return Err(ParseError::InvalidInterval(
                        "perfect intervals (unison/4th/5th/octave) cannot be major".to_string(),
                    ));
                } else {
                    0
                }
            }
            IntervalQuality::Augmented(n) => n as i32,
        };

        let semitones = (base_semitones + adjustment + octaves * 12) * direction;

        Ok(Self { generic, semitones })
    }

    /// Create a unison
    pub fn unison() -> Self {
        Self::new(0, 0)
    }

    /// Create a minor second
    pub fn minor_second() -> Self {
        Self::new(1, 1)
    }

    /// Create a major second
    pub fn major_second() -> Self {
        Self::new(1, 2)
    }

    /// Create a minor third
    pub fn minor_third() -> Self {
        Self::new(2, 3)
    }

    /// Create a major third
    pub fn major_third() -> Self {
        Self::new(2, 4)
    }

    /// Create a perfect fourth
    pub fn perfect_fourth() -> Self {
        Self::new(3, 5)
    }

    /// Create a tritone (augmented fourth / diminished fifth)
    pub fn tritone() -> Self {
        Self::new(3, 6)
    }

    /// Create a perfect fifth
    pub fn perfect_fifth() -> Self {
        Self::new(4, 7)
    }

    /// Create a minor sixth
    pub fn minor_sixth() -> Self {
        Self::new(5, 8)
    }

    /// Create a major sixth
    pub fn major_sixth() -> Self {
        Self::new(5, 9)
    }

    /// Create a minor seventh
    pub fn minor_seventh() -> Self {
        Self::new(6, 10)
    }

    /// Create a major seventh
    pub fn major_seventh() -> Self {
        Self::new(6, 11)
    }

    /// Create an octave
    pub fn octave() -> Self {
        Self::new(7, 12)
    }

    /// Get the generic (diatonic) interval size
    pub fn generic(&self) -> i32 {
        self.generic
    }

    /// Get the chromatic semitones
    pub fn semitones(&self) -> i32 {
        self.semitones
    }

    /// Get the simple interval name (1-7)
    pub fn simple_generic(&self) -> i32 {
        let abs_generic = self.generic.abs();
        if abs_generic == 0 {
            0
        } else {
            ((abs_generic - 1) % 7) + 1
        }
    }

    /// Get the simple semitones (within octave)
    pub fn simple_semitones(&self) -> i32 {
        self.semitones.rem_euclid(12)
    }

    /// Check if this is a compound interval (larger than an octave)
    pub fn is_compound(&self) -> bool {
        self.generic.abs() > 7
    }

    /// Get the number of octaves in this interval
    pub fn octaves(&self) -> i32 {
        self.generic.abs() / 7
    }

    /// Determine the quality of this interval
    pub fn quality(&self) -> IntervalQuality {
        let simple_generic = self.generic.abs() % 7;
        let simple_semitones = self.semitones.abs() % 12;

        // Expected semitones for major/perfect intervals
        let expected = match simple_generic {
            0 => 0,  // perfect unison
            1 => 2,  // major second
            2 => 4,  // major third
            3 => 5,  // perfect fourth
            4 => 7,  // perfect fifth
            5 => 9,  // major sixth
            6 => 11, // major seventh
            _ => unreachable!(),
        };

        let diff = simple_semitones - expected;
        let is_perfect_type = matches!(simple_generic, 0 | 3 | 4);

        match (is_perfect_type, diff) {
            (true, d) if d < 0 => IntervalQuality::Diminished((-d) as u8),
            (true, 0) => IntervalQuality::Perfect,
            (true, d) => IntervalQuality::Augmented(d as u8),
            (false, d) if d < -1 => IntervalQuality::Diminished((-d - 1) as u8),
            (false, -1) => IntervalQuality::Minor,
            (false, 0) => IntervalQuality::Major,
            (false, d) => IntervalQuality::Augmented(d as u8),
        }
    }

    /// Get the interval name (e.g., "M3", "P5", "m7")
    pub fn name(&self) -> String {
        let direction = if self.generic < 0 { "-" } else { "" };
        let quality = self.quality();
        let generic_num = self.generic.abs() + 1; // 1-indexed

        format!("{}{}{}", direction, quality.abbreviation(), generic_num)
    }

    /// Get the full name (e.g., "major third", "perfect fifth")
    pub fn full_name(&self) -> String {
        let direction = if self.generic < 0 {
            "descending "
        } else {
            ""
        };
        let generic_num = self.generic.abs() + 1;
        let generic_name = match generic_num {
            1 => "unison",
            2 => "second",
            3 => "third",
            4 => "fourth",
            5 => "fifth",
            6 => "sixth",
            7 => "seventh",
            8 => "octave",
            9 => "ninth",
            10 => "tenth",
            11 => "eleventh",
            12 => "twelfth",
            13 => "thirteenth",
            _ => return format!("{}interval {}", direction, generic_num),
        };

        format!("{}{} {}", direction, self.quality().name(), generic_name)
    }

    /// Get the chromatic complement (inversion within an octave): the
    /// interval that, added to this one, makes an octave. Only meaningful
    /// for simple intervals (unison through octave); compound intervals
    /// are reduced to their simple form first.
    ///
    /// Unison (0 semitones) and an exact octave (12 semitones) are the two
    /// boundary "simple" intervals and complement each other (unison <->
    /// octave), matching the standard "1st <-> 8th" pairing. A naive
    /// `% 7`/`% 12` reduction collapses both of them to the same residue
    /// (0), which used to make `complement()` of an actual octave return
    /// another octave instead of a unison.
    pub fn complement(&self) -> Interval {
        let sign = if self.generic < 0 { -1 } else { 1 };
        let abs_generic = self.generic.abs();
        let abs_semitones = self.semitones.abs();

        // Reduce compound intervals (more than one octave) down to the
        // simple range 1..=7, without collapsing an exact octave (7) down
        // to 0 the way a plain `% 7` would.
        let (simple_generic, simple_semitones) = if abs_generic > 7 {
            let octaves = (abs_generic - 1) / 7;
            (abs_generic - octaves * 7, abs_semitones - octaves * 12)
        } else {
            (abs_generic, abs_semitones)
        };

        let new_generic = (7 - simple_generic) * sign;
        let new_semitones = (12 - simple_semitones) * sign;
        Interval::new(new_generic, new_semitones)
    }

    /// Reverse the direction of the interval
    pub fn reverse(&self) -> Interval {
        Interval::new(-self.generic, -self.semitones)
    }

    /// Add two intervals
    pub fn add(&self, other: &Interval) -> Interval {
        Interval::new(self.generic + other.generic, self.semitones + other.semitones)
    }

    /// Check if this interval is consonant
    pub fn is_consonant(&self) -> bool {
        matches!(self.simple_semitones(), 0 | 3 | 4 | 5 | 7 | 8 | 9 | 12)
    }

    /// Check if this interval is a perfect consonance
    pub fn is_perfect_consonance(&self) -> bool {
        matches!(self.simple_semitones(), 0 | 5 | 7 | 12)
    }

    /// Construct the interval between two pitches (`p2` relative to `p1`).
    /// This is the direction the module doc comment describes
    /// ("distance between two pitches") but that, before this method
    /// existed, could only be produced the other way around (generic +
    /// semitones -> a transposed pitch).
    pub fn between(p1: &Pitch, p2: &Pitch) -> Interval {
        let generic = p2.diatonic_note_num() - p1.diatonic_note_num();
        let semitones = (p2.ps() - p1.ps()).round() as i32;
        Interval::new(generic, semitones)
    }

    // -- GenericInterval-style properties --

    /// Whether this is a true unison (no diatonic step difference at all;
    /// distinct from an exact octave, which has 7 steps of difference).
    pub fn is_unison(&self) -> bool {
        self.generic == 0
    }

    /// Whether this is a diatonic step (a 2nd, major or minor).
    pub fn is_diatonic_step(&self) -> bool {
        self.generic.abs() == 1
    }

    /// Whether this is a "skip"/leap (a 3rd or larger, but see
    /// `is_diatonic_step`/`is_unison` for the smaller cases).
    pub fn is_skip(&self) -> bool {
        self.generic.abs() >= 2
    }

    /// Number of complete octaves in this interval, ignoring direction.
    pub fn undirected_octaves(&self) -> i32 {
        self.octaves()
    }

    /// Distance in staff positions (lines/spaces) this interval spans —
    /// the same as the raw (0-indexed, signed) generic value.
    pub fn staff_distance(&self) -> i32 {
        self.generic
    }

    /// The generic interval reduced mod 7 (0-6): a letter-name distance,
    /// ignoring accidentals and octave count.
    pub fn mod7(&self) -> i32 {
        self.generic.rem_euclid(7)
    }

    /// The diatonic "inversion" of `mod7()` within mod-7 arithmetic.
    pub fn mod7_inversion(&self) -> i32 {
        (7 - self.mod7()) % 7
    }

    /// 1-indexed "scale degree distance" (1 = unison, 8 = an exact octave,
    /// ...), reduced to the 2..=8 range for anything larger than an octave
    /// — unlike `simple_undirected`, an exact octave stays 8 rather than
    /// collapsing to 1. Always positive (undirected). Mirrors music21's
    /// `GenericInterval.semiSimpleUndirected`.
    pub fn semi_simple_undirected(&self) -> i32 {
        let n = self.generic.abs() + 1;
        if n <= 8 {
            n
        } else {
            ((n - 2) % 7) + 2
        }
    }

    /// Directed version of `semi_simple_undirected` (negative for
    /// descending intervals).
    pub fn semi_simple_directed(&self) -> i32 {
        let n = self.semi_simple_undirected();
        if self.generic < 0 {
            -n
        } else {
            n
        }
    }

    /// 1-indexed simple generic size (1-7): like `semi_simple_undirected`,
    /// but an exact octave (or any exact multiple) collapses down to 1
    /// (unison) rather than staying 8. Always positive (undirected).
    pub fn simple_undirected(&self) -> i32 {
        let n = self.semi_simple_undirected();
        if n == 8 {
            1
        } else {
            n
        }
    }

    /// Directed version of `simple_undirected`.
    pub fn simple_directed(&self) -> i32 {
        let n = self.simple_undirected();
        if self.generic < 0 {
            -n
        } else {
            n
        }
    }

    // -- Name variants --

    /// Word form of a 1-indexed generic interval number (1=unison,
    /// 2=second, ... falls back to "interval N" beyond the 13th).
    fn generic_num_word(num: i32) -> String {
        match num {
            1 => "unison".to_string(),
            2 => "second".to_string(),
            3 => "third".to_string(),
            4 => "fourth".to_string(),
            5 => "fifth".to_string(),
            6 => "sixth".to_string(),
            7 => "seventh".to_string(),
            8 => "octave".to_string(),
            9 => "ninth".to_string(),
            10 => "tenth".to_string(),
            11 => "eleventh".to_string(),
            12 => "twelfth".to_string(),
            13 => "thirteenth".to_string(),
            _ => format!("interval {num}"),
        }
    }

    /// Alias of `full_name()` — a human-readable, direction-aware name
    /// (e.g. "descending minor third").
    pub fn nice_name(&self) -> String {
        self.full_name()
    }

    /// Alias of `name()` — the abbreviated, direction-aware name (e.g. "-m3").
    pub fn directed_name(&self) -> String {
        self.name()
    }

    /// Alias of `full_name()` — the full, direction-aware name.
    pub fn directed_nice_name(&self) -> String {
        self.full_name()
    }

    /// Abbreviated name using the simple (octave-reduced) generic size,
    /// e.g. a compound major 9th reports as "M2" rather than "M9".
    pub fn simple_name(&self) -> String {
        format!("{}{}", self.quality().abbreviation(), self.simple_undirected())
    }

    /// Full name using the simple (octave-reduced) generic size, e.g. a
    /// compound major 9th reports as "major second" rather than "major ninth".
    pub fn simple_nice_name(&self) -> String {
        format!(
            "{} {}",
            self.quality().name(),
            Self::generic_num_word(self.simple_undirected())
        )
    }

    /// Abbreviated name using the semi-simple generic size (keeps an exact
    /// octave as "8" rather than reducing it to a unison).
    pub fn semi_simple_name(&self) -> String {
        format!(
            "{}{}",
            self.quality().abbreviation(),
            self.semi_simple_undirected()
        )
    }

    /// Full name using the semi-simple generic size.
    pub fn semi_simple_nice_name(&self) -> String {
        format!(
            "{} {}",
            self.quality().name(),
            Self::generic_num_word(self.semi_simple_undirected())
        )
    }

    /// The Pythagorean (3-limit just intonation) tuning ratio for this
    /// interval's simple semitone class, derived from stacking pure
    /// fifths (3:2). E.g. a perfect fifth is exactly 3/2; a major third is
    /// 81/64 (noticeably different from the 5/4 of 5-limit just
    /// intonation).
    pub fn interval_to_pythagorean_ratio(&self) -> f64 {
        match self.simple_semitones() {
            0 => 1.0,
            1 => 256.0 / 243.0,
            2 => 9.0 / 8.0,
            3 => 32.0 / 27.0,
            4 => 81.0 / 64.0,
            5 => 4.0 / 3.0,
            6 => 729.0 / 512.0,
            7 => 3.0 / 2.0,
            8 => 128.0 / 81.0,
            9 => 27.0 / 16.0,
            10 => 16.0 / 9.0,
            11 => 243.0 / 128.0,
            _ => 1.0,
        }
    }

    /// Transpose a pitch by this interval's *generic* (diatonic) distance,
    /// using `key`'s scale to determine the correct spelling for the
    /// result, rather than computing an accidental from generic semitone
    /// math the way the key-agnostic `Pitch::transpose` does. E.g.
    /// transposing up a third in G major correctly lands on the key's own
    /// F# rather than an chromatically-derived spelling.
    ///
    /// Only the interval's generic (letter-step) distance is used — the
    /// resulting accidental always comes from whatever the key's scale has
    /// at that letter, even if this interval's own `semitones()` would
    /// imply something else (this is what "key aware" means: the key wins).
    pub fn transpose_pitch_key_aware(&self, pitch: &Pitch, key: &crate::notation::Key) -> Pitch {
        let new_diatonic_num = pitch.diatonic_note_num() + self.generic;
        let mut new_pitch = pitch.clone();
        new_pitch.set_diatonic_note_num(new_diatonic_num);

        let scale = crate::notation::Scale::new(key.tonic().clone(), key.mode());
        let accidental = scale
            .pitches()
            .into_iter()
            .find(|p| p.step() == new_pitch.step())
            .and_then(|p| p.accidental());
        new_pitch.set_accidental(accidental);
        new_pitch
    }
}

/// Construct the interval between two notes' pitches (`n2` relative to `n1`).
pub fn notes_to_interval(n1: &Note, n2: &Note) -> Interval {
    Interval::between(n1.pitch(), n2.pitch())
}

/// Sum a list of intervals.
pub fn add(intervals: &[Interval]) -> Interval {
    intervals.iter().fold(Interval::unison(), |acc, i| acc.add(i))
}

/// Cumulatively subtract each interval after the first from the first
/// (`intervals[0] - intervals[1] - intervals[2] - ...`). Returns a unison
/// for an empty list.
pub fn subtract(intervals: &[Interval]) -> Interval {
    let mut iter = intervals.iter();
    let first = match iter.next() {
        Some(i) => *i,
        None => return Interval::unison(),
    };
    iter.fold(first, |acc, i| {
        Interval::new(acc.generic() - i.generic(), acc.semitones() - i.semitones())
    })
}

/// Get whichever of two pitches is written higher (by diatonic letter
/// name + octave, i.e. staff position) — not necessarily the one that
/// actually sounds higher (see `get_absolute_higher_note` for that).
pub fn get_written_higher_note(p1: &Pitch, p2: &Pitch) -> Pitch {
    if p1.diatonic_note_num() >= p2.diatonic_note_num() {
        p1.clone()
    } else {
        p2.clone()
    }
}

/// Get whichever of two pitches is written lower (by staff position).
pub fn get_written_lower_note(p1: &Pitch, p2: &Pitch) -> Pitch {
    if p1.diatonic_note_num() <= p2.diatonic_note_num() {
        p1.clone()
    } else {
        p2.clone()
    }
}

/// Get whichever of two pitches actually sounds higher (by pitch space),
/// regardless of how each is spelled/written.
pub fn get_absolute_higher_note(p1: &Pitch, p2: &Pitch) -> Pitch {
    if p1.ps() >= p2.ps() {
        p1.clone()
    } else {
        p2.clone()
    }
}

/// Get whichever of two pitches actually sounds lower (by pitch space).
pub fn get_absolute_lower_note(p1: &Pitch, p2: &Pitch) -> Pitch {
    if p1.ps() <= p2.ps() {
        p1.clone()
    } else {
        p2.clone()
    }
}

/// Convert an absolute diatonic note number (see `Pitch::diatonic_note_num`)
/// to its letter-name `Step`.
pub fn convert_diatonic_number_to_step(num: i32) -> Step {
    Step::from_index(num - 1)
}

impl Default for Interval {
    fn default() -> Self {
        Self::unison()
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Interval {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseError::InvalidInterval(s.to_string()));
        }

        // Check for descending
        let (negative, s) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };

        // Parse quality
        let mut chars = s.chars().peekable();
        let mut quality_str = String::new();

        while let Some(&c) = chars.peek() {
            if c.is_alphabetic() {
                quality_str.push(chars.next().unwrap());
            } else {
                break;
            }
        }

        // Parse number
        let num_str: String = chars.collect();
        let num: i32 = num_str
            .parse()
            .map_err(|_| ParseError::InvalidInterval(s.to_string()))?;

        if num < 1 {
            return Err(ParseError::InvalidInterval(s.to_string()));
        }

        let generic = num - 1; // Convert to 0-indexed

        // Determine quality
        let quality = match quality_str.as_str() {
            "P" | "p" => IntervalQuality::Perfect,
            "M" => IntervalQuality::Major,
            "m" => IntervalQuality::Minor,
            "A" | "a" => IntervalQuality::Augmented(1),
            "AA" | "aa" => IntervalQuality::Augmented(2),
            "d" => IntervalQuality::Diminished(1),
            "dd" => IntervalQuality::Diminished(2),
            _ => return Err(ParseError::InvalidInterval(s.to_string())),
        };

        let mut interval = Interval::from_quality(quality, generic)?;

        if negative {
            interval = interval.reverse();
        }

        Ok(interval)
    }
}

impl From<i32> for Interval {
    /// Create an interval from semitones (generic is inferred)
    fn from(semitones: i32) -> Self {
        // Infer a reasonable generic based on semitones
        let abs_semi = semitones.abs();
        let octaves = abs_semi / 12;
        let simple_semi = abs_semi % 12;

        let simple_generic = match simple_semi {
            0 => 0,
            1 | 2 => 1,
            3 | 4 => 2,
            5 | 6 => 3,
            7 | 8 => 4,
            9 | 10 => 5,
            11 => 6,
            _ => unreachable!(),
        };

        let generic = simple_generic + octaves * 7;
        let direction = if semitones >= 0 { 1 } else { -1 };

        Interval::new(generic * direction, semitones)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_basic() {
        let m3 = Interval::minor_third();
        assert_eq!(m3.generic(), 2);
        assert_eq!(m3.semitones(), 3);
        assert_eq!(m3.quality(), IntervalQuality::Minor);

        let p5 = Interval::perfect_fifth();
        assert_eq!(p5.generic(), 4);
        assert_eq!(p5.semitones(), 7);
        assert_eq!(p5.quality(), IntervalQuality::Perfect);
    }

    #[test]
    fn test_interval_names() {
        assert_eq!(Interval::minor_third().name(), "m3");
        assert_eq!(Interval::major_third().name(), "M3");
        assert_eq!(Interval::perfect_fifth().name(), "P5");
        assert_eq!(Interval::minor_seventh().name(), "m7");
    }

    #[test]
    fn test_interval_parse() {
        let m3: Interval = "m3".parse().unwrap();
        assert_eq!(m3, Interval::minor_third());

        let p5: Interval = "P5".parse().unwrap();
        assert_eq!(p5, Interval::perfect_fifth());
    }

    #[test]
    fn test_interval_complement() {
        let m3 = Interval::minor_third();
        let complement = m3.complement();
        assert_eq!(complement.semitones(), 9); // minor third + major sixth = octave
    }

    #[test]
    fn test_interval_from_semitones() {
        let i = Interval::from(7);
        assert_eq!(i.semitones(), 7);
        assert_eq!(i.quality(), IntervalQuality::Perfect);
    }

    #[test]
    fn test_complement_octave_gives_unison() {
        // Regression test: complement() of an exact octave used to return
        // another octave (both reduced to the same `% 7`/`% 12` residue as
        // a unison) instead of the correct unison.
        let octave = Interval::octave();
        let complement = octave.complement();
        assert_eq!(complement.generic(), 0);
        assert_eq!(complement.semitones(), 0);

        // And unison still complements to an octave.
        let unison = Interval::unison();
        assert_eq!(unison.complement(), Interval::octave());
    }

    #[test]
    fn test_complement_compound_interval_reduces_to_simple() {
        // A major ninth (octave + major second) should complement as if
        // it were a simple major second.
        let ninth = Interval::new(8, 14);
        let complement = ninth.complement();
        assert_eq!(complement, Interval::minor_seventh());
    }

    #[test]
    fn test_from_quality_invalid_combination_returns_err() {
        // Regression test: from_quality used to panic!() on an invalid
        // quality/generic combination instead of returning a Result,
        // inconsistent with the rest of the crate's fallible parsing.
        let result = Interval::from_quality(IntervalQuality::Minor, 3); // "minor fourth"
        assert!(result.is_err());
    }

    #[test]
    fn test_interval_between_pitches() {
        use crate::core::{Pitch, Step};

        let c4 = Pitch::from_parts(Step::C, Some(4), None);
        let g4 = Pitch::from_parts(Step::G, Some(4), None);
        let interval = Interval::between(&c4, &g4);
        assert_eq!(interval, Interval::perfect_fifth());

        // Reverse direction should be a descending fifth.
        let reversed = Interval::between(&g4, &c4);
        assert_eq!(reversed.generic(), -4);
        assert_eq!(reversed.semitones(), -7);
    }

    #[test]
    fn test_notes_to_interval() {
        use crate::core::{Note, Pitch, Step};

        let c4 = Note::quarter(Pitch::from_parts(Step::C, Some(4), None));
        let e4 = Note::quarter(Pitch::from_parts(Step::E, Some(4), None));
        assert_eq!(notes_to_interval(&c4, &e4), Interval::major_third());
    }

    #[test]
    fn test_generic_interval_properties() {
        assert!(Interval::unison().is_unison());
        assert!(!Interval::octave().is_unison());
        assert!(Interval::major_second().is_diatonic_step());
        assert!(Interval::minor_third().is_skip());
        assert!(!Interval::major_second().is_skip());

        assert_eq!(Interval::octave().undirected_octaves(), 1);
        assert_eq!(Interval::minor_third().staff_distance(), 2);
        assert_eq!(Interval::octave().mod7(), 0);
    }

    #[test]
    fn test_semi_simple_vs_simple_undirected() {
        let octave = Interval::octave();
        assert_eq!(octave.semi_simple_undirected(), 8); // stays as octave
        assert_eq!(octave.simple_undirected(), 1); // collapses to unison

        let ninth = Interval::new(8, 14); // major 9th
        assert_eq!(ninth.semi_simple_undirected(), 2);
        assert_eq!(ninth.simple_undirected(), 2);

        let descending_third = Interval::minor_third().reverse();
        assert_eq!(descending_third.simple_directed(), -3);
    }

    #[test]
    fn test_name_variants() {
        let m9 = Interval::new(8, 13); // minor ninth
        assert_eq!(m9.simple_name(), "m2");
        assert_eq!(m9.simple_nice_name(), "minor second");
        // semi-simple only differs from simple at the unison/octave
        // boundary (keeping an exact octave as "8" rather than reducing it
        // to a unison); anything larger than an octave, like a 9th,
        // reduces the same way under both.
        assert_eq!(m9.semi_simple_name(), "m2");
        assert_eq!(m9.nice_name(), m9.full_name());
        assert_eq!(m9.directed_name(), m9.name());

        let octave = Interval::octave();
        assert_eq!(octave.semi_simple_name(), "P8");
        assert_eq!(octave.simple_name(), "P1");
    }

    #[test]
    fn test_add_and_subtract_intervals() {
        let sum = add(&[Interval::major_third(), Interval::minor_third()]);
        assert_eq!(sum, Interval::new(4, 7)); // major third + minor third = fifth

        let diff = subtract(&[Interval::perfect_fifth(), Interval::major_third()]);
        assert_eq!(diff, Interval::minor_third());
    }

    #[test]
    fn test_written_vs_absolute_higher_lower() {
        use crate::core::{Accidental, Pitch, Step};

        // Fb4 is written lower than E4 (F comes after E) but sounds the
        // same pitch (both pitch class 4) — use a genuinely
        // written-vs-absolute-diverging pair: Cb4 (written C, sounds B3)
        // vs B3.
        let cb4 = Pitch::from_parts(Step::C, Some(4), Some(Accidental::Flat));
        let b3 = Pitch::from_parts(Step::B, Some(3), None);

        // Written: C comes after B, so Cb4 is written higher even though
        // they sound identical.
        assert_eq!(get_written_higher_note(&cb4, &b3), cb4);
        // Absolute: they sound the same pitch, so `>=` picks the first arg.
        assert_eq!(get_absolute_higher_note(&cb4, &b3), cb4);
        assert_eq!(get_absolute_higher_note(&b3, &cb4), b3);
    }

    #[test]
    fn test_convert_diatonic_number_to_step() {
        use crate::core::Step;
        assert_eq!(convert_diatonic_number_to_step(1), Step::C);
        assert_eq!(convert_diatonic_number_to_step(8), Step::C);
    }

    #[test]
    fn test_transpose_pitch_key_aware() {
        use crate::core::{Accidental, Step};
        use crate::notation::Key;

        // In G major (1 sharp: F#), transposing E up a third should land
        // on the key's own F# rather than a chromatically-computed
        // spelling (which the key-agnostic Pitch::transpose would give
        // from a *generic* third alone without key context).
        let g_major = Key::major(Step::G);
        let e4 = Pitch::from_parts(Step::E, Some(4), None);
        let third_up = Interval::new(2, 4); // generic third (semitones ignored here)

        let transposed = third_up.transpose_pitch_key_aware(&e4, &g_major);
        assert_eq!(transposed.step(), Step::G);
        assert_eq!(transposed.accidental(), None); // G is unaltered in G major

        // Transposing F up nothing (a "second" from F lands on G is wrong
        // test; instead check a note that should pick up the key's F#).
        let d4 = Pitch::from_parts(Step::D, Some(4), None);
        let second_up = Interval::new(1, 2);
        let transposed2 = second_up.transpose_pitch_key_aware(&d4, &g_major);
        assert_eq!(transposed2.step(), Step::E);
        assert_eq!(transposed2.accidental(), None);

        let e4b = Pitch::from_parts(Step::E, Some(4), None);
        let transposed3 = second_up.transpose_pitch_key_aware(&e4b, &g_major);
        assert_eq!(transposed3.step(), Step::F);
        assert_eq!(transposed3.accidental(), Some(Accidental::Sharp));
    }

    #[test]
    fn test_pythagorean_ratio() {
        let fifth = Interval::perfect_fifth();
        assert!((fifth.interval_to_pythagorean_ratio() - 1.5).abs() < 1e-9);

        let unison = Interval::unison();
        assert_eq!(unison.interval_to_pythagorean_ratio(), 1.0);
    }
}
