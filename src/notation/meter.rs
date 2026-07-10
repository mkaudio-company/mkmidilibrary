//! Time signature / meter representation

use std::fmt;
use std::str::FromStr;

use crate::core::{Fraction, ParseError};

/// Simple/compound/complex meter classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeterClassification {
    /// Beats divide into 2 (e.g. 2/4, 3/4, 4/4).
    Simple,
    /// Beats divide into 3, i.e. the numerator is a multiple of 3 greater
    /// than 3 (e.g. 6/8, 9/8, 12/8).
    Compound,
    /// Neither simple nor compound — an additive/irregular meter whose
    /// beats are an explicit or inferred mix of 2s and 3s (e.g. 5/8, 7/8).
    Complex,
}

/// A time signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeSignature {
    /// Numerator (number of beats)
    numerator: u8,
    /// Denominator (beat unit as power of 2)
    denominator: u8,
    /// Explicit additive groupings (e.g. `[3, 2]` for a summed "3+2/8"
    /// meter), if this signature was built with one. Stored inline
    /// (rather than as a `Vec`) so `TimeSignature` can stay `Copy`; a
    /// capacity of 8 comfortably covers any real summed meter.
    groups: [u8; 8],
    /// How many of `groups`'s slots are populated (0 = no explicit
    /// groups; fall back to the inferred default in `beat_group_sizes`).
    groups_len: u8,
}

