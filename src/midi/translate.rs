//! Score to MIDI conversion and vice versa

use super::file::MidiFile;
use super::message::{MetaEvent, MidiMessage};
use super::track::MidiTrack;
use super::MidiFormat;

use crate::core::{Duration, Fraction, Note, Pitch};
use crate::stream::{Measure, Part, Score};

/// Conversion from Score to MIDI
pub struct ScoreToMidi {
    /// Ticks per quarter note
    ticks_per_quarter: u16,
}

impl ScoreToMidi {
    /// Create a new converter
    pub fn new() -> Self {
        Self {
            ticks_per_quarter: 480,
        }
    }

    /// Set ticks per quarter note
    pub fn with_ticks_per_quarter(mut self, tpq: u16) -> Self {
        self.ticks_per_quarter = tpq;
        self
    }

    /// Convert a Score to a MidiFile
    pub fn convert(&self, score: &Score) -> MidiFile {
        let mut midi = MidiFile::with_format(MidiFormat::MultiTrack, self.ticks_per_quarter);

        // Create tempo track
        let tempo_track = midi.add_track();
        tempo_track.set_name("Tempo");

        // Add initial tempo if specified
        if let Some(tempo) = score.tempo() {
            tempo_track.add_tempo(0, tempo.bpm());
        } else {
            tempo_track.add_tempo(0, 120.0);
        }

        // Add time signature if specified
        if let Some(ts) = score.time_signature() {
            tempo_track.add_time_signature(0, ts.numerator(), ts.denominator());
        }

        // Add key signature if specified
        if let Some(ks) = score.key_signature() {
            tempo_track.add_key_signature(0, ks.sharps(), ks.is_minor());
        }

        tempo_track.add_end_of_track();

        // Convert each part to a track
        for (i, part) in score.parts().iter().enumerate() {
            let track = midi.add_track();
            track.set_name(part.name().unwrap_or(&format!("Part {}", i + 1)));

            // Set initial program if specified
            if let Some(instrument) = part.instrument() {
                track.add_program_change(0, i as u8, instrument.midi_program());
            }

            self.convert_part(part, track, i as u8);
            track.add_end_of_track();
        }

        midi.link_note_events();
        midi
    }

    /// Convert a single Part to a MidiTrack
    fn convert_part(&self, part: &Part, track: &mut MidiTrack, channel: u8) {
        let mut current_tick: u64 = 0;

        for measure in part.measures() {
            self.convert_measure(measure, track, channel, &mut current_tick);
        }
    }

    /// Convert a Measure to events
    fn convert_measure(
        &self,
        measure: &Measure,
        track: &mut MidiTrack,
        channel: u8,
        current_tick: &mut u64,
    ) {
        // Get measure start tick
        let measure_start = *current_tick;

        // Convert elements
        for (offset, element) in measure.elements() {
            let element_tick = measure_start + self.fraction_to_ticks(*offset);

            match element {
                crate::stream::MusicElement::Note(note) => {
                    let duration_ticks = self.fraction_to_ticks(note.quarter_length());
                    track.add_note(
                        element_tick,
                        duration_ticks,
                        channel,
                        note.midi(),
                        note.volume().velocity,
                    );
                }
                crate::stream::MusicElement::Chord(chord) => {
                    let duration_ticks = self.fraction_to_ticks(chord.quarter_length());
                    for note in chord.notes() {
                        track.add_note(
                            element_tick,
                            duration_ticks,
                            channel,
                            note.midi(),
                            note.volume().velocity,
                        );
                    }
                }
                crate::stream::MusicElement::Rest(_) => {
                    // Rests don't produce MIDI events
                }
            }
        }

        // Advance to next measure
        *current_tick += self.fraction_to_ticks(measure.duration());
    }

    /// Convert a fraction (quarter lengths) to ticks
    fn fraction_to_ticks(&self, fraction: Fraction) -> u64 {
        let ticks = fraction * Fraction::from(self.ticks_per_quarter as i64);
        (*ticks.numer() / *ticks.denom()) as u64
    }
}

impl Default for ScoreToMidi {
    fn default() -> Self {
        Self::new()
    }
}

/// Conversion from MIDI to Score
pub struct MidiToScore {
    /// Quantization grid (in ticks)
    quantize_ticks: Option<u64>,
    /// Whether to separate voices
    separate_voices: bool,
}

impl MidiToScore {
    /// Create a new converter
    pub fn new() -> Self {
        Self {
            quantize_ticks: None,
            separate_voices: false,
        }
    }

    /// Set quantization
    pub fn with_quantization(mut self, ticks: u64) -> Self {
        self.quantize_ticks = Some(ticks);
        self
    }

    /// Enable voice separation
    pub fn with_voice_separation(mut self, enabled: bool) -> Self {
        self.separate_voices = enabled;
        self
    }

