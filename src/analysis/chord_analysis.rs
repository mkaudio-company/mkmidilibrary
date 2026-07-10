//! Chord analysis tools

use std::fmt;

use crate::core::Chord;

/// Chord quality for analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant7,
    Major7,
    Minor7,
    HalfDiminished7,
    Diminished7,
    Augmented7,
    Sus2,
    Sus4,
    Power,
    Other,
}

impl ChordQuality {
    /// Get the chord symbol suffix
    pub fn symbol(&self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Diminished => "dim",
            ChordQuality::Augmented => "aug",
            ChordQuality::Dominant7 => "7",
            ChordQuality::Major7 => "maj7",
            ChordQuality::Minor7 => "m7",
            ChordQuality::HalfDiminished7 => "m7b5",
            ChordQuality::Diminished7 => "dim7",
            ChordQuality::Augmented7 => "aug7",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::Power => "5",
            ChordQuality::Other => "?",
        }
    }

    /// Get the full name
    pub fn name(&self) -> &'static str {
        match self {
            ChordQuality::Major => "major",
            ChordQuality::Minor => "minor",
            ChordQuality::Diminished => "diminished",
            ChordQuality::Augmented => "augmented",
            ChordQuality::Dominant7 => "dominant seventh",
            ChordQuality::Major7 => "major seventh",
            ChordQuality::Minor7 => "minor seventh",
            ChordQuality::HalfDiminished7 => "half-diminished seventh",
            ChordQuality::Diminished7 => "diminished seventh",
            ChordQuality::Augmented7 => "augmented seventh",
            ChordQuality::Sus2 => "suspended second",
            ChordQuality::Sus4 => "suspended fourth",
            ChordQuality::Power => "power chord",
            ChordQuality::Other => "other",
        }
    }
}

impl fmt::Display for ChordQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Roman numeral for harmonic analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomanNumeral {
    /// Scale degree (1-7)
    degree: u8,
    /// Quality
    quality: ChordQuality,
    /// Inversion (0 = root, 1 = first, etc.)
    inversion: u8,
    /// Secondary dominant target (e.g., V/V)
    secondary: Option<u8>,
    /// Semitone alteration of the root away from its plain diatonic scale
    /// degree (e.g. -1 for a Neapolitan/flat-II, +1 for a raised/
    /// borrowed degree). 0 for an ordinary diatonic root.
    root_alteration: i8,
}

impl RomanNumeral {
    /// Create a new roman numeral
    pub fn new(degree: u8, quality: ChordQuality) -> Self {
        Self {
            degree,
            quality,
            inversion: 0,
            secondary: None,
            root_alteration: 0,
        }
    }

    /// Set the inversion
    pub fn with_inversion(mut self, inversion: u8) -> Self {
        self.inversion = inversion;
        self
    }

    /// Set as secondary dominant
    pub fn secondary_of(mut self, target: u8) -> Self {
        self.secondary = Some(target);
        self
    }

    /// Set a semitone alteration of the root (e.g. `-1` for a Neapolitan/
    /// flat-II or other borrowed/altered-root chord).
    pub fn with_root_alteration(mut self, alteration: i8) -> Self {
        self.root_alteration = alteration;
        self
    }

    /// Get the degree
    pub fn degree(&self) -> u8 {
        self.degree
    }

    /// Get the quality
    pub fn quality(&self) -> ChordQuality {
        self.quality
    }

    /// Get the inversion
    pub fn inversion(&self) -> u8 {
        self.inversion
    }

    /// Get the secondary-dominant target degree, if any.
    pub fn secondary(&self) -> Option<u8> {
        self.secondary
    }

    /// Get the root's semitone alteration (0 for an ordinary diatonic root).
    pub fn root_alteration(&self) -> i8 {
        self.root_alteration
    }

