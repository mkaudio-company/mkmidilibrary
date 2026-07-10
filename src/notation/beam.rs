//! Beaming logic — grouping consecutive short-duration notes under a beam
//! according to a time signature's beat structure.

use crate::core::{Duration, Fraction};
use crate::notation::TimeSignature;

/// Which part of a beam group a note occupies, for rendering: the first
/// note of a group, one connected on both sides, or the last note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeamType {
    /// The first note of a beam group.
    Start,
    /// A note within a beam group, connected on both sides.
    Continue,
    /// The last note of a beam group.
    Stop,
}

/// One beam group: the indices (into the slice originally passed to
/// `compute_beams`) of the notes it connects, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beam {
    indices: Vec<usize>,
}

impl Beam {
    /// The indices (into the original `durations` slice given to
    /// `compute_beams`) of the notes in this beam group.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// The `BeamType` of the note at `position` within this group (0 =
    /// first note in the group, not a global index into `durations`).
    /// `None` if `position` is out of range.
    pub fn beam_type_at(&self, position: usize) -> Option<BeamType> {
        if position >= self.indices.len() {
            return None;
        }
        Some(if position == 0 {
            BeamType::Start
        } else if position == self.indices.len() - 1 {
            BeamType::Stop
        } else {
            BeamType::Continue
        })
    }
}

/// Group beam-eligible notes (duration strictly between 0 and a quarter
/// note) from `durations` — given in order, spanning a single measure
/// starting at offset 0 — into beam groups according to
/// `time_signature`'s beat structure (`TimeSignature::beat_offsets`/
/// `beat_sequence`): consecutive eligible notes are grouped together only
/// while they remain within the same beat. A rest placeholder should be
/// represented by a duration of a quarter note or longer at that
/// position (this function only sees durations, not rests/notes, so the
/// caller is responsible for that substitution) — it, or crossing into
/// the next beat, breaks the current group. A single eligible note with
/// no beam-mate on either side is not returned as its own group (nothing
/// to beam it to). Mirrors a simplified form of music21's `makeBeams`.
pub fn compute_beams(durations: &[Duration], time_signature: &TimeSignature) -> Vec<Beam> {
    let beat_starts = time_signature.beat_offsets();
    let beat_lengths = time_signature.beat_sequence();

    let beat_index_for = |offset: Fraction| -> usize {
        for (i, (&start, &length)) in beat_starts.iter().zip(beat_lengths.iter()).enumerate() {
            if offset >= start && offset < start + length {
                return i;
            }
        }
        beat_starts.len().saturating_sub(1)
    };

    let mut groups: Vec<Beam> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut current_beat: Option<usize> = None;
    let mut offset = Fraction::new(0, 1);

    for (i, duration) in durations.iter().enumerate() {
        let length = duration.quarter_length();
        let eligible = length > Fraction::new(0, 1) && length < Fraction::new(1, 1);
        let beat = beat_index_for(offset);

        if eligible && current_beat == Some(beat) {
            current.push(i);
        } else {
            if current.len() > 1 {
                groups.push(Beam {
                    indices: std::mem::take(&mut current),
                });
            } else {
                current.clear();
            }
            if eligible {
                current.push(i);
                current_beat = Some(beat);
            } else {
                current_beat = None;
            }
        }

        offset += length;
    }
    if current.len() > 1 {
        groups.push(Beam { indices: current });
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_beams_groups_within_a_beat_in_common_time() {
        // 4/4: 8 eighth notes, one beat = 1 quarter length = 2 eighths.
        // Expect 4 separate beam groups of 2 notes each.
        let durations: Vec<Duration> = (0..8).map(|_| Duration::eighth()).collect();
        let beams = compute_beams(&durations, &TimeSignature::common_time());

        assert_eq!(beams.len(), 4);
        for (i, beam) in beams.iter().enumerate() {
            assert_eq!(beam.indices(), &[i * 2, i * 2 + 1]);
            assert_eq!(beam.beam_type_at(0), Some(BeamType::Start));
            assert_eq!(beam.beam_type_at(1), Some(BeamType::Stop));
        }
    }

    #[test]
    fn test_compute_beams_quarter_notes_break_grouping() {
        // Eighth+eighth (beat 1), quarter (beat 2), eighth+eighth (beat
        // 3): the intervening quarter note must prevent the two eighth
        // pairs from being merged into one beam group.
        let durations = vec![
            Duration::eighth(),
            Duration::eighth(),
            Duration::quarter(),
            Duration::eighth(),
            Duration::eighth(),
        ];
        let beams = compute_beams(&durations, &TimeSignature::common_time());
        assert_eq!(beams.len(), 2);
        assert_eq!(beams[0].indices(), &[0, 1]);
        assert_eq!(beams[1].indices(), &[3, 4]);
    }

    #[test]
    fn test_compute_beams_compound_meter_groups_whole_beat() {
        // 6/8: each beat is a full dotted quarter (3 eighth notes). Six
        // eighth notes should beam as two groups of 3, not three groups
        // of 2 (which a naive "pair eighths" rule would produce).
        let durations: Vec<Duration> = (0..6).map(|_| Duration::eighth()).collect();
        let beams = compute_beams(&durations, &TimeSignature::six_eight());

        assert_eq!(beams.len(), 2);
        assert_eq!(beams[0].indices(), &[0, 1, 2]);
        assert_eq!(beams[1].indices(), &[3, 4, 5]);
    }

    #[test]
    fn test_compute_beams_does_not_cross_beat_boundary() {
        // 3/4: three quarter-note beats. Splitting each into two eighths
        // must never let a beam span across a beat boundary.
        let durations: Vec<Duration> = (0..6).map(|_| Duration::eighth()).collect();
        let beams = compute_beams(&durations, &TimeSignature::three_four());

        assert_eq!(beams.len(), 3);
        assert_eq!(beams[0].indices(), &[0, 1]);
        assert_eq!(beams[1].indices(), &[2, 3]);
        assert_eq!(beams[2].indices(), &[4, 5]);
    }
}
