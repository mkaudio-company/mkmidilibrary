//! Musical intervals
//!
//! An interval represents the distance between two pitches,
//! described by both a generic (diatonic) size and a quality.

use std::fmt;
use std::str::FromStr;

use super::ParseError;

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

    /// Create an interval from quality and generic size
    pub fn from_quality(quality: IntervalQuality, generic: i32) -> Self {
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
                    panic!("Perfect intervals cannot be minor")
                } else {
                    -1
                }
            }
            IntervalQuality::Perfect => {
                if is_perfect_type {
                    0
                } else {
                    panic!("Imperfect intervals cannot be perfect")
                }
            }
            IntervalQuality::Major => {
                if is_perfect_type {
                    panic!("Perfect intervals cannot be major")
                } else {
                    0
                }
            }
            IntervalQuality::Augmented(n) => n as i32,
        };

        let semitones = (base_semitones + adjustment + octaves * 12) * direction;
        let generic = generic;

        Self { generic, semitones }
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

        let diff = simple_semitones as i32 - expected as i32;
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

    /// Get the chromatic complement (inversion within an octave)
    pub fn complement(&self) -> Interval {
        let new_generic = 7 - (self.generic % 7);
        let new_semitones = 12 - (self.semitones % 12);
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
        match self.simple_semitones() {
            0 | 3 | 4 | 5 | 7 | 8 | 9 | 12 => true,
            _ => false,
        }
    }

    /// Check if this interval is a perfect consonance
    pub fn is_perfect_consonance(&self) -> bool {
        matches!(self.simple_semitones(), 0 | 5 | 7 | 12)
    }
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
        let (negative, s) = if s.starts_with('-') {
            (true, &s[1..])
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

        let mut interval = Interval::from_quality(quality, generic);

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
}