    /// Get the numeral string
    pub fn numeral(&self) -> &'static str {
        match self.degree {
            1 => "I",
            2 => "II",
            3 => "III",
            4 => "IV",
            5 => "V",
            6 => "VI",
            7 => "VII",
            _ => "?",
        }
    }

    /// Get the figured bass notation
    pub fn figured_bass(&self) -> &'static str {
        // Regression fix: this used to bucket *anything* that wasn't
        // Major/Minor/Diminished/Augmented as "seventh-chord-like" via a
        // wildcard `(_, 0) => "7"` arm — so a root-position Sus2/Sus4/
        // Power/Other roman numeral (all genuinely triads or dyads, not
        // sevenths) incorrectly reported a figured bass of "7". Deriving
        // the seventh/non-seventh bucket from real membership in the
        // seventh-chord quality set (rather than "not one of these four
        // triad qualities") fixes that for every quality, not just the
        // ones this file happened to enumerate.
        let is_seventh_like = matches!(
            self.quality,
            ChordQuality::Dominant7
                | ChordQuality::Major7
                | ChordQuality::Minor7
                | ChordQuality::HalfDiminished7
                | ChordQuality::Diminished7
                | ChordQuality::Augmented7
        );
        match (is_seventh_like, self.inversion) {
            (false, 0) => "",
            (false, 1) => "6",
            (false, 2) => "6/4",
            (true, 0) => "7",
            (true, 1) => "6/5",
            (true, 2) => "4/3",
            (true, 3) => "4/2",
            _ => "",
        }
    }

    /// Parse a roman numeral figure such as `"V7"`, `"ii6"`, `"viio7"`,
    /// `"N6"` (Neapolitan sixth), `"bII"`, or `"V7/V"` (a secondary
    /// dominant). Case indicates the default triad quality (upper =
    /// major/dominant-family, lower = minor-family) unless overridden by
    /// an explicit quality symbol (`°`/`o` diminished, `ø` half-
    /// diminished, `+` augmented). Mirrors a scoped subset of music21's
    /// `roman.RomanNumeral` figure parser.
    pub fn from_figure(figure: &str) -> Result<RomanNumeral, crate::core::ParseError> {
        let figure = figure.trim();
        let invalid = || crate::core::ParseError::InvalidInterval(figure.to_string());

        // Only treat a '/' as introducing a secondary-dominant target
        // (e.g. "V7/V") if what follows is a roman numeral (starts with
        // a letter) — a figured-bass fraction like "6/5" or "4/3" also
        // contains a '/', but is followed by a digit.
        let (main, secondary_str) = match figure.split_once('/') {
            Some((m, s)) if s.chars().next().is_some_and(|c| c.is_alphabetic()) => (m, Some(s)),
            _ => (figure, None),
        };
        if main.is_empty() {
            return Err(invalid());
        }

        // Leading accidental(s) altering the root away from its plain
        // diatonic scale degree.
        let mut root_alteration = 0i8;
        let mut rest = main;
        while let Some(stripped) = rest.strip_prefix('b') {
            root_alteration -= 1;
            rest = stripped;
        }
        while let Some(stripped) = rest.strip_prefix('#') {
            root_alteration += 1;
            rest = stripped;
        }

        // Neapolitan: a flat-II major triad, conventionally written "N"
        // rather than "bII".
        if rest.to_uppercase().starts_with('N') {
            let figured = &rest[1..];
            let (inversion, _has_seventh) = parse_figured_bass(figured)?;
            return Ok(RomanNumeral::new(2, ChordQuality::Major)
                .with_inversion(inversion)
                .with_root_alteration(-1));
        }

        let (degree, is_lower, remainder) = parse_numeral_prefix(rest).ok_or_else(invalid)?;

        let mut remainder_chars = remainder.chars();
        let explicit_quality = match remainder_chars.clone().next() {
            Some('°') | Some('o') => {
                remainder_chars.next();
                Some(ExplicitQuality::Diminished)
            }
            Some('ø') => {
                remainder_chars.next();
                Some(ExplicitQuality::HalfDiminished)
            }
            Some('+') => {
                remainder_chars.next();
                Some(ExplicitQuality::Augmented)
            }
            _ => None,
        };
        let figured: String = remainder_chars.collect();
        let (inversion, has_seventh) = parse_figured_bass(&figured)?;

        let quality = match (explicit_quality, has_seventh) {
            (Some(ExplicitQuality::Diminished), false) => ChordQuality::Diminished,
            (Some(ExplicitQuality::Diminished), true) => ChordQuality::Diminished7,
            (Some(ExplicitQuality::HalfDiminished), _) => ChordQuality::HalfDiminished7,
            (Some(ExplicitQuality::Augmented), false) => ChordQuality::Augmented,
            (Some(ExplicitQuality::Augmented), true) => ChordQuality::Augmented7,
            (None, false) if is_lower => ChordQuality::Minor,
            (None, true) if is_lower => ChordQuality::Minor7,
            (None, false) => ChordQuality::Major,
            (None, true) => ChordQuality::Dominant7,
        };

        let mut rn = RomanNumeral::new(degree, quality)
            .with_inversion(inversion)
            .with_root_alteration(root_alteration);

        if let Some(secondary_str) = secondary_str {
            let (secondary_degree, _, _) =
                parse_numeral_prefix(secondary_str.trim()).ok_or_else(invalid)?;
            rn = rn.secondary_of(secondary_degree);
        }

        Ok(rn)
    }

    /// Realize this roman numeral's actual pitches in `key`'s tonal
    /// context, stacked from the bass according to `inversion` (e.g. a
    /// first-inversion triad returns `[third, fifth, root]`, each pitch
    /// placed in the correct octave so the list forms an ascending
    /// stack from the bass). A secondary dominant (`secondary_of`)
    /// tonicizes that scale degree of `key` (using `key`'s own mode) as
    /// a temporary local tonic before finding the root. Mirrors a scoped
    /// subset of music21's `RomanNumeral.pitches`.
    pub fn pitches(&self, key: &crate::notation::Key) -> Vec<crate::core::Pitch> {
        use crate::core::Interval;
        use crate::notation::Scale;

        let local_tonic = match self.secondary {
            Some(target_degree) => {
                let scale = Scale::new(key.tonic().clone(), key.mode());
                scale
                    .pitch_for_degree(target_degree)
                    .unwrap_or_else(|| key.tonic().clone())
            }
            None => key.tonic().clone(),
        };
        let local_scale = Scale::new(local_tonic, key.mode());

        let mut root = local_scale
            .pitch_for_degree(self.degree)
            .unwrap_or_else(|| local_scale.tonic().clone());
        // `Scale::pitch_for_degree` returns octave-less pitches; give the
        // root a concrete octave so `Pitch::transpose`'s octave-crossing
        // arithmetic below actually has something to increment (starting
        // from `None`, a transposed octave stays `None` too, silently
        // discarding any octave change).
        if root.octave().is_none() {
            root.set_octave(Some(4));
        }
        if self.root_alteration != 0 {
            root = root.transpose_semitones(self.root_alteration as i32);
        }

        let intervals: Vec<Interval> = match self.quality {
            ChordQuality::Major => vec![
                Interval::unison(),
                Interval::major_third(),
                Interval::perfect_fifth(),
            ],
            ChordQuality::Minor => vec![
                Interval::unison(),
                Interval::minor_third(),
                Interval::perfect_fifth(),
            ],
            ChordQuality::Diminished => vec![
                Interval::unison(),
                Interval::minor_third(),
                Interval::tritone(),
            ],
            ChordQuality::Augmented => vec![
                Interval::unison(),
                Interval::major_third(),
                Interval::new(4, 8),
            ],
            ChordQuality::Dominant7 => vec![
                Interval::unison(),
                Interval::major_third(),
                Interval::perfect_fifth(),
                Interval::minor_seventh(),
            ],
            ChordQuality::Major7 => vec![
                Interval::unison(),
                Interval::major_third(),
                Interval::perfect_fifth(),
                Interval::major_seventh(),
            ],
            ChordQuality::Minor7 => vec![
                Interval::unison(),
                Interval::minor_third(),
                Interval::perfect_fifth(),
                Interval::minor_seventh(),
            ],
            ChordQuality::HalfDiminished7 => vec![
                Interval::unison(),
                Interval::minor_third(),
                Interval::tritone(),
                Interval::minor_seventh(),
            ],
            ChordQuality::Diminished7 => vec![
                Interval::unison(),
                Interval::minor_third(),
                Interval::tritone(),
                Interval::new(6, 9),
            ],
            ChordQuality::Augmented7 => vec![
                Interval::unison(),
                Interval::major_third(),
                Interval::new(4, 8),
                Interval::minor_seventh(),
            ],
            ChordQuality::Sus2 => vec![
                Interval::unison(),
                Interval::major_second(),
                Interval::perfect_fifth(),
            ],
            ChordQuality::Sus4 => vec![
                Interval::unison(),
                Interval::perfect_fourth(),
                Interval::perfect_fifth(),
            ],
            ChordQuality::Power => vec![Interval::unison(), Interval::perfect_fifth()],
            ChordQuality::Other => vec![Interval::unison()],
        };

        let chord_tones: Vec<crate::core::Pitch> =
            intervals.iter().map(|iv| root.transpose(iv)).collect();

        // Rotate so the requested inversion's chord tone is first (the
        // bass), then bump everything before the rotation point up an
        // octave so the result forms an ascending stack from the bass.
        let n = chord_tones.len();
        let inversion = (self.inversion as usize).min(n.saturating_sub(1));
        let mut stacked: Vec<crate::core::Pitch> = (0..n)
            .map(|i| {
                let mut pitch = chord_tones[(inversion + i) % n].clone();
                if i + inversion >= n {
                    pitch = pitch.transpose(&Interval::octave());
                }
                pitch
            })
            .collect();
        // The rotation above only bumps tones that wrapped past the end;
        // tones before the inversion index also need to sit below the
        // bass shifted up, i.e. everything is already relative to the
        // unshifted root's octave except the wrapped ones, which is
        // exactly what's needed for an ascending stack from the bass.
        stacked.truncate(n);
        stacked
    }
}