impl TimeSignature {
    /// Create a new time signature
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator,
            groups: [0; 8],
            groups_len: 0,
        }
    }

    /// Create a time signature with explicit additive groupings (e.g.
    /// `TimeSignature::with_groups(5, 8, &[3, 2])` for a "3+2/8" meter).
    /// `groups` must sum to `numerator`; only the first 8 entries are
    /// kept if more are given.
    pub fn with_groups(numerator: u8, denominator: u8, groups: &[u8]) -> Self {
        let mut g = [0u8; 8];
        let len = groups.len().min(8);
        g[..len].copy_from_slice(&groups[..len]);
        Self {
            numerator,
            denominator,
            groups: g,
            groups_len: len as u8,
        }
    }

    /// Parse a time signature from a ratio string: a plain `"N/D"` (e.g.
    /// `"5/8"`) or a summed/additive `"N+N+.../D"` (e.g. `"3+2/8"`, which
    /// stores explicit groupings `[3, 2]` and a numerator of 5).
    pub fn from_ratio_string(s: &str) -> Result<Self, ParseError> {
        let trimmed = s.trim();
        let (num_part, den_part) = trimmed
            .split_once('/')
            .ok_or_else(|| ParseError::InvalidTimeSignature(trimmed.to_string()))?;

        let denominator: u8 = den_part
            .trim()
            .parse()
            .map_err(|_| ParseError::InvalidTimeSignature(trimmed.to_string()))?;

        let parts: Vec<u8> = num_part
            .split('+')
            .map(|p| p.trim().parse::<u8>())
            .collect::<Result<_, _>>()
            .map_err(|_| ParseError::InvalidTimeSignature(trimmed.to_string()))?;

        if parts.is_empty() {
            return Err(ParseError::InvalidTimeSignature(trimmed.to_string()));
        }

        let numerator: u8 = parts.iter().copied().sum();
        if parts.len() == 1 {
            Ok(Self::new(numerator, denominator))
        } else {
            Ok(Self::with_groups(numerator, denominator, &parts))
        }
    }

    /// Serialize back to a ratio string: `"N/D"`, or `"a+b+.../D"` if this
    /// signature has explicit additive groupings.
    pub fn ratio_string(&self) -> String {
        match self.explicit_groups() {
            Some(groups) => {
                let parts: Vec<String> = groups.iter().map(|g| g.to_string()).collect();
                format!("{}/{}", parts.join("+"), self.denominator)
            }
            None => format!("{}/{}", self.numerator, self.denominator),
        }
    }

    /// This signature's explicit additive groupings, if it was built with
    /// `with_groups`/parsed from a summed ratio string.
    pub fn explicit_groups(&self) -> Option<&[u8]> {
        if self.groups_len == 0 {
            None
        } else {
            Some(&self.groups[..self.groups_len as usize])
        }
    }

    /// Create common time (4/4)
    pub fn common_time() -> Self {
        Self::new(4, 4)
    }

    /// Create cut time (2/2)
    pub fn cut_time() -> Self {
        Self::new(2, 2)
    }

    /// Create 3/4 time
    pub fn three_four() -> Self {
        Self::new(3, 4)
    }

    /// Create 6/8 time
    pub fn six_eight() -> Self {
        Self::new(6, 8)
    }

    /// Get the numerator
    pub fn numerator(&self) -> u8 {
        self.numerator
    }

    /// Get the denominator
    pub fn denominator(&self) -> u8 {
        self.denominator
    }

    /// Get the bar duration in quarter lengths
    pub fn bar_duration(&self) -> Fraction {
        // Duration = numerator * (4 / denominator)
        Fraction::new(self.numerator as i64 * 4, self.denominator as i64)
    }

    /// Get the beat duration in quarter lengths (one denominator-unit,
    /// e.g. one eighth note for an x/8 meter). For compound/additive
    /// meters, the actual notated *beat* spans several of these units —
    /// see `beat_sequence` for the real per-beat durations.
    pub fn beat_duration(&self) -> Fraction {
        Fraction::new(4, self.denominator as i64)
    }

    /// Get the number of beats per bar
    pub fn beats_per_bar(&self) -> u8 {
        self.beat_group_sizes().len() as u8
    }

    /// Check if this is compound meter (divisible by 3, and more than a
    /// single triplet-beat)
    pub fn is_compound(&self) -> bool {
        self.explicit_groups().is_none() && self.numerator > 3 && self.numerator % 3 == 0
    }

    /// Check if this is simple meter (2, 3, or 4 beats to the bar)
    pub fn is_simple(&self) -> bool {
        self.explicit_groups().is_none() && self.numerator <= 4
    }

    /// Check if this is an additive/irregular meter: neither a simple
    /// (<=4) nor compound (multiple-of-3, >3) numerator, or one built
    /// with explicit additive groupings regardless of its numerator.
    pub fn is_additive(&self) -> bool {
        self.explicit_groups().is_some() || !(self.is_simple() || self.is_compound())
    }

    /// Unified simple/compound/complex classification.
    pub fn classification(&self) -> MeterClassification {
        if self.is_compound() {
            MeterClassification::Compound
        } else if self.is_additive() {
            MeterClassification::Complex
        } else {
            MeterClassification::Simple
        }
    }

    /// Check if this is duple meter (2 beats)
    pub fn is_duple(&self) -> bool {
        self.beats_per_bar() == 2
    }

    /// Check if this is triple meter (3 beats)
    pub fn is_triple(&self) -> bool {
        self.beats_per_bar() == 3
    }

    /// Check if this is quadruple meter (4 beats)
    pub fn is_quadruple(&self) -> bool {
        self.beats_per_bar() == 4
    }

    /// The size (in denominator-units) of each beat in the bar, in order:
    /// explicit groupings if this signature has them; each compound beat
    /// as `3` for compound meters; the first canonical additive grouping
    /// for additive meters; otherwise one denominator-unit per beat.
    fn beat_group_sizes(&self) -> Vec<u8> {
        if let Some(groups) = self.explicit_groups() {
            return groups.to_vec();
        }
        if self.numerator > 3 && self.numerator % 3 == 0 {
            return vec![3; (self.numerator / 3) as usize];
        }
        if !(self.numerator <= 4) {
            return self
                .additive_groupings()
                .into_iter()
                .next()
                .unwrap_or_else(|| vec![1; self.numerator as usize]);
        }
        vec![1; self.numerator as usize]
    }

    /// Get each beat's duration in quarter lengths, in order (the real
    /// per-beat durations — e.g. `[3/4, 3/4]` for 6/8, a dotted-quarter
    /// beat each, not the raw denominator-unit length).
    pub fn beat_sequence(&self) -> Vec<Fraction> {
        let unit = self.beat_duration();
        self.beat_group_sizes()
            .into_iter()
            .map(|size| unit * Fraction::from(size as i64))
            .collect()
    }

    /// Get each beat's starting offset within the bar, in quarter
    /// lengths, in the same order as `beat_sequence`.
    pub fn beat_offsets(&self) -> Vec<Fraction> {
        let mut acc = Fraction::new(0, 1);
        let mut offsets = Vec::new();
        for duration in self.beat_sequence() {
            offsets.push(acc);
            acc += duration;
        }
        offsets
    }

    /// Get the beat groupings for beaming: each beat's starting offset
    /// within the bar (in quarter lengths). Regression fix: this used to
    /// multiply the *denominator-unit* length by the beat index, which is
    /// only the same thing as the real beat length for simple meters —
    /// for a compound meter like 6/8 it produced `[0, 1/2]` (an eighth
    /// note apart) instead of the correct `[0, 3/4]` (a dotted-quarter
    /// apart, since each compound beat spans 3 eighth-note units).
    pub fn beat_groups(&self) -> Vec<Fraction> {
        self.beat_offsets()
    }

    /// The beat number (1-indexed, fractional for positions between beat
    /// starts) that `offset` (in quarter lengths, relative to the start
    /// of *a* bar — reduced modulo the bar duration first) falls on.
    pub fn get_beat(&self, offset: Fraction) -> f64 {
        let normalized = self.normalize_offset(offset);
        let starts = self.beat_offsets();
        let durations = self.beat_sequence();

        for (i, (&start, &duration)) in starts.iter().zip(durations.iter()).enumerate() {
            let end = start + duration;
            if normalized >= start && normalized < end {
                let into_beat = fraction_to_f64(normalized - start) / fraction_to_f64(duration);
                return (i + 1) as f64 + into_beat;
            }
        }
        (starts.len() + 1) as f64
    }

    /// The offset (in quarter lengths, relative to the start of the bar)
    /// of a given 1-indexed (possibly fractional) beat number — the
    /// inverse of `get_beat`. The fractional part is resolved to the
    /// nearest millionth to keep the result an exact `Fraction`.
    pub fn get_offset_from_beat(&self, beat: f64) -> Fraction {
        let starts = self.beat_offsets();
        let durations = self.beat_sequence();
        if starts.is_empty() {
            return Fraction::new(0, 1);
        }

        let beat_index = ((beat.floor() as i64) - 1).clamp(0, starts.len() as i64 - 1) as usize;
        let into_beat = (beat - beat.floor()).clamp(0.0, 1.0);
        let start = starts[beat_index];
        let duration = durations[beat_index];

        let scaled_numer = (into_beat * 1_000_000.0).round() as i64;
        start + duration * Fraction::new(scaled_numer, 1_000_000)
    }

    /// A simplified metric-accent weight for `offset` (in quarter
    /// lengths, reduced modulo the bar duration): `1.0` for the downbeat,
    /// `0.5` for the start of any other beat, `0.25` for the midpoint of
    /// a compound beat's three-part subdivision, and `0.125` for any
    /// other subdivision. This mirrors music21's `beatStrength` only at
    /// the level of a simplified hierarchy (not a full perceptual/
    /// metrical accent model).
    pub fn beat_strength(&self, offset: Fraction) -> f64 {
        let normalized = self.normalize_offset(offset);
        let starts = self.beat_offsets();
        let durations = self.beat_sequence();
        let unit = self.beat_duration();

        for (i, (&start, &duration)) in starts.iter().zip(durations.iter()).enumerate() {
            let end = start + duration;
            if normalized >= start && normalized < end {
                if normalized == start {
                    return if i == 0 { 1.0 } else { 0.5 };
                }
                if duration == unit * Fraction::from(3) && normalized - start == unit {
                    return 0.25;
                }
                return 0.125;
            }
        }
        1.0
    }

    /// Reduce `offset` into `[0, bar_duration)` (treated as a cyclic
    /// position within a single bar).
    fn normalize_offset(&self, offset: Fraction) -> Fraction {
        let bar = self.bar_duration();
        if bar <= Fraction::new(0, 1) {
            return offset;
        }
        let quotient = (offset / bar).floor();
        offset - bar * quotient
    }

    /// Get common additive groupings (e.g., 5/8 = `[3,2]` or `[2,3]`).
    /// Generalized to any additive numerator (not just 5/7/11), fixing
    /// the previous inconsistency where `is_additive()` reported `true`
    /// for a numerator of 13 while this returned an empty list for it.
    pub fn additive_groupings(&self) -> Vec<Vec<u8>> {
        if !self.is_additive() {
            return vec![];
        }
        let n = self.numerator;
        let mut groupings: Vec<Vec<u8>> = Vec::new();

        // One 3 plus the remaining as 2s (and its reverse), if the
        // remainder after removing a single 3 is even.
        if n >= 3 && (n - 3) % 2 == 0 {
            let mut g = vec![3];
            g.extend(std::iter::repeat(2).take(((n - 3) / 2) as usize));
            let mut rev = g.clone();
            rev.reverse();
            groupings.push(g.clone());
            if rev != g {
                groupings.push(rev);
            }
        }

        // Two 3s plus the remaining as 2s (and its reverse), for larger
        // additive meters where that's also a common grouping (e.g. 11 =
        // 4+4+3 style groupings live here as [3,3,...2s]-shaped options).
        if n >= 6 && (n - 6) % 2 == 0 {
            let mut g = vec![3, 3];
            g.extend(std::iter::repeat(2).take(((n - 6) / 2) as usize));
            let mut rev = g.clone();
            rev.reverse();
            if !groupings.contains(&g) {
                groupings.push(g.clone());
            }
            if rev != g && !groupings.contains(&rev) {
                groupings.push(rev);
            }
        }

        // All-2s, if the numerator is even.
        if n % 2 == 0 {
            let g = vec![2; (n / 2) as usize];
            if !groupings.contains(&g) {
                groupings.push(g);
            }
        }

        groupings
    }
}

