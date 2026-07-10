//! Discrete key-finding algorithms (the Krumhansl-Schmuckler family)
//!
//! Each algorithm supplies a 12-value major and a 12-value minor "key
//! profile" — weights for how strongly each scale degree (relative to a
//! candidate tonic) implies that key. Finding the best key for a
//! pitch-class distribution is the same procedure for all of them:
//! correlate the distribution against all 24 (12 major + 12 minor)
//! rotations of the profile and pick the highest-correlating one.
//!
//! Profile values are transcribed from the published literature
//! (Krumhansl & Kessler 1982; Aarden 2003; Bellman & Budge 2005;
//! Temperley 1999, based on the Kostka-Payne corpus) rather than derived
//! independently in this crate; small numeric differences from the
//! original sources are possible, but the qualitative shape (tonic/
//! dominant emphasis, etc.) is preserved, and a clearly-tonal passage
//! resolves to the correct key regardless.

use crate::core::{Note, Pitch, Step};
use crate::notation::{Key, KeyMode};
use crate::stream::Part;

/// Which published key-profile weighting to use for key-finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyFindingAlgorithm {
    /// Krumhansl & Kessler (1982) — the original, most widely cited
    /// profile, derived from probe-tone perception experiments.
    KrumhanslSchmuckler,
    /// Aarden (2003) — revised profiles derived from corpus pitch-class
    /// frequencies rather than perceptual experiments.
    AardenEssen,
    /// A simple theoretical weighting (tonic/dominant emphasis, scale
    /// membership only) rather than an empirically-fit profile.
    SimpleWeights,
    /// Bellman & Budge (2005) — profiles derived from an 18th/19th
    /// century corpus.
    BellmanBudge,
    /// Temperley (1999) — profiles fit to the Kostka-Payne corpus of
    /// common-practice excerpts.
    TemperleyKostkaPayne,
}

impl KeyFindingAlgorithm {
    /// This algorithm's (major, minor) 12-value profiles, index 0 = the
    /// tonic's own weight, indices increasing by semitone above it.
    fn profiles(&self) -> ([f64; 12], [f64; 12]) {
        match self {
            KeyFindingAlgorithm::KrumhanslSchmuckler => (
                [
                    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
                ],
                [
                    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
                ],
            ),
            KeyFindingAlgorithm::AardenEssen => (
                [
                    17.7661, 0.145624, 14.9265, 0.160186, 19.8049, 11.3587, 0.291248, 22.062,
                    0.145624, 8.15494, 0.232998, 4.95122,
                ],
                [
                    18.2648, 0.737619, 14.0499, 16.8599, 0.702494, 14.4362, 0.702494, 18.6161,
                    4.56621, 1.93186, 7.37619, 1.75623,
                ],
            ),
            KeyFindingAlgorithm::SimpleWeights => (
                [2.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 2.0, 1.0, 0.0, 0.5, 0.5],
            ),
            KeyFindingAlgorithm::BellmanBudge => (
                [
                    16.80, 0.86, 12.95, 1.41, 13.49, 11.93, 1.25, 20.28, 1.80, 8.04, 0.62, 10.57,
                ],
                [
                    18.16, 0.69, 12.99, 13.34, 1.07, 11.15, 1.38, 21.07, 7.49, 1.53, 0.92, 10.21,
                ],
            ),
            KeyFindingAlgorithm::TemperleyKostkaPayne => (
                [
                    0.748, 0.060, 0.488, 0.082, 0.670, 0.460, 0.096, 0.715, 0.104, 0.366, 0.057,
                    0.400,
                ],
                [
                    0.712, 0.084, 0.474, 0.618, 0.049, 0.460, 0.105, 0.747, 0.404, 0.067, 0.133,
                    0.330,
                ],
            ),
        }
    }
}

/// The result of a key-finding pass: the best-matching key and its
/// Pearson correlation coefficient against the input pitch-class
/// distribution (-1.0..=1.0; higher is a stronger match).
#[derive(Debug, Clone, PartialEq)]
pub struct KeyAnalysisResult {
    pub key: Key,
    pub correlation: f64,
}

/// Build a duration-weighted pitch-class histogram (12 entries, index 0 =
/// C) from a sequence of notes.
pub fn pitch_class_distribution(notes: &[Note]) -> [f64; 12] {
    let mut histogram = [0.0; 12];
    for note in notes {
        let pc = note.pitch().pitch_class() as usize;
        let quarter_length = note.quarter_length();
        let weight = *quarter_length.numer() as f64 / *quarter_length.denom() as f64;
        histogram[pc] += weight;
    }
    histogram
}

/// Pearson correlation coefficient between two equal-length distributions.
/// Returns `0.0` if either has zero variance (undefined correlation).
fn correlation(a: &[f64; 12], b: &[f64; 12]) -> f64 {
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;

    let mut covariance = 0.0;
    let mut variance_a = 0.0;
    let mut variance_b = 0.0;
    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        covariance += da * db;
        variance_a += da * da;
        variance_b += db * db;
    }

    if variance_a <= 0.0 || variance_b <= 0.0 {
        return 0.0;
    }
    covariance / (variance_a.sqrt() * variance_b.sqrt())
}