/// Explicit quality symbol found while parsing a roman-numeral figure.
enum ExplicitQuality {
    Diminished,
    HalfDiminished,
    Augmented,
}

/// Try each roman-numeral token (longest first, to avoid e.g. "I"
/// matching before "III"/"IV") as a prefix of `s`, in both upper and
/// lower case. Returns `(degree, is_lower, remainder)`.
fn parse_numeral_prefix(s: &str) -> Option<(u8, bool, &str)> {
    const TOKENS: [(&str, u8); 7] = [
        ("VII", 7),
        ("III", 3),
        ("II", 2),
        ("IV", 4),
        ("VI", 6),
        ("I", 1),
        ("V", 5),
    ];
    for (token, degree) in TOKENS {
        if let Some(remainder) = s.strip_prefix(token) {
            return Some((degree, false, remainder));
        }
        let lower = token.to_lowercase();
        if let Some(remainder) = s.strip_prefix(lower.as_str()) {
            return Some((degree, true, remainder));
        }
    }
    None
}

/// Parse a figured-bass suffix (after any quality symbol has already
/// been stripped) into `(inversion, has_seventh)`.
fn parse_figured_bass(figured: &str) -> Result<(u8, bool), crate::core::ParseError> {
    match figured {
        "" => Ok((0, false)),
        "7" => Ok((0, true)),
        "6" => Ok((1, false)),
        "6/4" | "64" => Ok((2, false)),
        "6/5" | "65" => Ok((1, true)),
        "4/3" | "43" => Ok((2, true)),
        "4/2" | "42" => Ok((3, true)),
        other => Err(crate::core::ParseError::InvalidInterval(other.to_string())),
    }
}