/// Convert a `Fraction` to `f64` (exact for the small integer ratios used
/// throughout this module).
fn fraction_to_f64(f: Fraction) -> f64 {
    *f.numer() as f64 / *f.denom() as f64
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self::common_time()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ratio_string())
    }
}

impl FromStr for TimeSignature {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ratio_string(s)
    }
}

/// A marker for "senza misura" (no time signature / free meter) sections
/// of a score. Unlike `TimeSignature`, it carries no numeric meaning —
/// beats/bar durations aren't defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SenzaMisuraTimeSignature;

impl fmt::Display for SenzaMisuraTimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "senza misura")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_signature_creation() {
        let ts = TimeSignature::new(4, 4);
        assert_eq!(ts.numerator(), 4);
        assert_eq!(ts.denominator(), 4);
    }

    #[test]
    fn test_bar_duration() {
        assert_eq!(
            TimeSignature::common_time().bar_duration(),
            Fraction::new(4, 1)
        );
        assert_eq!(
            TimeSignature::three_four().bar_duration(),
            Fraction::new(3, 1)
        );
        assert_eq!(
            TimeSignature::six_eight().bar_duration(),
            Fraction::new(3, 1)
        );
    }

    #[test]
    fn test_compound_meter() {
        assert!(TimeSignature::six_eight().is_compound());
        assert!(!TimeSignature::three_four().is_compound());
        assert!(!TimeSignature::common_time().is_compound());
    }

    #[test]
    fn test_beats_per_bar() {
        assert_eq!(TimeSignature::common_time().beats_per_bar(), 4);
        assert_eq!(TimeSignature::three_four().beats_per_bar(), 3);
        assert_eq!(TimeSignature::six_eight().beats_per_bar(), 2); // Compound duple
    }

    #[test]
    fn test_meter_type() {
        assert!(TimeSignature::cut_time().is_duple());
        assert!(TimeSignature::three_four().is_triple());
        assert!(TimeSignature::common_time().is_quadruple());
    }

    #[test]
    fn test_is_additive_and_groupings_consistent_for_13() {
        // Regression: is_additive() used to report true for a numerator
        // of 13 while additive_groupings() returned an empty Vec for it
        // (the hardcoded table only covered 5, 7, and 11).
        let thirteen_eight = TimeSignature::new(13, 8);
        assert!(thirteen_eight.is_additive());
        assert!(!thirteen_eight.additive_groupings().is_empty());
        for grouping in thirteen_eight.additive_groupings() {
            assert_eq!(grouping.iter().sum::<u8>(), 13);
        }
    }

    #[test]
    fn test_additive_groupings_five_and_seven() {
        let five_eight = TimeSignature::new(5, 8);
        assert!(five_eight.additive_groupings().contains(&vec![3, 2]));
        assert!(five_eight.additive_groupings().contains(&vec![2, 3]));

        let seven_eight = TimeSignature::new(7, 8);
        assert!(seven_eight.additive_groupings().contains(&vec![3, 2, 2]));
        assert!(seven_eight.additive_groupings().contains(&vec![2, 2, 3]));
    }

    #[test]
    fn test_compound_beat_groups_use_real_beat_length_not_denominator_unit() {
        // Regression: beat_groups() used to compute offsets as
        // `beat_duration() * index`, where beat_duration() is the
        // *denominator-unit* length (an eighth note = 1/2 quarter length
        // for 6/8) rather than the real compound-beat length (a
        // dotted quarter = 3/2 quarter length) — producing `[0, 1/2]`
        // instead of the correct `[0, 3/2]`.
        let six_eight = TimeSignature::six_eight();
        assert_eq!(
            six_eight.beat_groups(),
            vec![Fraction::new(0, 1), Fraction::new(3, 2)]
        );
        assert_eq!(
            six_eight.beat_sequence(),
            vec![Fraction::new(3, 2), Fraction::new(3, 2)]
        );
    }

    #[test]
    fn test_ratio_string_round_trip_plain() {
        let ts = TimeSignature::new(5, 4);
        assert_eq!(ts.ratio_string(), "5/4");
        assert_eq!(TimeSignature::from_ratio_string("5/4").unwrap(), ts);
        assert_eq!("5/4".parse::<TimeSignature>().unwrap(), ts);
    }

    #[test]
    fn test_ratio_string_round_trip_summed() {
        let ts = TimeSignature::from_ratio_string("3+2/8").unwrap();
        assert_eq!(ts.numerator(), 5);
        assert_eq!(ts.denominator(), 8);
        assert_eq!(ts.explicit_groups(), Some(&[3, 2][..]));
        assert_eq!(ts.ratio_string(), "3+2/8");
    }

    #[test]
    fn test_from_ratio_string_invalid() {
        assert!(TimeSignature::from_ratio_string("bogus").is_err());
        assert!(TimeSignature::from_ratio_string("3+x/8").is_err());
    }

    #[test]
    fn test_explicit_groups_override_default_beat_sequence() {
        // Without explicit groups, 5/8's default grouping is used;
        // with an explicit "2+3/8", the groupings are respected exactly.
        let default_five = TimeSignature::new(5, 8);
        let explicit_five = TimeSignature::with_groups(5, 8, &[2, 3]);
        assert_ne!(default_five.beat_sequence(), explicit_five.beat_sequence());
        assert_eq!(
            explicit_five.beat_sequence(),
            vec![Fraction::new(1, 1), Fraction::new(3, 2)]
        );
    }

    #[test]
    fn test_classification() {
        assert_eq!(
            TimeSignature::common_time().classification(),
            MeterClassification::Simple
        );
        assert_eq!(
            TimeSignature::six_eight().classification(),
            MeterClassification::Compound
        );
        assert_eq!(
            TimeSignature::new(7, 8).classification(),
            MeterClassification::Complex
        );
    }

    #[test]
    fn test_get_beat_and_get_offset_from_beat_round_trip() {
        let four_four = TimeSignature::common_time();
        assert_eq!(four_four.get_beat(Fraction::new(0, 1)), 1.0);
        assert_eq!(four_four.get_beat(Fraction::new(1, 1)), 2.0);
        assert_eq!(four_four.get_beat(Fraction::new(3, 2)), 2.5);

        assert_eq!(
            four_four.get_offset_from_beat(1.0),
            Fraction::new(0, 1)
        );
        assert_eq!(
            four_four.get_offset_from_beat(3.0),
            Fraction::new(2, 1)
        );
    }

    #[test]
    fn test_get_beat_compound_meter() {
        // In 6/8, beat 2 starts at offset 3/2 (a dotted quarter in, i.e.
        // 3 eighth-note units of 1/2 quarter length each).
        let six_eight = TimeSignature::six_eight();
        assert_eq!(six_eight.get_beat(Fraction::new(0, 1)), 1.0);
        assert_eq!(six_eight.get_beat(Fraction::new(3, 2)), 2.0);
    }

    #[test]
    fn test_beat_strength_downbeat_and_secondary_beats() {
        let four_four = TimeSignature::common_time();
        assert_eq!(four_four.beat_strength(Fraction::new(0, 1)), 1.0);
        assert_eq!(four_four.beat_strength(Fraction::new(1, 1)), 0.5);
        assert_eq!(four_four.beat_strength(Fraction::new(1, 2)), 0.125);
    }

    #[test]
    fn test_beat_strength_compound_meter_subdivision() {
        let six_eight = TimeSignature::six_eight();
        assert_eq!(six_eight.beat_strength(Fraction::new(0, 1)), 1.0);
        // Second eighth-note unit within the first compound beat (which
        // spans offsets 0 to 3/2): weaker than the downbeat but stronger
        // than an arbitrary subdivision.
        assert_eq!(six_eight.beat_strength(Fraction::new(1, 2)), 0.25);
        // Third eighth-note unit within the first beat: an arbitrary
        // (non-midpoint) subdivision.
        assert_eq!(six_eight.beat_strength(Fraction::new(1, 1)), 0.125);
        // Start of the second beat (a dotted quarter in, offset 3/2).
        assert_eq!(six_eight.beat_strength(Fraction::new(3, 2)), 0.5);
    }

    #[test]
    fn test_senza_misura_display() {
        assert_eq!(SenzaMisuraTimeSignature.to_string(), "senza misura");
    }
}
