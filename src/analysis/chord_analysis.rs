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
}

impl RomanNumeral {
    /// Create a new roman numeral
    pub fn new(degree: u8, quality: ChordQuality) -> Self {
        Self {
            degree,
            quality,
            inversion: 0,
            secondary: None,
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
        match (self.quality, self.inversion) {
            // Triads
            (ChordQuality::Major | ChordQuality::Minor | ChordQuality::Diminished | ChordQuality::Augmented, 0) => "",
            (ChordQuality::Major | ChordQuality::Minor | ChordQuality::Diminished | ChordQuality::Augmented, 1) => "6",
            (ChordQuality::Major | ChordQuality::Minor | ChordQuality::Diminished | ChordQuality::Augmented, 2) => "6/4",
            // Seventh chords
            (_, 0) => "7",
            (_, 1) => "6/5",
            (_, 2) => "4/3",
            (_, 3) => "4/2",
            _ => "",
        }
    }
}

impl fmt::Display for RomanNumeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let numeral = match self.quality {
            ChordQuality::Minor | ChordQuality::Diminished | ChordQuality::HalfDiminished7 | ChordQuality::Diminished7 => {
                self.numeral().to_lowercase()
            }
            _ => self.numeral().to_string(),
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
            write!(f, "{}{}{}/{}", numeral, quality_symbol, figured_bass, target_numeral)
        } else {
            write!(f, "{}{}{}", numeral, quality_symbol, figured_bass)
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
            let mut rotated: Vec<u8> = (0..n)
                .map(|i| pcs[(rotation + i) % n])
                .collect();

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

        // Also check inversion
        let inverted: Vec<u8> = pcs.iter().map(|&pc| (12 - pc) % 12).collect();
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

    /// Get interval vector (count of each interval class)
    pub fn interval_vector(pcs: &[u8]) -> [u8; 6] {
        let mut vector = [0u8; 6];
        let n = pcs.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let interval = (pcs[j] as i8 - pcs[i] as i8).abs() as u8 % 12;
                let ic = if interval > 6 { 12 - interval } else { interval };
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
        assert_eq!(ChordAnalyzer::analyze_quality(&c_major), ChordQuality::Major);

        let a_minor = Chord::minor_triad(Pitch::from_parts(Step::A, Some(4), None));
        assert_eq!(ChordAnalyzer::analyze_quality(&a_minor), ChordQuality::Minor);
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
}