/// Reverse of `RomanNumeral::pitches`: analyze a `Chord`'s root, quality,
/// and inversion against `key`'s scale to find the roman numeral that
/// describes it. The root's scale degree is matched by letter name (so
/// an altered root, e.g. a Neapolitan's flat-II, is still recognized as
/// degree 2 with a `-1` `root_alteration` rather than failing to match).
/// Returns `None` if the chord has no identifiable root (e.g. it's
/// empty). Mirrors a scoped subset of music21's
/// `roman.romanNumeralFromChord`.
pub fn roman_numeral_from_chord(chord: &Chord, key: &crate::notation::Key) -> Option<RomanNumeral> {
    let root = chord.root()?;
    let scale = crate::notation::Scale::new(key.tonic().clone(), key.mode());

    let scale_pitches = scale.pitches();
    let degree = scale_pitches.iter().position(|p| p.step() == root.step())? as u8 + 1;
    let degree_pitch_class = scale_pitches[(degree - 1) as usize].pitch_class() as i32;
    let root_alteration = {
        let mut diff = (root.pitch_class() as i32 - degree_pitch_class).rem_euclid(12);
        if diff > 6 {
            diff -= 12;
        }
        diff as i8
    };

    let quality: ChordQuality = chord.quality().into();
    let inversion = chord.inversion();

    Some(
        RomanNumeral::new(degree, quality)
            .with_inversion(inversion)
            .with_root_alteration(root_alteration),
    )
}

