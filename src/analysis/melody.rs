//! Melodic-line analysis: pitch range (ambitus) and interval diversity.

use crate::core::Interval;
use crate::stream::Part;

/// The pitch range ("ambitus") of a part's melodic line: the interval
/// from its lowest to highest note. Mirrors music21's `analysis.ambitus`
/// (specifically its `range.Ambitus` interval result).
pub fn ambitus(part: &Part) -> Option<Interval> {
    let notes = part.find_consecutive_notes();
    let lowest = notes.iter().map(|n| n.pitch()).min()?;
    let highest = notes.iter().map(|n| n.pitch()).max()?;
    Some(Interval::between(lowest, highest))
}

/// How varied a part's melodic intervals are: the number of distinct
/// interval "simple names" (e.g. `"m3"`, `"P5"`, ignoring octave and
/// direction — see `Interval::simple_name`) divided by the total number
/// of melodic intervals, in `0.0..=1.0` (higher = more varied; `0.0` for
/// fewer than 2 notes). Mirrors music21's
/// `analysis.melody.MelodicIntervalDiversity`.
pub fn melodic_interval_diversity(part: &Part) -> f64 {
    let intervals = part.melodic_intervals();
    if intervals.is_empty() {
        return 0.0;
    }
    let mut distinct: Vec<String> = intervals.iter().map(|iv| iv.simple_name()).collect();
    distinct.sort();
    distinct.dedup();
    distinct.len() as f64 / intervals.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Duration, Fraction, Note, Pitch, Step};
    use crate::stream::{Measure, MusicElement};

    fn part_from_steps(steps: &[(Step, i8)]) -> Part {
        let mut part = Part::new();
        let mut measure = Measure::new(1);
        let mut offset = Fraction::new(0, 1);
        for &(step, octave) in steps {
            measure.insert(
                offset,
                MusicElement::Note(Note::quarter(Pitch::from_parts(step, Some(octave), None))),
            );
            offset += Duration::quarter().quarter_length();
        }
        part.add_measure(measure);
        part
    }

    #[test]
    fn test_ambitus_spans_lowest_to_highest_note() {
        let part = part_from_steps(&[(Step::C, 4), (Step::G, 5), (Step::E, 4)]);
        let range = ambitus(&part).unwrap();
        // C4 to G5 is an octave plus a fifth (12 generic steps: a 12th).
        assert_eq!(range.semitones(), 19);
    }

    #[test]
    fn test_ambitus_empty_part_is_none() {
        let part = Part::new();
        assert_eq!(ambitus(&part), None);
    }

    #[test]
    fn test_melodic_interval_diversity_all_same_interval() {
        // C4-D4-E4-F4: three consecutive major/minor second intervals,
        // but all the same *simple name* diversity-wise only if they're
        // literally identical; C-D (M2), D-E (M2), E-F (m2) -> 2 distinct
        // out of 3 total.
        let part = part_from_steps(&[(Step::C, 4), (Step::D, 4), (Step::E, 4), (Step::F, 4)]);
        let diversity = melodic_interval_diversity(&part);
        assert!((diversity - (2.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn test_melodic_interval_diversity_no_intervals_is_zero() {
        let part = part_from_steps(&[(Step::C, 4)]);
        assert_eq!(melodic_interval_diversity(&part), 0.0);
    }
}