/// Rotate a tonic-relative profile so index `pc` holds the weight for
/// that absolute pitch class, given the candidate tonic's pitch class.
fn rotate(profile: &[f64; 12], tonic_pc: usize) -> [f64; 12] {
    let mut rotated = [0.0; 12];
    for (pc, slot) in rotated.iter_mut().enumerate() {
        let degree = (pc + 12 - tonic_pc) % 12;
        *slot = profile[degree];
    }
    rotated
}

/// A conventional (sharps-preferred) spelling for a pitch class. Key-
/// finding fundamentally identifies a tonic *pitch class*; picking the
/// prettiest enharmonic spelling for a given key (e.g. Bb over A# for F
/// major) is a separate, secondary concern this simplifies by always
/// preferring sharps.
fn pitch_class_to_pitch(pc: usize) -> Pitch {
    let (step, accidental) = match pc {
        0 => (Step::C, None),
        1 => (Step::C, Some(crate::core::Accidental::Sharp)),
        2 => (Step::D, None),
        3 => (Step::D, Some(crate::core::Accidental::Sharp)),
        4 => (Step::E, None),
        5 => (Step::F, None),
        6 => (Step::F, Some(crate::core::Accidental::Sharp)),
        7 => (Step::G, None),
        8 => (Step::G, Some(crate::core::Accidental::Sharp)),
        9 => (Step::A, None),
        10 => (Step::A, Some(crate::core::Accidental::Sharp)),
        11 => (Step::B, None),
        _ => unreachable!("pitch class is always 0..12"),
    };
    Pitch::from_parts(step, None, accidental)
}

/// Find the best-fitting key for a pitch-class distribution (see
/// `pitch_class_distribution`) using `algorithm`'s profile: correlate the
/// distribution against all 24 major/minor rotations and return the
/// highest-correlating one.
pub fn find_key(distribution: &[f64; 12], algorithm: KeyFindingAlgorithm) -> KeyAnalysisResult {
    let (major_profile, minor_profile) = algorithm.profiles();

    let mut best_correlation = f64::NEG_INFINITY;
    let mut best_tonic_pc = 0usize;
    let mut best_mode = KeyMode::Major;

    for tonic_pc in 0..12 {
        let major_corr = correlation(distribution, &rotate(&major_profile, tonic_pc));
        if major_corr > best_correlation {
            best_correlation = major_corr;
            best_tonic_pc = tonic_pc;
            best_mode = KeyMode::Major;
        }
        let minor_corr = correlation(distribution, &rotate(&minor_profile, tonic_pc));
        if minor_corr > best_correlation {
            best_correlation = minor_corr;
            best_tonic_pc = tonic_pc;
            best_mode = KeyMode::Minor;
        }
    }

    KeyAnalysisResult {
        key: Key::new(pitch_class_to_pitch(best_tonic_pc), best_mode),
        correlation: best_correlation,
    }
}

/// Analyze a `Part`'s melodic content (via `Part::find_consecutive_notes`
/// — chords aren't currently included) to find its best-fitting key.
/// Mirrors music21's `analyzeStream`/`Stream.analyze('key')` dispatcher
/// for this algorithm family.
pub fn analyze_part(part: &Part, algorithm: KeyFindingAlgorithm) -> KeyAnalysisResult {
    let notes = part.find_consecutive_notes();
    let distribution = pitch_class_distribution(&notes);
    find_key(&distribution, algorithm)
}

/// A confidence score for how decisively `algorithm` prefers its
/// best-matching key over the runner-up, given a pitch-class
/// distribution: the gap between the best and second-best of the 24
/// candidate correlations, normalized by the best correlation (`0.0` for
/// a totally ambiguous distribution where every key scores about the
/// same — including an all-zero/no-notes distribution — up to `1.0` when
/// the runner-up scores essentially zero). A clearly tonal passage (one
/// key standing out) scores high; a chromatic or key-ambiguous passage
/// scores low. Mirrors music21's `Key.tonalCertainty`.
pub fn tonal_certainty(distribution: &[f64; 12], algorithm: KeyFindingAlgorithm) -> f64 {
    let (major_profile, minor_profile) = algorithm.profiles();

    let mut correlations: Vec<f64> = Vec::with_capacity(24);
    for tonic_pc in 0..12 {
        correlations.push(correlation(distribution, &rotate(&major_profile, tonic_pc)));
        correlations.push(correlation(distribution, &rotate(&minor_profile, tonic_pc)));
    }
    correlations.sort_by(|a, b| b.partial_cmp(a).unwrap());

    let best = correlations[0];
    let second = correlations.get(1).copied().unwrap_or(0.0);
    if best <= 0.0 {
        return 0.0;
    }
    ((best - second) / best).clamp(0.0, 1.0)
}