impl fmt::Display for RomanNumeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The Neapolitan (flat-II major triad) conventionally displays
        // as "N", not "bII".
        if self.degree == 2 && self.root_alteration == -1 && self.quality == ChordQuality::Major {
            return write!(f, "N{}", self.figured_bass());
        }

        let numeral = match self.quality {
            ChordQuality::Minor
            | ChordQuality::Diminished
            | ChordQuality::HalfDiminished7
            | ChordQuality::Diminished7 => self.numeral().to_lowercase(),
            _ => self.numeral().to_string(),
        };

        let accidental_prefix = match self.root_alteration {
            n if n < 0 => "b".repeat((-n) as usize),
            n if n > 0 => "#".repeat(n as usize),
            _ => String::new(),
        };

        let quality_symbol = match self.quality {
            ChordQuality::Diminished | ChordQuality::Diminished7 => "°",
            ChordQuality::HalfDiminished7 => "ø",
            ChordQuality::Augmented | ChordQuality::Augmented7 => "+",
            _ => "",
        };

        let figured_bass = self.figured_bass();

        if let Some(target) = self.secondary {
            let target_numeral = match target {
                1 => "I",
                2 => "II",
                3 => "III",
                4 => "IV",
                5 => "V",
                6 => "VI",
                7 => "VII",
                _ => "?",
            };
            write!(
                f,
                "{}{}{}{}/{}",
                accidental_prefix, numeral, quality_symbol, figured_bass, target_numeral
            )
        } else {
            write!(
                f,
                "{}{}{}{}",
                accidental_prefix, numeral, quality_symbol, figured_bass
            )
        }
    }
}

/// Chord analyzer
pub struct ChordAnalyzer;

impl ChordAnalyzer {
    /// Analyze a chord and determine its quality
    pub fn analyze_quality(chord: &Chord) -> ChordQuality {
        chord.quality().into()
    }

    /// Get pitch classes from chord, normalized
    pub fn get_pitch_class_set(chord: &Chord) -> Vec<u8> {
        let mut pcs: Vec<u8> = chord.pitches().iter().map(|p| p.pitch_class()).collect();
        pcs.sort();
        pcs.dedup();
        pcs
    }

    /// Get normal order of pitch class set
    pub fn normal_order(pcs: &[u8]) -> Vec<u8> {
        if pcs.is_empty() {
            return vec![];
        }

        let n = pcs.len();
        let mut best = pcs.to_vec();
        let mut best_span = 12u8;

        for rotation in 0..n {
            let mut rotated: Vec<u8> = (0..n).map(|i| pcs[(rotation + i) % n]).collect();

            // Normalize to start from 0
            let first = rotated[0];
            for pc in &mut rotated {
                *pc = (*pc + 12 - first) % 12;
            }

            let span = *rotated.last().unwrap();
            if span < best_span {
                best_span = span;
                best = rotated;
            }
        }

        best
    }

    /// Get prime form of pitch class set
    pub fn prime_form(pcs: &[u8]) -> Vec<u8> {
        let normal = Self::normal_order(pcs);

        // Also check inversion. `normal_order`'s rotation logic only finds
        // the correct most-compact cyclic rotation when its input is
        // already ascending (that's how `ordered_pitch_classes` feeds
        // `pcs` above) — inverting a sorted ascending list does not itself
        // yield an ascending list, so it must be re-sorted before being
        // handed to `normal_order`, or the rotations considered don't
        // correspond to real positions around the pitch-class circle.
        let mut inverted: Vec<u8> = pcs.iter().map(|&pc| (12 - pc) % 12).collect();
        inverted.sort_unstable();
        let inverted_normal = Self::normal_order(&inverted);

        // Return the more compact form
        if Self::is_more_compact(&normal, &inverted_normal) {
            normal
        } else {
            inverted_normal
        }
    }

    fn is_more_compact(a: &[u8], b: &[u8]) -> bool {
        for (x, y) in a.iter().zip(b.iter()) {
            if x < y {
                return true;
            }
            if x > y {
                return false;
            }
        }
        true
    }