    /// Convert a MidiFile to a Score
    pub fn convert(&self, midi: &MidiFile) -> Score {
        let mut score = Score::new();
        let tpq = midi.ticks_per_quarter();

        // Extract tempo and time signature from first track
        if let Some(track) = midi.tracks().first() {
            for event in track.events() {
                match event.message() {
                    MidiMessage::Meta(MetaEvent::Tempo(us)) => {
                        let bpm = 60_000_000.0 / *us as f64;
                        score.set_tempo(crate::notation::Tempo::new(bpm));
                    }
                    MidiMessage::Meta(MetaEvent::TimeSignature {
                        numerator,
                        denominator_power,
                        ..
                    }) => {
                        let denom = 1 << denominator_power;
                        score.set_time_signature(crate::notation::TimeSignature::new(
                            *numerator, denom,
                        ));
                    }
                    MidiMessage::Meta(MetaEvent::KeySignature {
                        sharps_flats,
                        minor,
                    }) => {
                        score.set_key_signature(crate::notation::KeySignature::new(
                            *sharps_flats,
                            *minor,
                        ));
                    }
                    _ => {}
                }
            }
        }

        // Convert tracks to parts (skip tempo track if multi-track)
        let start_track = if midi.format() == MidiFormat::MultiTrack && midi.num_tracks() > 1 {
            1
        } else {
            0
        };

        for track in midi.tracks().iter().skip(start_track) {
            let part = self.convert_track(track, tpq);
            score.add_part(part);
        }

        score
    }

    /// Convert a MidiTrack to a Part
    fn convert_track(&self, track: &MidiTrack, tpq: u16) -> Part {
        let mut part = Part::new();

        if let Some(name) = track.name() {
            part.set_name(name);
        }

        // Collect note events with their durations
        let mut notes: Vec<(u64, u64, u8, u8)> = Vec::new(); // (start, duration, key, velocity)

        for event in track.events() {
            if event.is_note_on() {
                if let (Some(key), Some(vel), Some(duration)) =
                    (event.key(), event.velocity(), event.tick_duration(track.events()))
                {
                    notes.push((event.tick(), duration, key, vel));
                }
            }
        }

        // Sort by start time
        notes.sort_by_key(|(start, _, _, _)| *start);

        // Determine time signature for measure length
        let beats_per_measure = 4; // Default 4/4
        let ticks_per_measure = tpq as u64 * beats_per_measure;

        // Group notes by measure
        let mut current_measure = Measure::new(1);
        let mut measure_start_tick: u64 = 0;
        let mut measure_number = 1;

        for (start, duration, key, velocity) in notes {
            // Check if we need to start a new measure
            while start >= measure_start_tick + ticks_per_measure {
                part.add_measure(current_measure);
                measure_number += 1;
                current_measure = Measure::new(measure_number);
                measure_start_tick += ticks_per_measure;
            }

            // Create note
            let pitch = Pitch::from_midi(key);
            let quarter_length = Fraction::new(duration as i64, tpq as i64);
            let duration_obj = Duration::from_quarter_length(quarter_length);
            let mut note = Note::new(pitch, duration_obj);
            note.set_velocity(velocity);

            // Calculate offset within measure
            let offset_ticks = start - measure_start_tick;
            let offset = Fraction::new(offset_ticks as i64, tpq as i64);

            current_measure.insert(offset, crate::stream::MusicElement::Note(note));
        }

        // Add final measure
        part.add_measure(current_measure);

        part
    }
}

impl Default for MidiToScore {
    fn default() -> Self {
        Self::new()
    }
}

// Implement From traits for convenience

impl From<&Score> for MidiFile {
    fn from(score: &Score) -> MidiFile {
        ScoreToMidi::new().convert(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Duration, Note, Pitch, Step};
    use crate::stream::Score;

    #[test]
    fn test_score_to_midi() {
        let mut score = Score::new();
        let mut part = Part::new();
        part.set_name("Piano");

        let mut measure = Measure::new(1);
        let note = Note::new(
            Pitch::from_parts(Step::C, Some(4), None),
            Duration::quarter(),
        );
        measure.append(crate::stream::MusicElement::Note(note));
        part.add_measure(measure);

        score.add_part(part);

        let midi = ScoreToMidi::new().convert(&score);

        assert_eq!(midi.num_tracks(), 2); // Tempo + Piano
        assert_eq!(midi.track(1).unwrap().name(), Some("Piano"));
    }

    #[test]
    fn test_fraction_to_ticks() {
        let converter = ScoreToMidi::new().with_ticks_per_quarter(480);

        // Quarter note = 480 ticks
        assert_eq!(
            converter.fraction_to_ticks(Fraction::new(1, 1)),
            480
        );

        // Half note = 960 ticks
        assert_eq!(
            converter.fraction_to_ticks(Fraction::new(2, 1)),
            960
        );

        // Eighth note = 240 ticks
        assert_eq!(
            converter.fraction_to_ticks(Fraction::new(1, 2)),
            240
        );
    }
}