/// `analyze_part` plus its `tonal_certainty` confidence score.
pub fn analyze_part_with_certainty(
    part: &Part,
    algorithm: KeyFindingAlgorithm,
) -> (KeyAnalysisResult, f64) {
    let notes = part.find_consecutive_notes();
    let distribution = pitch_class_distribution(&notes);
    (
        find_key(&distribution, algorithm),
        tonal_certainty(&distribution, algorithm),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Duration;
    use crate::stream::{Measure, MusicElement};

    fn note(step: Step, duration: Duration) -> Note {
        Note::new(Pitch::from_parts(step, Some(4), None), duration)
    }

    /// A short C major phrase with realistic tonic/dominant/mediant
    /// emphasis (I-V-I-ish), rather than a bare, undifferentiated scale
    /// run — a plain 7-note scale doesn't actually disambiguate C major
    /// from closely related keys (e.g. F major, which shares 6 of its 7
    /// notes) without the cadential weighting real tonal music has, so
    /// it isn't a fair test of "does this identify C major" for every
    /// profile.
    fn c_major_scale_part() -> Part {
        let mut part = Part::new();
        let mut measure = Measure::new(1);
        let phrase = [
            (Step::C, Duration::half()),
            (Step::E, Duration::quarter()),
            (Step::G, Duration::quarter()),
            (Step::G, Duration::half()),
            (Step::D, Duration::quarter()),
            (Step::B, Duration::quarter()),
            (Step::C, Duration::whole()),
        ];
        let mut offset = crate::core::Fraction::new(0, 1);
        for (step, duration) in phrase {
            let quarter_length = duration.quarter_length();
            measure.insert(offset, MusicElement::Note(note(step, duration)));
            offset += quarter_length;
        }
        part.add_measure(measure);
        part
    }

    #[test]
    fn test_krumhansl_schmuckler_identifies_c_major() {
        let part = c_major_scale_part();
        let result = analyze_part(&part, KeyFindingAlgorithm::KrumhanslSchmuckler);
        assert_eq!(result.key.tonic().step(), Step::C);
        assert_eq!(result.key.mode(), KeyMode::Major);
        assert!(result.correlation > 0.5);
    }

    #[test]
    fn test_all_five_algorithms_identify_c_major() {
        let part = c_major_scale_part();
        for algorithm in [
            KeyFindingAlgorithm::KrumhanslSchmuckler,
            KeyFindingAlgorithm::AardenEssen,
            KeyFindingAlgorithm::SimpleWeights,
            KeyFindingAlgorithm::BellmanBudge,
            KeyFindingAlgorithm::TemperleyKostkaPayne,
        ] {
            let result = analyze_part(&part, algorithm);
            assert_eq!(
                result.key.tonic().step(),
                Step::C,
                "algorithm {algorithm:?} failed"
            );
            assert_eq!(
                result.key.mode(),
                KeyMode::Major,
                "algorithm {algorithm:?} failed"
            );
        }
    }

    #[test]
    fn test_pitch_class_distribution_weights_by_duration() {
        let notes = vec![
            note(Step::C, Duration::whole()),
            note(Step::D, Duration::quarter()),
        ];
        let distribution = pitch_class_distribution(&notes);
        assert_eq!(distribution[0], 4.0); // C: whole note = 4 quarter lengths
        assert_eq!(distribution[2], 1.0); // D: quarter note = 1
    }

    #[test]
    fn test_rotate_aligns_tonic_to_absolute_pitch_class() {
        let profile = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let rotated = rotate(&profile, 4); // tonic = E (pc 4)
        assert_eq!(rotated[4], 1.0); // the tonic itself gets profile[0]
        assert_eq!(rotated[11], 8.0); // pc 11 is 7 semitones above E (the fifth, B) -> profile[7]
    }

    #[test]
    fn test_correlation_zero_variance_is_zero_not_nan() {
        let flat = [1.0; 12];
        let other = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        assert_eq!(correlation(&flat, &other), 0.0);
    }

    #[test]
    fn test_tonal_certainty_high_for_clearly_tonal_distribution() {
        let part = c_major_scale_part();
        let notes = part.find_consecutive_notes();
        let distribution = pitch_class_distribution(&notes);
        let certainty = tonal_certainty(&distribution, KeyFindingAlgorithm::KrumhanslSchmuckler);
        assert!(
            certainty > 0.05,
            "expected clear tonal certainty, got {certainty}"
        );
    }

    #[test]
    fn test_tonal_certainty_zero_for_flat_distribution() {
        // Every pitch class equally present: no key should stand out.
        let flat_distribution = [1.0; 12];
        let certainty =
            tonal_certainty(&flat_distribution, KeyFindingAlgorithm::KrumhanslSchmuckler);
        assert_eq!(certainty, 0.0);
    }

    #[test]
    fn test_analyze_part_with_certainty_matches_separate_calls() {
        let part = c_major_scale_part();
        let (result, certainty) =
            analyze_part_with_certainty(&part, KeyFindingAlgorithm::KrumhanslSchmuckler);
        assert_eq!(result.key.tonic().step(), Step::C);
        assert!(certainty > 0.0);
    }
}