    /// Get interval vector (count of each interval class).
    ///
    /// Deduplicates `pcs` internally before counting pairs, so — unlike a
    /// version that trusted the caller to pass an already-deduplicated
    /// set — passing a raw pitch-class list with a doubled note (e.g.
    /// `Chord::pitch_classes()` rather than `ordered_pitch_classes()`)
    /// can't silently produce an inflated vector.
    pub fn interval_vector(pcs: &[u8]) -> [u8; 6] {
        let mut deduped: Vec<u8> = pcs.to_vec();
        deduped.sort_unstable();
        deduped.dedup();

        let mut vector = [0u8; 6];
        let n = deduped.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let interval = (deduped[j] as i8 - deduped[i] as i8).unsigned_abs() % 12;
                let ic = if interval > 6 {
                    12 - interval
                } else {
                    interval
                };
                if ic > 0 {
                    vector[(ic - 1) as usize] += 1;
                }
            }
        }

        vector
    }

    /// Check if chord contains a tritone
    pub fn has_tritone(chord: &Chord) -> bool {
        let pcs = Self::get_pitch_class_set(chord);
        let iv = Self::interval_vector(&pcs);
        iv[5] > 0 // ic6 = tritone
    }
}

impl From<crate::core::ChordQuality> for ChordQuality {
    fn from(q: crate::core::ChordQuality) -> Self {
        match q {
            crate::core::ChordQuality::Major => ChordQuality::Major,
            crate::core::ChordQuality::Minor => ChordQuality::Minor,
            crate::core::ChordQuality::Diminished => ChordQuality::Diminished,
            crate::core::ChordQuality::Augmented => ChordQuality::Augmented,
            crate::core::ChordQuality::Dominant => ChordQuality::Dominant7,
            crate::core::ChordQuality::HalfDiminished => ChordQuality::HalfDiminished7,
            crate::core::ChordQuality::FullyDiminished => ChordQuality::Diminished7,
            crate::core::ChordQuality::Suspended2 => ChordQuality::Sus2,
            crate::core::ChordQuality::Suspended4 => ChordQuality::Sus4,
            crate::core::ChordQuality::Power => ChordQuality::Power,
            crate::core::ChordQuality::Other => ChordQuality::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Pitch, Step};

    #[test]
    fn test_chord_quality() {
        let c_major = Chord::major_triad(Pitch::from_parts(Step::C, Some(4), None));
        assert_eq!(
            ChordAnalyzer::analyze_quality(&c_major),
            ChordQuality::Major
        );

        let a_minor = Chord::minor_triad(Pitch::from_parts(Step::A, Some(4), None));
        assert_eq!(
            ChordAnalyzer::analyze_quality(&a_minor),
            ChordQuality::Minor
        );
    }

    #[test]
    fn test_roman_numeral() {
        let rn = RomanNumeral::new(5, ChordQuality::Dominant7);
        assert_eq!(format!("{}", rn), "V7");

        let rn = RomanNumeral::new(2, ChordQuality::Minor).with_inversion(1);
        assert_eq!(format!("{}", rn), "ii6");
    }

    #[test]
    fn test_secondary_dominant() {
        let rn = RomanNumeral::new(5, ChordQuality::Dominant7).secondary_of(5);
        assert_eq!(format!("{}", rn), "V7/V");
    }

    #[test]
    fn test_from_figure_basic_triads_and_case_implies_quality() {
        let major = RomanNumeral::from_figure("V").unwrap();
        assert_eq!(major.degree(), 5);
        assert_eq!(major.quality(), ChordQuality::Major);

        let minor = RomanNumeral::from_figure("ii").unwrap();
        assert_eq!(minor.degree(), 2);
        assert_eq!(minor.quality(), ChordQuality::Minor);
    }

    #[test]
    fn test_from_figure_sevenths_and_inversions() {
        let dominant7 = RomanNumeral::from_figure("V7").unwrap();
        assert_eq!(dominant7.quality(), ChordQuality::Dominant7);
        assert_eq!(dominant7.inversion(), 0);

        let minor7 = RomanNumeral::from_figure("ii7").unwrap();
        assert_eq!(minor7.quality(), ChordQuality::Minor7);

        let first_inversion = RomanNumeral::from_figure("I6").unwrap();
        assert_eq!(first_inversion.degree(), 1);
        assert_eq!(first_inversion.inversion(), 1);

        let six_five = RomanNumeral::from_figure("V6/5").unwrap();
        assert_eq!(six_five.quality(), ChordQuality::Dominant7);
        assert_eq!(six_five.inversion(), 1);

        let four_two = RomanNumeral::from_figure("V4/2").unwrap();
        assert_eq!(four_two.inversion(), 3);
    }

    #[test]
    fn test_from_figure_explicit_quality_symbols() {
        let diminished7 = RomanNumeral::from_figure("vii°7").unwrap();
        assert_eq!(diminished7.degree(), 7);
        assert_eq!(diminished7.quality(), ChordQuality::Diminished7);

        let half_diminished = RomanNumeral::from_figure("viiø7").unwrap();
        assert_eq!(half_diminished.quality(), ChordQuality::HalfDiminished7);

        let augmented = RomanNumeral::from_figure("III+").unwrap();
        assert_eq!(augmented.quality(), ChordQuality::Augmented);
    }

    #[test]
    fn test_from_figure_secondary_dominant() {
        let rn = RomanNumeral::from_figure("V7/V").unwrap();
        assert_eq!(rn.degree(), 5);
        assert_eq!(rn.quality(), ChordQuality::Dominant7);
        assert_eq!(rn.secondary(), Some(5));
    }

    #[test]
    fn test_from_figure_neapolitan_and_altered_root() {
        let neapolitan = RomanNumeral::from_figure("N6").unwrap();
        assert_eq!(neapolitan.degree(), 2);
        assert_eq!(neapolitan.root_alteration(), -1);
        assert_eq!(neapolitan.inversion(), 1);
        assert_eq!(format!("{}", neapolitan), "N6");

        let flat_two = RomanNumeral::from_figure("bII").unwrap();
        assert_eq!(flat_two.degree(), 2);
        assert_eq!(flat_two.root_alteration(), -1);
    }

    #[test]
    fn test_from_figure_invalid_input() {
        assert!(RomanNumeral::from_figure("").is_err());
        assert!(RomanNumeral::from_figure("Q7").is_err());
        assert!(RomanNumeral::from_figure("V9").is_err());
    }

    #[test]
    fn test_roman_numeral_pitches_root_position_triad() {
        use crate::notation::{Key, KeyMode};

        let c_major = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        let five = RomanNumeral::from_figure("V").unwrap();
        let pitches = five.pitches(&c_major);

        let names: Vec<String> = pitches.iter().map(|p| p.name()).collect();
        assert_eq!(names, vec!["G", "B", "D"]);
    }

    #[test]
    fn test_roman_numeral_pitches_inversion_puts_correct_tone_in_bass() {
        use crate::notation::{Key, KeyMode};

        let c_major = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        let first_inversion = RomanNumeral::from_figure("I6").unwrap();
        let pitches = first_inversion.pitches(&c_major);

        // First inversion of I (C-E-G) puts the third (E) in the bass.
        assert_eq!(pitches[0].name(), "E");
        assert_eq!(pitches[1].name(), "G");
        assert_eq!(pitches[2].name(), "C");
        // The stack must ascend from the bass.
        assert!(pitches[0].midi() < pitches[1].midi());
        assert!(pitches[1].midi() < pitches[2].midi());
    }

    #[test]
    fn test_roman_numeral_pitches_secondary_dominant_tonicizes_target() {
        use crate::notation::{Key, KeyMode};

        let c_major = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        // V/V in C major: the dominant of G major, i.e. D major triad.
        let five_of_five = RomanNumeral::from_figure("V/V").unwrap();
        let pitches = five_of_five.pitches(&c_major);

        let names: Vec<String> = pitches.iter().map(|p| p.name()).collect();
        assert_eq!(names, vec!["D", "F#", "A"]);
    }

    #[test]
    fn test_figured_bass_sus_and_power_chords_are_not_treated_as_sevenths() {
        // Regression: figured_bass() used to bucket anything that wasn't
        // Major/Minor/Diminished/Augmented into the seventh-chord arm
        // via a wildcard, so a root-position Sus4/Power roman numeral
        // incorrectly reported "7" instead of "".
        let sus4 = RomanNumeral::new(5, ChordQuality::Sus4);
        assert_eq!(sus4.figured_bass(), "");

        let power = RomanNumeral::new(1, ChordQuality::Power).with_inversion(1);
        assert_eq!(power.figured_bass(), "6");
    }

    #[test]
    fn test_roman_numeral_from_chord_root_position_triad() {
        use crate::notation::{Key, KeyMode};

        let key = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        let g_major = Chord::major_triad(Pitch::from_parts(Step::G, Some(4), None));

        let rn = roman_numeral_from_chord(&g_major, &key).unwrap();
        assert_eq!(rn.degree(), 5);
        assert_eq!(rn.quality(), ChordQuality::Major);
        assert_eq!(rn.inversion(), 0);
        assert_eq!(rn.root_alteration(), 0);
    }

    #[test]
    fn test_roman_numeral_from_chord_detects_inversion() {
        use crate::notation::{Key, KeyMode};

        let key = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        // First-inversion C major triad, entered bass-first as E-G-C.
        let first_inversion = Chord::from_pitches(
            vec![
                Pitch::from_parts(Step::E, Some(4), None),
                Pitch::from_parts(Step::G, Some(4), None),
                Pitch::from_parts(Step::C, Some(5), None),
            ],
            crate::core::Duration::quarter(),
        );

        let rn = roman_numeral_from_chord(&first_inversion, &key).unwrap();
        assert_eq!(rn.degree(), 1);
        assert_eq!(rn.inversion(), 1);
    }

    #[test]
    fn test_roman_numeral_from_chord_altered_root_neapolitan() {
        use crate::notation::{Key, KeyMode};

        let key = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        // Db major triad in C major: a Neapolitan (flat-II).
        let neapolitan = Chord::major_triad(Pitch::from_parts(
            Step::D,
            Some(4),
            Some(crate::core::Accidental::Flat),
        ));

        let rn = roman_numeral_from_chord(&neapolitan, &key).unwrap();
        assert_eq!(rn.degree(), 2);
        assert_eq!(rn.root_alteration(), -1);
        assert_eq!(rn.quality(), ChordQuality::Major);
    }

    #[test]
    fn test_roman_numeral_pitches_seventh_chord_has_four_tones() {
        use crate::notation::{Key, KeyMode};

        let c_major = Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major);
        let dominant7 = RomanNumeral::from_figure("V7").unwrap();
        let pitches = dominant7.pitches(&c_major);

        let names: Vec<String> = pitches.iter().map(|p| p.name()).collect();
        assert_eq!(names, vec!["G", "B", "D", "F"]);
    }

