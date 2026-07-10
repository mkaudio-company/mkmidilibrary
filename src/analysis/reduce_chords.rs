//! Reduce a (typically already-chordified — see `Score::chordify`/
//! `Score::implode`) part down to a fixed maximum number of chords per
//! measure, keeping the most metrically/harmonically salient slices.

use crate::core::Duration;
use crate::stream::{MusicElement, Part};

/// Reduces a chorded part to at most a fixed number of chords per
/// measure, ranking each slice by a combination of metric weight (see
/// `TimeSignature::beat_strength`) and a small consonance bonus, then
/// extending each surviving slice's duration to cover the span of any
/// dropped slices that followed it. Mirrors a scoped subset of music21's
/// `analysis.reduceChords.ChordReducer`.
pub struct ChordReducer;

impl ChordReducer {
    /// Reduce `part` so no measure has more than `max_chords_per_measure`
    /// slices (measures already at or under that count are untouched).
    pub fn reduce(part: &Part, max_chords_per_measure: usize) -> Part {
        let max_chords_per_measure = max_chords_per_measure.max(1);
        let mut result = part.clone();

        for i in 0..result.num_measures() {
            let time_signature = result.time_signature_at(i).copied().unwrap_or_default();
            let measure_duration = result.measure_duration(i);
            let elements = match result.measure(i) {
                Some(m) => m.elements().to_vec(),
                None => continue,
            };
            if elements.len() <= max_chords_per_measure {
                continue;
            }

            let mut scored: Vec<(usize, f64)> = elements
                .iter()
                .enumerate()
                .map(|(idx, (offset, element))| {
                    let beat_weight = time_signature.beat_strength(*offset);
                    let consonance_bonus = match element {
                        MusicElement::Chord(c) if c.is_consonant() => 0.5,
                        MusicElement::Note(_) => 0.25,
                        _ => 0.0,
                    };
                    (idx, beat_weight + consonance_bonus)
                })
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            let mut keep_indices: Vec<usize> = scored
                .into_iter()
                .take(max_chords_per_measure)
                .map(|(idx, _)| idx)
                .collect();
            keep_indices.sort_unstable();
            // Always keep the downbeat slice so the reduced measure
            // doesn't start with a silent gap.
            if keep_indices.first() != Some(&0) {
                keep_indices.insert(0, 0);
                keep_indices.dedup();
            }

            let mut new_elements = Vec::with_capacity(keep_indices.len());
            for (k, &idx) in keep_indices.iter().enumerate() {
                let (offset, element) = &elements[idx];
                let next_offset = keep_indices
                    .get(k + 1)
                    .map(|&next_idx| elements[next_idx].0)
                    .unwrap_or(measure_duration);
                let new_duration = Duration::from_quarter_length(next_offset - *offset);

                let mut element = element.clone();
                match &mut element {
                    MusicElement::Note(n) => n.set_duration(new_duration),
                    MusicElement::Chord(c) => c.set_duration(new_duration),
                    MusicElement::Rest(r) => r.set_duration(new_duration),
                }
                new_elements.push((*offset, element));
            }

            if let Some(measure) = result.measure_mut(i) {
                measure.clear();
                for (offset, element) in new_elements {
                    measure.insert(offset, element);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Chord, Fraction, Pitch, Step};
    use crate::notation::TimeSignature;
    use crate::stream::Measure;

    #[test]
    fn test_reduce_keeps_measures_already_under_the_limit() {
        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Chord(Chord::major_triad(Pitch::from_parts(
                Step::C,
                Some(4),
                None,
            ))),
        );
        part.add_measure(m1);

        let reduced = ChordReducer::reduce(&part, 4);
        assert_eq!(reduced.measure(0).unwrap().len(), 1);
    }

    #[test]
    fn test_reduce_drops_weak_beats_and_extends_survivors() {
        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        // 4 quarter-length chord slices, one per beat.
        for (i, root) in [Step::C, Step::D, Step::E, Step::F].into_iter().enumerate() {
            m1.insert(
                Fraction::new(i as i64, 1),
                MusicElement::Chord(Chord::major_triad(Pitch::from_parts(root, Some(4), None))),
            );
        }
        part.add_measure(m1);

        let reduced = ChordReducer::reduce(&part, 2);
        let elements = reduced.measure(0).unwrap().elements().to_vec();

        // Reduced to (at most) 2 slices, and the downbeat (offset 0,
        // strongest beat) must survive.
        assert!(elements.len() <= 2);
        assert_eq!(elements[0].0, Fraction::new(0, 1));

        // The surviving slices' durations must sum to the full measure
        // (4 quarter lengths) — nothing was silently dropped.
        let total: Fraction = elements.iter().map(|(_, e)| e.quarter_length()).sum();
        assert_eq!(total, Fraction::new(4, 1));
    }
}
