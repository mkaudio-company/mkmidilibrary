//! Sliding-window local-key tracking ("floating key" / modulation
//! detection): rather than one global key for a whole piece, run
//! key-finding independently over a moving window of measures to see how
//! the local key changes over time.

use crate::stream::Part;

use super::discrete::{KeyAnalysisResult, KeyFindingAlgorithm, find_key, pitch_class_distribution};

/// One sliding-window position's local key-finding result.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowedKeyResult {
    /// Index of the first measure in this window.
    pub start_measure: usize,
    /// Number of measures this window actually covers (may be smaller
    /// than the requested window size at the end of the part).
    pub window_size: usize,
    /// The key-finding result for this window's notes.
    pub result: KeyAnalysisResult,
}

/// Slide a `window_size`-measure window across `part` (advancing one
/// measure at a time), running `algorithm`'s key-finding on each
/// window's notes — tracking how the local key changes over the course
/// of the piece. Mirrors a scoped subset of music21's
/// `analysis.floatingKey.KeyAnalyzer`.
pub fn analyze_floating_key(
    part: &Part,
    window_size: usize,
    algorithm: KeyFindingAlgorithm,
) -> Vec<WindowedKeyResult> {
    let window_size = window_size.max(1);
    let num_measures = part.num_measures();

    let mut results = Vec::new();
    let mut start = 0;
    while start < num_measures {
        let end = (start + window_size).min(num_measures);
        let notes: Vec<_> = (start..end)
            .filter_map(|i| part.measure(i))
            .flat_map(|m| m.notes().cloned())
            .collect();
        let distribution = pitch_class_distribution(&notes);
        let result = find_key(&distribution, algorithm);
        results.push(WindowedKeyResult {
            start_measure: start,
            window_size: end - start,
            result,
        });
        start += 1;
    }
    results
}

/// The measure indices where the local key (from `analyze_floating_key`)
/// differs from the immediately preceding window's — a simple
/// modulation-detection signal (adjacent windows overlap heavily, so a
/// real, sustained modulation shows up as a run of consecutive changed
/// indices rather than an isolated blip).
pub fn detect_modulations(windowed: &[WindowedKeyResult]) -> Vec<usize> {
    windowed
        .windows(2)
        .filter(|pair| pair[0].result.key != pair[1].result.key)
        .map(|pair| pair[1].start_measure)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Duration, Fraction, Note, Pitch, Step};
    use crate::notation::KeyMode;
    use crate::stream::{Measure, MusicElement, Part};

    fn measure_with_phrase(number: u32, phrase: &[(Step, Duration)]) -> Measure {
        let mut measure = Measure::new(number);
        let mut offset = Fraction::new(0, 1);
        for (step, duration) in phrase.iter().cloned() {
            let quarter_length = duration.quarter_length();
            measure.insert(
                offset,
                MusicElement::Note(Note::new(Pitch::from_parts(step, Some(4), None), duration)),
            );
            offset += quarter_length;
        }
        measure
    }

    fn c_major_measure(number: u32) -> Measure {
        measure_with_phrase(
            number,
            &[
                (Step::C, Duration::half()),
                (Step::G, Duration::quarter()),
                (Step::C, Duration::quarter()),
            ],
        )
    }

    fn g_major_measure(number: u32) -> Measure {
        // Emphasize G/D/F# (the leading tone in G major) to clearly
        // differentiate from C major.
        measure_with_phrase(
            number,
            &[
                (Step::G, Duration::half()),
                (Step::D, Duration::quarter()),
                (Step::G, Duration::quarter()),
            ],
        )
    }

    #[test]
    fn test_analyze_floating_key_produces_one_result_per_window_position() {
        let mut part = Part::new();
        for i in 1..=4 {
            part.add_measure(c_major_measure(i));
        }
        let windowed = analyze_floating_key(&part, 2, KeyFindingAlgorithm::KrumhanslSchmuckler);
        // Windows start at measure 0,1,2,3 (4 measures total).
        assert_eq!(windowed.len(), 4);
        assert_eq!(windowed[0].start_measure, 0);
        assert_eq!(windowed[0].window_size, 2);
        // The last window only has 1 measure left to cover.
        assert_eq!(windowed[3].start_measure, 3);
        assert_eq!(windowed[3].window_size, 1);
    }

    #[test]
    fn test_analyze_floating_key_detects_modulation() {
        let mut part = Part::new();
        for i in 1..=4 {
            part.add_measure(c_major_measure(i));
        }
        for i in 5..=8 {
            part.add_measure(g_major_measure(i));
        }

        let windowed = analyze_floating_key(&part, 2, KeyFindingAlgorithm::KrumhanslSchmuckler);

        // The early windows (fully within the C major section) should
        // agree on C major.
        assert_eq!(windowed[0].result.key.tonic().step(), Step::C);
        assert_eq!(windowed[0].result.key.mode(), KeyMode::Major);

        // The late windows (fully within the G major section) should
        // agree on G major.
        let last = windowed.last().unwrap();
        assert_eq!(last.result.key.tonic().step(), Step::G);
        assert_eq!(last.result.key.mode(), KeyMode::Major);

        // Somewhere in the middle, the detected local key must change.
        let modulations = detect_modulations(&windowed);
        assert!(!modulations.is_empty());
    }
}