    #[test]
    fn test_normal_order() {
        let pcs = vec![0, 4, 7]; // C major
        let normal = ChordAnalyzer::normal_order(&pcs);
        assert_eq!(normal, vec![0, 4, 7]);
    }

    #[test]
    fn test_interval_vector() {
        let pcs = vec![0, 4, 7]; // Major triad
        let iv = ChordAnalyzer::interval_vector(&pcs);
        assert_eq!(iv, [0, 0, 1, 1, 1, 0]); // m3, M3, P5
    }

    #[test]
    fn test_prime_form_major_triad_matches_forte_canonical_form() {
        // Regression: prime_form used to feed the *unsorted* inverted pitch
        // classes into normal_order, whose rotation logic only finds the
        // correct most-compact rotation for an already-ascending input.
        // That bug made a major triad's prime form come out as [0,4,7]
        // (the un-inverted normal order) instead of the canonical [0,3,7]
        // that Forte set-class 3-11 is keyed on.
        let major_triad_pcs = vec![0, 4, 7];
        assert_eq!(ChordAnalyzer::prime_form(&major_triad_pcs), vec![0, 3, 7]);
    }

    #[test]
    fn test_interval_vector_foolproof_against_duplicates() {
        // Regression test: interval_vector used to trust the caller to
        // pass an already-deduplicated pitch-class set; a doubled note
        // (e.g. raw pitch_classes() rather than ordered_pitch_classes())
        // used to silently inflate the vector by double-counting pairs.
        let pcs_with_duplicate = vec![0, 4, 4, 7]; // C major with doubled E
        let iv = ChordAnalyzer::interval_vector(&pcs_with_duplicate);
        assert_eq!(iv, [0, 0, 1, 1, 1, 0]); // same as the deduplicated C major triad
    }
}
