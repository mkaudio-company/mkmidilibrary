//! Part representation
//!
//! A Part represents a single instrument part in a score.

use std::fmt;

use crate::core::{
    update_accidental_display, AccidentalDisplay, Duration, Fraction, Interval, Note, Pitch, Tie,
    TieType,
};
use crate::notation::{compute_beams, Beam, Clef, KeySignature, TimeSignature};

use super::base::MusicElement;
use super::measure::Measure;
use super::voice::Voice;

/// One element reached by `Part::recurse`, carrying its full positional
/// context (which measure it came from, its offset within that measure,
/// and its absolute offset from the start of the part) alongside the
/// element itself.
#[derive(Debug, Clone, PartialEq)]
pub struct RecursedElement {
    /// The measure number this element belongs to.
    pub measure_number: u32,
    /// Offset within that measure, in quarter lengths.
    pub offset_in_measure: Fraction,
    /// Offset from the start of the part, in quarter lengths.
    pub absolute_offset: Fraction,
    /// The element itself.
    pub element: MusicElement,
}

/// Instrument information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    /// Instrument name
    name: String,
    /// Abbreviated name
    abbreviation: Option<String>,
    /// MIDI program number (0-127)
    midi_program: u8,
    /// MIDI channel (0-15)
    midi_channel: Option<u8>,
    /// Transposition in semitones
    transposition: i8,
}

impl Instrument {
    /// Create a new instrument
    pub fn new(name: impl Into<String>, midi_program: u8) -> Self {
        Self {
            name: name.into(),
            abbreviation: None,
            midi_program,
            midi_channel: None,
            transposition: 0,
        }
    }

    /// Create a piano
    pub fn piano() -> Self {
        Self::new("Piano", 0)
    }

    /// Create a violin
    pub fn violin() -> Self {
        Self::new("Violin", 40)
    }

    /// Create a flute
    pub fn flute() -> Self {
        Self::new("Flute", 73)
    }

    /// Create a trumpet
    pub fn trumpet() -> Self {
        let mut inst = Self::new("Trumpet", 56);
        inst.transposition = -2; // Bb trumpet
        inst
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Get the abbreviation
    pub fn abbreviation(&self) -> Option<&str> {
        self.abbreviation.as_deref()
    }

    /// Set the abbreviation
    pub fn set_abbreviation(&mut self, abbr: impl Into<String>) {
        self.abbreviation = Some(abbr.into());
    }

    /// Get the MIDI program number
    pub fn midi_program(&self) -> u8 {
        self.midi_program
    }

    /// Set the MIDI program number
    pub fn set_midi_program(&mut self, program: u8) {
        self.midi_program = program;
    }

    /// Get the MIDI channel
    pub fn midi_channel(&self) -> Option<u8> {
        self.midi_channel
    }

    /// Set the MIDI channel
    pub fn set_midi_channel(&mut self, channel: u8) {
        self.midi_channel = Some(channel);
    }

    /// Get the transposition
    pub fn transposition(&self) -> i8 {
        self.transposition
    }

    /// Set the transposition
    pub fn set_transposition(&mut self, semitones: i8) {
        self.transposition = semitones;
    }
}

impl Default for Instrument {
    fn default() -> Self {
        Self::piano()
    }
}

/// A single instrument part
#[derive(Debug, Clone, Default)]
pub struct Part {
    /// Part name
    name: Option<String>,
    /// Abbreviated name
    abbreviation: Option<String>,
    /// Instrument
    instrument: Option<Instrument>,
    /// Measures in this part
    measures: Vec<Measure>,
    /// Part ID
    id: Option<String>,
}

impl Part {
    /// Create a new empty part
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a part with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get the abbreviation
    pub fn abbreviation(&self) -> Option<&str> {
        self.abbreviation.as_deref()
    }

    /// Set the abbreviation
    pub fn set_abbreviation(&mut self, abbr: impl Into<String>) {
        self.abbreviation = Some(abbr.into());
    }

    /// Get the instrument
    pub fn instrument(&self) -> Option<&Instrument> {
        self.instrument.as_ref()
    }

    /// Set the instrument
    pub fn set_instrument(&mut self, instrument: Instrument) {
        self.instrument = Some(instrument);
    }

    /// Get the part ID
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Set the part ID
    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    /// Get all measures
    pub fn measures(&self) -> &[Measure] {
        &self.measures
    }

    /// Get mutable measures
    pub fn measures_mut(&mut self) -> &mut Vec<Measure> {
        &mut self.measures
    }

    /// Get a specific measure
    pub fn measure(&self, index: usize) -> Option<&Measure> {
        self.measures.get(index)
    }

    /// Get a mutable specific measure
    pub fn measure_mut(&mut self, index: usize) -> Option<&mut Measure> {
        self.measures.get_mut(index)
    }

    /// Get measure by number
    pub fn measure_by_number(&self, number: u32) -> Option<&Measure> {
        self.measures.iter().find(|m| m.number() == number)
    }

    /// Get the time signature in effect for the measure at `index`,
    /// searching backward through earlier measures if this measure didn't
    /// set one explicitly. `Measure` itself has no reference back to its
    /// containing `Part`, so this container-level context search is how
    /// this crate answers "what meter is measure N actually in" — mirrors
    /// music21's `Stream.timeSignature` context-search semantics (most
    /// measures in a real piece don't repeat the time signature).
    pub fn time_signature_at(&self, index: usize) -> Option<&TimeSignature> {
        self.measures
            .get(..=index)?
            .iter()
            .rev()
            .find_map(|m| m.time_signature())
    }

    /// Get the key signature in effect for the measure at `index`, with the
    /// same backward context search as `time_signature_at`.
    pub fn key_signature_at(&self, index: usize) -> Option<&KeySignature> {
        self.measures
            .get(..=index)?
            .iter()
            .rev()
            .find_map(|m| m.key_signature())
    }

    /// Get the clef in effect for the measure at `index`, with the same
    /// backward context search as `time_signature_at`.
    pub fn clef_at(&self, index: usize) -> Option<&Clef> {
        self.measures
            .get(..=index)?
            .iter()
            .rev()
            .find_map(|m| m.clef())
    }

    /// Get the actual duration of the measure at `index`, correctly
    /// falling back to the *prevailing* (context-searched) time signature
    /// when this measure doesn't set one explicitly — the context-aware
    /// fix for the fact that `Measure::duration()` alone has no way to see
    /// earlier measures and silently assumes 4/4 in that case.
    pub fn measure_duration(&self, index: usize) -> Fraction {
        let measure = match self.measures.get(index) {
            Some(m) => m,
            None => return Fraction::new(0, 1),
        };
        if let Some(explicit) = measure.explicit_duration() {
            return explicit;
        }
        if measure.time_signature().is_some() {
            return measure.duration();
        }
        match self.time_signature_at(index) {
            Some(ts) => ts.bar_duration(),
            None => Fraction::new(4, 1),
        }
    }

    /// Add a measure
    pub fn add_measure(&mut self, measure: Measure) {
        self.measures.push(measure);
    }

    /// Insert a measure at index
    pub fn insert_measure(&mut self, index: usize, measure: Measure) {
        self.measures.insert(index, measure);
    }

    /// Remove a measure
    pub fn remove_measure(&mut self, index: usize) -> Option<Measure> {
        if index < self.measures.len() {
            Some(self.measures.remove(index))
        } else {
            None
        }
    }

    /// Get the number of measures
    pub fn num_measures(&self) -> usize {
        self.measures.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.measures.is_empty()
    }

    /// Clear all measures
    pub fn clear(&mut self) {
        self.measures.clear();
    }

    /// Get total duration in quarter lengths. Uses `measure_duration` for
    /// each measure (not bare `Measure::duration()`) so that measures which
    /// inherit their time signature from an earlier measure are counted
    /// correctly instead of silently defaulting to 4/4.
    pub fn duration(&self) -> crate::core::Fraction {
        (0..self.measures.len())
            .map(|i| self.measure_duration(i))
            .sum()
    }

    /// Iterate over all notes in the part
    pub fn notes(&self) -> impl Iterator<Item = &crate::core::Note> {
        self.measures.iter().flat_map(|m| m.notes())
    }

    /// Iterate over all chords in the part
    pub fn chords(&self) -> impl Iterator<Item = &crate::core::Chord> {
        self.measures.iter().flat_map(|m| m.chords())
    }

    /// Create measures up to a given number if they don't exist
    pub fn ensure_measures(&mut self, count: usize) {
        while self.measures.len() < count {
            let num = self.measures.len() as u32 + 1;
            self.measures.push(Measure::new(num));
        }
    }

    /// Partition a flat, offset-tagged sequence of elements (e.g. from
    /// `Part::flatten`, or freshly-built elements with no measure
    /// structure yet) into measures of `time_signature`'s bar duration,
    /// producing a new `Part` with that time signature set on its first
    /// measure. Mirrors music21's `Stream.makeMeasures`.
    pub fn make_measures(
        elements: &[(Fraction, MusicElement)],
        time_signature: TimeSignature,
    ) -> Part {
        let bar_duration = time_signature.bar_duration();
        let mut part = Part::new();

        let num_measures = if elements.is_empty() {
            1
        } else {
            let max_offset = elements
                .iter()
                .map(|(offset, element)| *offset + element.quarter_length())
                .fold(Fraction::new(0, 1), |a, b| if b > a { b } else { a });
            (max_offset / bar_duration).ceil().to_integer().max(1) as usize
        };

        for i in 0..num_measures {
            let mut measure = Measure::new(i as u32 + 1);
            if i == 0 {
                measure.set_time_signature(time_signature);
            }
            part.add_measure(measure);
        }

        for (offset, element) in elements {
            let last_index = part.num_measures() - 1;
            let measure_index = (*offset / bar_duration).to_integer().max(0) as usize;
            let measure_index = measure_index.min(last_index);
            let local_offset = *offset - bar_duration * Fraction::from(measure_index as i64);
            part.measures[measure_index].insert(local_offset, element.clone());
        }

        part
    }

    /// Split any note whose duration extends beyond its containing
    /// measure's boundary into tied fragments (`Tie::Start` /
    /// `Tie::Continue` / `Tie::Stop`) across as many subsequent measures
    /// as needed, creating new measures if the note extends past the
    /// last existing one. Assumes a monophonic part (this crate's
    /// `Measure` holds one flat `Stream`, not simultaneous `Voice`s) —
    /// chords and rests aren't split. Mirrors music21's
    /// `Stream.makeTies`.
    pub fn make_ties(&mut self) {
        let mut i = 0;
        while i < self.measures.len() {
            let bar_duration = self.measure_duration(i);
            let old_elements: Vec<(Fraction, MusicElement)> = self.measures[i].elements().to_vec();

            let mut kept: Vec<(Fraction, MusicElement)> = Vec::new();
            let mut carry: Vec<(Fraction, Pitch)> = Vec::new();

            for (offset, element) in old_elements {
                if let MusicElement::Note(note) = &element {
                    let end = offset + note.quarter_length();
                    if end > bar_duration {
                        let first_len = bar_duration - offset;
                        let mut first_note = note.clone();
                        first_note.set_duration(Duration::from_quarter_length(first_len));
                        first_note.set_tie(Some(Tie::start()));
                        kept.push((offset, MusicElement::Note(first_note)));
                        carry.push((note.quarter_length() - first_len, note.pitch().clone()));
                        continue;
                    }
                }
                kept.push((offset, element));
            }

            self.measures[i].clear();
            for (offset, element) in kept {
                self.measures[i].insert(offset, element);
            }

            let mut next_measure = i + 1;
            for (remaining, pitch) in carry {
                let mut remaining = remaining;
                loop {
                    if next_measure >= self.measures.len() {
                        self.ensure_measures(next_measure + 1);
                    }
                    let next_bar_duration = self.measure_duration(next_measure);
                    if remaining <= next_bar_duration {
                        let mut note =
                            Note::new(pitch.clone(), Duration::from_quarter_length(remaining));
                        note.set_tie(Some(Tie::stop()));
                        self.measures[next_measure]
                            .insert(Fraction::new(0, 1), MusicElement::Note(note));
                        break;
                    } else {
                        let mut note = Note::new(
                            pitch.clone(),
                            Duration::from_quarter_length(next_bar_duration),
                        );
                        note.set_tie(Some(Tie::new(TieType::Continue)));
                        self.measures[next_measure]
                            .insert(Fraction::new(0, 1), MusicElement::Note(note));
                        remaining -= next_bar_duration;
                        next_measure += 1;
                    }
                }
            }

            i += 1;
        }
    }

    /// Merge runs of tied notes (`Tie::Start` followed by one or more
    /// `Tie::Continue`/`Tie::Stop` notes, immediately adjacent in time)
    /// back into single notes with the summed duration and no tie.
    /// Assumes a monophonic part, like `make_ties`. Mirrors music21's
    /// `Stream.stripTies`.
    pub fn strip_ties(&mut self) {
        let recursed = self.recurse();

        let mut merges: Vec<(usize, usize, Fraction)> = Vec::new();
        let mut i = 0;
        while i < recursed.len() {
            let is_start = matches!(
                &recursed[i].element,
                MusicElement::Note(n) if n.tie().map(|t| t.type_ == TieType::Start).unwrap_or(false)
            );
            if is_start {
                let mut total = recursed[i].element.quarter_length();
                let mut j = i + 1;
                while j < recursed.len() {
                    let continuation_type = match &recursed[j].element {
                        MusicElement::Note(n) => n.tie().map(|t| t.type_),
                        _ => None,
                    };
                    match continuation_type {
                        Some(TieType::Continue) => {
                            total += recursed[j].element.quarter_length();
                            j += 1;
                        }
                        Some(TieType::Stop) => {
                            total += recursed[j].element.quarter_length();
                            j += 1;
                            break;
                        }
                        _ => break,
                    }
                }
                if j > i + 1 {
                    merges.push((i, j - 1, total));
                    i = j;
                    continue;
                }
            }
            i += 1;
        }

        let mut removals: Vec<(u32, Fraction)> = Vec::new();
        for (start, end, total_duration) in merges {
            let first = &recursed[start];
            if let Some(measure) = self
                .measures
                .iter_mut()
                .find(|m| m.number() == first.measure_number)
            {
                if let Some((_, MusicElement::Note(note))) = measure
                    .stream_mut()
                    .elements_mut()
                    .iter_mut()
                    .find(|(o, _)| *o == first.offset_in_measure)
                {
                    note.set_duration(Duration::from_quarter_length(total_duration));
                    note.set_tie(None);
                }
            }
            for recursed_element in recursed.iter().take(end + 1).skip(start + 1) {
                removals.push((
                    recursed_element.measure_number,
                    recursed_element.offset_in_measure,
                ));
            }
        }

        for (measure_number, offset) in removals {
            if let Some(measure) = self
                .measures
                .iter_mut()
                .find(|m| m.number() == measure_number)
            {
                measure
                    .stream_mut()
                    .elements_mut()
                    .retain(|(o, _)| *o != offset);
            }
        }
    }

    /// Find adjacent (strictly consecutive in time, no gap), same-pitch
    /// note pairs that aren't already tied, and tie them together
    /// (`Tie::Start` on the first, `Tie::Stop` on the second). Assumes a
    /// monophonic part, like `make_ties`. Mirrors music21's
    /// `Stream.extendTies`.
    pub fn extend_ties(&mut self) {
        let recursed = self.recurse();

        let mut to_tie: Vec<((u32, Fraction), (u32, Fraction))> = Vec::new();
        for pair in recursed.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let (a_note, b_note) = match (&a.element, &b.element) {
                (MusicElement::Note(an), MusicElement::Note(bn)) => (an, bn),
                _ => continue,
            };
            if a_note.tie().is_some() || b_note.tie().is_some() {
                continue;
            }
            let a_end = a.absolute_offset + a_note.quarter_length();
            if a_note.pitch() == b_note.pitch() && a_end == b.absolute_offset {
                to_tie.push((
                    (a.measure_number, a.offset_in_measure),
                    (b.measure_number, b.offset_in_measure),
                ));
            }
        }

        for ((a_measure, a_offset), (b_measure, b_offset)) in to_tie {
            self.set_note_tie(a_measure, a_offset, Tie::start());
            self.set_note_tie(b_measure, b_offset, Tie::stop());
        }
    }

    /// Locate the note at `(measure_number, offset_in_measure)` and set
    /// its tie, if found (used by `extend_ties`).
    fn set_note_tie(&mut self, measure_number: u32, offset: Fraction, tie: Tie) {
        if let Some(measure) = self
            .measures
            .iter_mut()
            .find(|m| m.number() == measure_number)
        {
            if let Some((_, MusicElement::Note(note))) = measure
                .stream_mut()
                .elements_mut()
                .iter_mut()
                .find(|(o, _)| *o == offset)
            {
                note.set_tie(Some(tie));
            }
        }
    }

    /// Compute beam groupings for the notes in measure `index`, using
    /// that measure's context-searched prevailing time signature (via
    /// `time_signature_at`, defaulting to common time if none is set
    /// anywhere). Mirrors music21's `Stream.makeBeams` (per measure).
    pub fn make_beams(&self, index: usize) -> Vec<Beam> {
        let Some(measure) = self.measures.get(index) else {
            return Vec::new();
        };
        let time_signature = self.time_signature_at(index).copied().unwrap_or_default();
        let durations: Vec<Duration> = measure
            .elements()
            .iter()
            .map(|(_, element)| element.duration().clone())
            .collect();
        compute_beams(&durations, &time_signature)
    }

    /// Compute accidental display state for the notes in measure `index`
    /// (whether each note's accidental needs to actually be printed,
    /// given that measure's context-searched prevailing key signature —
    /// see `crate::core::update_accidental_display`). Chords contribute
    /// each of their pitches in order; rests are skipped. Mirrors
    /// music21's `Stream.makeAccidentals` (per measure).
    pub fn make_accidentals(&self, index: usize) -> Vec<AccidentalDisplay> {
        let Some(measure) = self.measures.get(index) else {
            return Vec::new();
        };
        let key_signature = self.key_signature_at(index).cloned().unwrap_or_default();
        let pitches: Vec<Pitch> = measure
            .elements()
            .iter()
            .flat_map(|(_, element)| match element {
                MusicElement::Note(n) => vec![n.pitch().clone()],
                MusicElement::Chord(c) => c.pitches().into_iter().cloned().collect(),
                MusicElement::Rest(_) => Vec::new(),
            })
            .collect();
        update_accidental_display(&pitches, &key_signature)
    }

    /// Run the full notation pipeline: `make_ties` (applied in place,
    /// splitting any measure-overflowing notes), then compute beam
    /// groupings and accidental-display state for every measure. Beams
    /// and accidental display aren't stored on `Note`/`Measure` in this
    /// crate (there's nowhere to put them yet), so they're returned per
    /// measure for the caller — typically a renderer — to use. Mirrors
    /// music21's `Stream.makeNotation` pipeline.
    pub fn make_notation(&mut self) -> Vec<(Vec<Beam>, Vec<AccidentalDisplay>)> {
        self.make_ties();
        (0..self.measures.len())
            .map(|i| (self.make_beams(i), self.make_accidentals(i)))
            .collect()
    }

    /// Insert `element` at `offset` within measure `measure_index`,
    /// shifting every element already at or after that offset (in that
    /// same measure) later by `element`'s own duration, so nothing
    /// already there gets overwritten or overlapped. Mirrors music21's
    /// `Stream.insertAndShift`.
    pub fn insert_and_shift(
        &mut self,
        measure_index: usize,
        offset: Fraction,
        element: MusicElement,
    ) {
        let Some(measure) = self.measures.get_mut(measure_index) else {
            return;
        };
        let shift_amount = element.quarter_length();
        let old_elements = measure.elements().to_vec();
        measure.clear();
        for (o, e) in old_elements {
            if o >= offset {
                measure.insert(o + shift_amount, e);
            } else {
                measure.insert(o, e);
            }
        }
        measure.insert(offset, element);
    }

    /// Every `Note` in this part, in chronological order across all
    /// measures (skipping `Rest`s and `Chord`s) — the part's melodic
    /// line. Mirrors a scoped subset of music21's
    /// `Stream.findConsecutiveNotes`.
    pub fn find_consecutive_notes(&self) -> Vec<Note> {
        self.recurse()
            .into_iter()
            .filter_map(|r| match r.element {
                MusicElement::Note(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// The melodic interval between each consecutive pair of notes in
    /// `find_consecutive_notes` (so `n - 1` intervals for `n` notes).
    /// Mirrors music21's `Stream.melodicIntervals`.
    pub fn melodic_intervals(&self) -> Vec<Interval> {
        let notes = self.find_consecutive_notes();
        notes
            .windows(2)
            .map(|pair| Interval::between(pair[0].pitch(), pair[1].pitch()))
            .collect()
    }

    /// A copy of this part with every `Note`/`Chord` pitch transposed by
    /// `interval` (`Rest`s are unaffected). Key/time signatures are left
    /// as-is — this only transposes the pitched content, not the
    /// notated key (re-run `Measure::set_key_signature` separately if a
    /// coherent new key signature is also wanted). Mirrors a scoped
    /// subset of music21's whole-`Stream` `transpose`.
    pub fn transpose(&self, interval: &Interval) -> Part {
        let mut result = self.clone();
        for measure in &mut result.measures {
            let old_elements = measure.elements().to_vec();
            measure.clear();
            for (offset, element) in old_elements {
                let transposed = match element {
                    MusicElement::Note(n) => MusicElement::Note(n.transpose(interval)),
                    MusicElement::Chord(c) => MusicElement::Chord(c.transpose(interval)),
                    rest @ MusicElement::Rest(_) => rest,
                };
                measure.insert(offset, transposed);
            }
        }
        result
    }

    /// A copy of this part with every element's offset and duration
    /// scaled by `scalar` (e.g. `2` doubles all durations — an
    /// augmentation; `1/2` halves them — a diminution). Note this
    /// rescales the *content*, not each measure's own time-signature-
    /// derived bar duration, so the result generally needs its measures
    /// rebuilt (e.g. via `Part::make_measures` on the flattened result)
    /// to stay well-formed — mirrors a scoped subset of music21's whole-
    /// `Stream` `augmentOrDiminish`.
    pub fn augment_or_diminish(&self, scalar: Fraction) -> Part {
        let mut result = self.clone();
        for measure in &mut result.measures {
            let old_elements = measure.elements().to_vec();
            measure.clear();
            for (offset, element) in old_elements {
                let scaled = match element {
                    MusicElement::Note(n) => MusicElement::Note(n.augment_or_diminish(scalar)),
                    MusicElement::Chord(c) => {
                        let mut c = c;
                        let new_duration = c.duration().augment_or_diminish(scalar);
                        c.set_duration(new_duration);
                        MusicElement::Chord(c)
                    }
                    MusicElement::Rest(r) => MusicElement::Rest(r.augment_or_diminish(scalar)),
                };
                measure.insert(offset * scalar, scaled);
            }
        }
        result
    }

    /// Snap every element's offset and duration to the nearest multiple
    /// of `grid` (in quarter lengths, e.g. `Fraction::new(1, 4)` for a
    /// 16th-note grid), within each measure independently. Durations are
    /// floored to at least one grid unit. Mirrors music21's
    /// `Stream.quantize`.
    pub fn quantize(&mut self, grid: Fraction) {
        if grid <= Fraction::new(0, 1) {
            return;
        }
        for measure in &mut self.measures {
            let old_elements = measure.elements().to_vec();
            measure.clear();
            for (offset, mut element) in old_elements {
                let quantized_offset = round_to_grid(offset, grid);
                let quantized_len = round_to_grid(element.quarter_length(), grid).max(grid);
                let new_duration = Duration::from_quarter_length(quantized_len);
                match &mut element {
                    MusicElement::Note(n) => n.set_duration(new_duration),
                    MusicElement::Chord(c) => c.set_duration(new_duration),
                    MusicElement::Rest(r) => r.set_duration(new_duration),
                }
                measure.insert(quantized_offset, element);
            }
        }
    }

    /// Split every `Note` into consecutive tied fragments no longer than
    /// `quarter_length` each (a `Chord`/`Rest` is left as-is); if
    /// `add_ties` is `true`, mark the fragments `Tie::Start`/`Continue`/
    /// `Stop` (only when a note actually needed to be split into more
    /// than one piece). Mirrors music21's `Stream.sliceByQuarterLengths`.
    pub fn slice_by_quarter_lengths(&mut self, quarter_length: Fraction, add_ties: bool) {
        if quarter_length <= Fraction::new(0, 1) {
            return;
        }
        for measure in &mut self.measures {
            let old_elements = measure.elements().to_vec();
            measure.clear();
            for (offset, element) in old_elements {
                let MusicElement::Note(note) = &element else {
                    measure.insert(offset, element);
                    continue;
                };

                let mut piece_lengths = Vec::new();
                let mut remaining = note.quarter_length();
                while remaining > Fraction::new(0, 1) {
                    let len = if remaining < quarter_length {
                        remaining
                    } else {
                        quarter_length
                    };
                    piece_lengths.push(len);
                    remaining -= len;
                }

                let n = piece_lengths.len();
                let mut cursor = offset;
                for (i, len) in piece_lengths.into_iter().enumerate() {
                    let mut piece = note.clone();
                    piece.set_duration(Duration::from_quarter_length(len));
                    if add_ties && n > 1 {
                        piece.set_tie(Some(Tie::new(tie_type_for_position(i, n))));
                    }
                    measure.insert(cursor, MusicElement::Note(piece));
                    cursor += len;
                }
            }
        }
    }

    /// Split every `Note` at each beat boundary it crosses (using each
    /// measure's context-searched prevailing time signature), tying the
    /// resulting fragments together. A `Chord`/`Rest` is left as-is.
    /// Mirrors music21's `Stream.sliceByBeat`.
    pub fn slice_by_beat(&mut self) {
        for i in 0..self.measures.len() {
            let time_signature = self.time_signature_at(i).copied().unwrap_or_default();
            let mut boundaries = time_signature.beat_offsets();
            boundaries.push(time_signature.bar_duration());
            self.slice_measure_at(i, &boundaries);
        }
    }

    /// Split every `Note` at each of the given (within-measure) offsets
    /// it spans across, in every measure, tying the resulting fragments
    /// together. A `Chord`/`Rest` is left as-is. Mirrors music21's
    /// `Stream.sliceAtOffsets`.
    pub fn slice_at_offsets(&mut self, offsets: &[Fraction]) {
        for i in 0..self.measures.len() {
            self.slice_measure_at(i, offsets);
        }
    }

    /// Find the greatest common quarter-length divisor across every
    /// element in this part, and slice every note to that unit (via
    /// `slice_by_quarter_lengths`) — the smallest common rhythmic
    /// quantum that evenly divides everything already present. Mirrors
    /// music21's `Stream.sliceByGreatestDivisor`.
    pub fn slice_by_greatest_divisor(&mut self) {
        let mut divisor: Option<Fraction> = None;
        for measure in &self.measures {
            for (_, element) in measure.elements() {
                let len = element.quarter_length();
                divisor = Some(match divisor {
                    None => len,
                    Some(g) => fraction_gcd(g, len),
                });
            }
        }
        if let Some(g) = divisor {
            if g > Fraction::new(0, 1) {
                self.slice_by_quarter_lengths(g, true);
            }
        }
    }

    /// Shared implementation for `slice_by_beat`/`slice_at_offsets`:
    /// split every `Note` in measure `index` at each of `cut_points` that
    /// falls strictly inside its span, tying the fragments together.
    fn slice_measure_at(&mut self, index: usize, cut_points: &[Fraction]) {
        let Some(measure) = self.measures.get_mut(index) else {
            return;
        };
        let old_elements = measure.elements().to_vec();
        measure.clear();

        for (offset, element) in old_elements {
            let MusicElement::Note(note) = &element else {
                measure.insert(offset, element);
                continue;
            };
            let end = offset + note.quarter_length();

            let mut cuts: Vec<Fraction> = cut_points
                .iter()
                .copied()
                .filter(|&c| c > offset && c < end)
                .collect();
            if cuts.is_empty() {
                measure.insert(offset, element);
                continue;
            }
            cuts.sort();

            let mut points = vec![offset];
            points.extend(cuts);
            points.push(end);
            let n = points.len() - 1;

            for k in 0..n {
                let seg_start = points[k];
                let seg_len = points[k + 1] - points[k];
                let mut piece = note.clone();
                piece.set_duration(Duration::from_quarter_length(seg_len));
                piece.set_tie(Some(Tie::new(tie_type_for_position(k, n))));
                measure.insert(seg_start, MusicElement::Note(piece));
            }
        }
    }

    /// Every explicit time signature change in this part, paired with its
    /// absolute offset (from the start of the part, in quarter lengths)
    /// — i.e. only measures that set one *locally*, not every measure's
    /// context-searched prevailing signature. Mirrors music21's
    /// `Stream.getTimeSignatures`.
    pub fn get_time_signatures(&self) -> Vec<(Fraction, TimeSignature)> {
        self.measure_offset_map()
            .into_iter()
            .zip(self.measures.iter())
            .filter_map(|((offset, _), measure)| measure.time_signature().map(|ts| (offset, *ts)))
            .collect()
    }

    /// Each measure's absolute start offset (from the start of the part,
    /// in quarter lengths), paired with its measure number, in measure
    /// order. Mirrors music21's `measureOffsetMap`.
    pub fn measure_offset_map(&self) -> Vec<(Fraction, u32)> {
        let mut offset = Fraction::new(0, 1);
        let mut map = Vec::with_capacity(self.measures.len());
        for (i, measure) in self.measures.iter().enumerate() {
            map.push((offset, measure.number()));
            offset += self.measure_duration(i);
        }
        map
    }

    /// Recursively walk every element in every measure, in order,
    /// annotated with its full positional context (containing measure
    /// number, offset within that measure, and absolute offset from the
    /// start of the part). Mirrors (a scoped form of, since this crate's
    /// `Measure` doesn't nest `Voice`s) music21's `Stream.recurse`.
    pub fn recurse(&self) -> Vec<RecursedElement> {
        let mut result = Vec::new();
        let mut absolute_offset = Fraction::new(0, 1);
        for (i, measure) in self.measures.iter().enumerate() {
            for (offset_in_measure, element) in measure.elements() {
                result.push(RecursedElement {
                    measure_number: measure.number(),
                    offset_in_measure: *offset_in_measure,
                    absolute_offset: absolute_offset + *offset_in_measure,
                    element: element.clone(),
                });
            }
            absolute_offset += self.measure_duration(i);
        }
        result
    }

    /// A flat `(absolute offset, element)` list of every element across
    /// every measure — `recurse` without the extra positional context.
    /// Mirrors music21's `Stream.flatten` (formerly `Stream.flat`).
    pub fn flatten(&self) -> Vec<(Fraction, MusicElement)> {
        self.recurse()
            .into_iter()
            .map(|r| (r.absolute_offset, r.element))
            .collect()
    }

    /// All elements (from `flatten`) whose absolute offset falls in
    /// `[start, end)`. Mirrors music21's `Stream.getElementsByOffset`
    /// (the default, non-inclusive-of-`end`, non-`mustBeginInSpan`-only
    /// behavior).
    pub fn get_elements_by_offset(
        &self,
        start: Fraction,
        end: Fraction,
    ) -> Vec<(Fraction, MusicElement)> {
        self.flatten()
            .into_iter()
            .filter(|(offset, _)| *offset >= start && *offset < end)
            .collect()
    }

    /// The element (from `flatten`) with the latest absolute offset at or
    /// before `offset`, if any. Mirrors music21's
    /// `Stream.getElementAtOrBefore`.
    pub fn get_element_at_or_before(&self, offset: Fraction) -> Option<(Fraction, MusicElement)> {
        self.flatten()
            .into_iter()
            .filter(|(o, _)| *o <= offset)
            .max_by_key(|(o, _)| *o)
    }

    /// Whether this part is organized into measures (as opposed to a
    /// flat, unmeasured stream). Mirrors music21's `Stream.hasMeasures`;
    /// always true for any non-empty `Part` in this crate's model, since
    /// `Part` is always `Measure`-structured.
    pub fn has_measures(&self) -> bool {
        !self.measures.is_empty()
    }

    /// Whether any measure in this part is split into multiple
    /// simultaneous voices. Mirrors music21's `Stream.hasVoices`; always
    /// `false` in this crate today, since `Measure` doesn't yet nest
    /// `Voice`s (each measure holds a single flat `Stream`) — see
    /// `crate::stream::Voice`'s doc comment.
    pub fn has_voices(&self) -> bool {
        false
    }

    /// A basic notation well-formedness check: no measure's content
    /// overflows its resolved duration (using the same context-aware
    /// time-signature search as `measure_duration`, not each measure's
    /// own possibly-defaulted `Measure::duration()`). Mirrors a scoped
    /// subset of music21's `Stream.isWellFormedNotation`.
    pub fn is_well_formed_notation(&self) -> bool {
        (0..self.measures.len()).all(|i| {
            let measure = &self.measures[i];
            measure.content_duration() <= self.measure_duration(i)
        })
    }

    /// A copy of this part's measure structure (numbers, time/key
    /// signatures, clefs) with every measure's musical content removed —
    /// a scaffold for building a new part that shares the same measure
    /// layout. Mirrors music21's `Stream.template`.
    pub fn template(&self) -> Part {
        let mut templated = self.clone();
        for measure in templated.measures_mut() {
            measure.clear();
        }
        templated
    }

    /// Split this part's chords into separate single-voice parts: for
    /// each measure, every `Chord` element is distributed across N output
    /// parts (N = the largest chord encountered anywhere in this part),
    /// its pitches assigned high-to-low (part 0 gets the highest pitch,
    /// matching the usual "voice 1 is the top line" convention); a plain
    /// `Note`/`Rest` element is copied as-is into part 0 only. Mirrors a
    /// scoped subset of music21's `Stream.explode` (chords-to-parts, as
    /// opposed to `voicesToParts`, which splits actual `Voice` objects —
    /// see the free function `voices_to_parts` for that).
    pub fn explode(&self) -> Vec<Part> {
        let max_voices = self
            .measures
            .iter()
            .flat_map(|m| m.elements())
            .map(|(_, e)| match e {
                MusicElement::Chord(c) => c.notes().len(),
                _ => 1,
            })
            .max()
            .unwrap_or(1)
            .max(1);

        let base_name = self.name.clone().unwrap_or_else(|| "Part".to_string());
        let mut result: Vec<Part> = (0..max_voices)
            .map(|i| {
                let mut p = self.template();
                p.set_name(format!("{base_name} {}", i + 1));
                p
            })
            .collect();

        for (mi, measure) in self.measures.iter().enumerate() {
            for (offset, element) in measure.elements() {
                match element {
                    MusicElement::Chord(chord) => {
                        let mut pitches: Vec<Pitch> =
                            chord.pitches().into_iter().cloned().collect();
                        pitches.sort_by(|a, b| b.cmp(a));
                        for (vi, pitch) in pitches.into_iter().enumerate() {
                            if let Some(part_measure) = result[vi].measures.get_mut(mi) {
                                part_measure.insert(
                                    *offset,
                                    MusicElement::Note(Note::new(pitch, chord.duration().clone())),
                                );
                            }
                        }
                    }
                    other => {
                        if let Some(part_measure) = result[0].measures.get_mut(mi) {
                            part_measure.insert(*offset, other.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Renumber measures starting from 1
    pub fn renumber_measures(&mut self) {
        // Regression fix: this used to compute
        // `start + i - if has_pickup { 0 } else { 0 }`, which subtracts 0 in
        // both branches — a dead conditional that always evaluated to
        // `start + i` regardless of whether a pickup measure was present.
        // With a pickup at index 0 (numbered 0), the following measures
        // should be numbered 1, 2, 3, ... (i.e. `i` itself); without a
        // pickup, they should be 1, 2, 3, ... starting from index 0 (i.e.
        // `i + 1`).
        let has_pickup = self
            .measures
            .first()
            .map(|m| m.is_pickup())
            .unwrap_or(false);

        for (i, measure) in self.measures.iter_mut().enumerate() {
            let number = if has_pickup { i as u32 } else { i as u32 + 1 };
            measure.set_number(number);
        }
    }

    /// Expand repeat barlines (`Measure::is_repeat_start`/`is_repeat_end`)
    /// into a fully "unrolled" copy of this part: each repeated section
    /// is duplicated once immediately after its original occurrence. A
    /// repeat-end with no preceding repeat-start in the same pass repeats
    /// from the start of the part (or from just after the previous
    /// repeated section). Measures are renumbered afterward. Mirrors a
    /// scoped subset (single-pass, no volta/ending support) of music21's
    /// `Stream.expandRepeats`.
    pub fn expand_repeats(&self) -> Part {
        let mut result = self.clone();
        result.measures.clear();

        let mut section_start = 0usize;
        let mut i = 0usize;
        while i < self.measures.len() {
            if self.measures[i].is_repeat_start() {
                section_start = i;
            }
            result.measures.push(self.measures[i].clone());
            if self.measures[i].is_repeat_end() {
                for measure in &self.measures[section_start..=i] {
                    result.measures.push(measure.clone());
                }
                section_start = i + 1;
            }
            i += 1;
        }

        result.renumber_measures();
        result
    }

    /// A key/ambitus/melodic-interval-diversity analysis dispatcher,
    /// selected by name (mirrors music21's `Stream.analyze(method)`):
    /// `"key"` (using the Krumhansl-Schmuckler profile),
    /// `"ambitus"`/`"range"`, or `"melodicIntervalDiversity"`. Returns
    /// `None` for an unrecognized method name.
    pub fn analyze(&self, method: &str) -> Option<crate::analysis::PartAnalysisResult> {
        crate::analysis::analyze_part_by_method(self, method)
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("Unnamed Part");
        write!(f, "Part '{}' ({} measures)", name, self.measures.len())
    }
}

/// Convert a set of `Voice`s (independent melodic lines layered together,
/// e.g. within a single measure) into separate single-voice `Part`s, one
/// per voice, each holding a single measure with that voice's own
/// elements at their original offsets. Mirrors a scoped subset of
/// music21's `Stream.voicesToParts`.
pub fn voices_to_parts(voices: &[Voice]) -> Vec<Part> {
    voices
        .iter()
        .map(|voice| {
            let mut part = Part::with_name(format!("Voice {}", voice.id()));
            let mut measure = Measure::new(1);
            for (offset, element) in voice.elements() {
                measure.insert(*offset, element.clone());
            }
            part.add_measure(measure);
            part
        })
        .collect()
}

/// The inverse of `voices_to_parts`: flatten each given part into its own
/// `Voice` (using `Part::flatten`'s absolute offsets), so multiple parts
/// can be layered back together as independent voices sharing one
/// measure/staff. Mirrors a scoped subset of music21's
/// `Stream.partsToVoices`.
pub fn parts_to_voices(parts: &[&Part]) -> Vec<Voice> {
    parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let mut voice = Voice::new(i as u8 + 1);
            for (offset, element) in part.flatten() {
                voice.insert(offset, element);
            }
            voice
        })
        .collect()
}

/// Round `value` to the nearest multiple of `grid` (both in quarter
/// lengths). Used by `Part::quantize`.
fn round_to_grid(value: Fraction, grid: Fraction) -> Fraction {
    grid * (value / grid).round()
}

/// The greatest common divisor of two quarter-length durations,
/// expressed as a `Fraction` (rather than assuming an integer number of
/// quarters) — e.g. `gcd(3/2, 1/2) = 1/2`. Used by
/// `Part::slice_by_greatest_divisor`.
fn fraction_gcd(a: Fraction, b: Fraction) -> Fraction {
    use num::Integer;
    let common_denom = a.denom() * b.denom();
    let a_scaled = a.numer() * b.denom();
    let b_scaled = b.numer() * a.denom();
    Fraction::new(a_scaled.gcd(&b_scaled), common_denom)
}

/// The `TieType` for fragment `index` out of `total` fragments produced
/// by splitting a single note (`total > 1`): `Start` for the first,
/// `Stop` for the last, `Continue` for any in between.
fn tie_type_for_position(index: usize, total: usize) -> TieType {
    if index == 0 {
        TieType::Start
    } else if index == total - 1 {
        TieType::Stop
    } else {
        TieType::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_creation() {
        let part = Part::with_name("Violin I");
        assert_eq!(part.name(), Some("Violin I"));
        assert!(part.is_empty());
    }

    #[test]
    fn test_part_measures() {
        let mut part = Part::new();
        part.add_measure(Measure::new(1));
        part.add_measure(Measure::new(2));

        assert_eq!(part.num_measures(), 2);
        assert_eq!(part.measure(0).unwrap().number(), 1);
    }

    #[test]
    fn test_part_instrument() {
        let mut part = Part::with_name("Piano");
        part.set_instrument(Instrument::piano());

        assert_eq!(part.instrument().unwrap().midi_program(), 0);
    }

    #[test]
    fn test_instrument_creation() {
        let trumpet = Instrument::trumpet();
        assert_eq!(trumpet.name(), "Trumpet");
        assert_eq!(trumpet.transposition(), -2);
    }

    #[test]
    fn test_ensure_measures() {
        let mut part = Part::new();
        part.ensure_measures(5);

        assert_eq!(part.num_measures(), 5);
        assert_eq!(part.measure(4).unwrap().number(), 5);
    }

    #[test]
    fn test_measure_duration_context_search() {
        // Regression test: Measure::duration() silently defaults to 4/4
        // when no time signature is set locally, which is wrong for any
        // measure that inherits its meter from an earlier measure (the
        // normal case — most measures don't repeat the time signature).
        use crate::notation::TimeSignature;

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(3, 4));
        part.add_measure(m1);
        part.add_measure(Measure::new(2)); // no explicit time signature
        part.add_measure(Measure::new(3)); // no explicit time signature

        // Measure::duration() alone can't see measure 1's time signature.
        assert_eq!(part.measure(1).unwrap().duration(), Fraction::new(4, 1));

        // Part::measure_duration correctly finds the prevailing 3/4.
        assert_eq!(part.measure_duration(0), Fraction::new(3, 1));
        assert_eq!(part.measure_duration(1), Fraction::new(3, 1));
        assert_eq!(part.measure_duration(2), Fraction::new(3, 1));

        // And the aggregate Part::duration() reflects the fix too (would
        // have been 3+4+4=11 before the fix; correct is 3+3+3=9).
        assert_eq!(part.duration(), Fraction::new(9, 1));
    }

    #[test]
    fn test_key_signature_and_clef_context_search() {
        use crate::notation::{Clef, KeySignature};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_key_signature(KeySignature::g_major());
        m1.set_clef(Clef::bass());
        part.add_measure(m1);
        part.add_measure(Measure::new(2));

        assert_eq!(part.key_signature_at(1), Some(&KeySignature::g_major()));
        assert_eq!(part.clef_at(1), Some(&Clef::bass()));

        assert_eq!(part.time_signature_at(1), None);
        assert_eq!(part.key_signature_at(10), None); // out of range
    }

    #[test]
    fn test_renumber_measures_no_pickup() {
        let mut part = Part::new();
        part.add_measure(Measure::new(99));
        part.add_measure(Measure::new(100));
        part.add_measure(Measure::new(101));

        part.renumber_measures();

        assert_eq!(part.measure(0).unwrap().number(), 1);
        assert_eq!(part.measure(1).unwrap().number(), 2);
        assert_eq!(part.measure(2).unwrap().number(), 3);
    }

    #[test]
    fn test_insert_and_shift_pushes_later_elements_back() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::D, Some(4), None))),
        );
        part.add_measure(m1);

        // Insert a half note at offset 1: the D at offset 1 (and anything
        // after) must shift later by the half note's duration (2).
        part.insert_and_shift(
            0,
            Fraction::new(1, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::E, Some(4), None))),
        );

        let elements = part.measure(0).unwrap().elements().to_vec();
        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0].0, Fraction::new(0, 1)); // C unchanged
        assert_eq!(elements[1].0, Fraction::new(1, 1)); // E inserted here
        assert_eq!(elements[1].1.as_note().unwrap().pitch().name(), "E");
        assert_eq!(elements[2].0, Fraction::new(3, 1)); // D shifted from 1 to 3
        assert_eq!(elements[2].1.as_note().unwrap().pitch().name(), "D");
    }

    #[test]
    fn test_find_consecutive_notes_skips_rests_and_chords() {
        use crate::core::{Chord, Note, Pitch, Rest, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(Fraction::new(1, 1), MusicElement::Rest(Rest::quarter()));
        m1.insert(
            Fraction::new(2, 1),
            MusicElement::Chord(Chord::major_triad(Pitch::from_parts(
                Step::C,
                Some(4),
                None,
            ))),
        );
        part.add_measure(m1);

        let mut m2 = Measure::new(2);
        m2.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::E, Some(4), None))),
        );
        part.add_measure(m2);

        let notes = part.find_consecutive_notes();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].pitch().name(), "C");
        assert_eq!(notes[1].pitch().name(), "E");
    }

    #[test]
    fn test_melodic_intervals_between_consecutive_notes() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::E, Some(4), None))),
        );
        m1.insert(
            Fraction::new(2, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::D, Some(4), None))),
        );
        part.add_measure(m1);

        let intervals = part.melodic_intervals();
        assert_eq!(intervals.len(), 2);
        assert_eq!(intervals[0].semitones(), 4); // C to E: major third up
        assert_eq!(intervals[1].semitones(), -2); // E to D: major second down
    }

    #[test]
    fn test_transpose_shifts_note_and_chord_pitches_leaves_rests() {
        use crate::core::{Chord, Interval, Note, Pitch, Rest, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Chord(Chord::major_triad(Pitch::from_parts(
                Step::C,
                Some(4),
                None,
            ))),
        );
        m1.insert(Fraction::new(2, 1), MusicElement::Rest(Rest::quarter()));
        part.add_measure(m1);

        let transposed = part.transpose(&Interval::perfect_fifth());

        let elements = transposed.measure(0).unwrap().elements().to_vec();
        assert_eq!(elements[0].1.as_note().unwrap().pitch().name(), "G");
        assert_eq!(
            elements[1].1.as_chord().unwrap().root().unwrap().name(),
            "G"
        );
        assert!(elements[2].1.is_rest());

        // Original part is untouched.
        assert_eq!(
            part.measure(0).unwrap().elements()[0]
                .1
                .as_note()
                .unwrap()
                .pitch()
                .name(),
            "C"
        );
    }

    #[test]
    fn test_augment_or_diminish_scales_offsets_and_durations() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        part.add_measure(m1);

        let augmented = part.augment_or_diminish(Fraction::new(2, 1));
        let (offset, element) = &augmented.measure(0).unwrap().elements()[0];
        assert_eq!(*offset, Fraction::new(2, 1));
        assert_eq!(element.quarter_length(), Fraction::new(2, 1));

        let diminished = part.augment_or_diminish(Fraction::new(1, 2));
        let (offset, element) = &diminished.measure(0).unwrap().elements()[0];
        assert_eq!(*offset, Fraction::new(1, 2));
        assert_eq!(element.quarter_length(), Fraction::new(1, 2));
    }

    #[test]
    fn test_quantize_snaps_offsets_and_durations_to_grid() {
        use crate::core::{Duration, Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        // Slightly-off offset/duration (e.g. from a sloppy MIDI import).
        m1.insert(
            Fraction::new(11, 40), // ~0.275, should snap to 0.25 on a 16th grid
            MusicElement::Note(Note::new(
                Pitch::from_parts(Step::C, Some(4), None),
                Duration::from_quarter_length(Fraction::new(24, 100)), // ~0.24 -> 0.25
            )),
        );
        part.add_measure(m1);

        part.quantize(Fraction::new(1, 4)); // 16th-note grid

        let (offset, element) = &part.measure(0).unwrap().elements()[0];
        assert_eq!(*offset, Fraction::new(1, 4));
        assert_eq!(element.quarter_length(), Fraction::new(1, 4));
    }

    #[test]
    fn test_slice_by_quarter_lengths_splits_and_ties() {
        use crate::core::{Note, Pitch, Step, TieType};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::C, Some(4), None))), // 2 quarters
        );
        part.add_measure(m1);

        part.slice_by_quarter_lengths(Fraction::new(1, 2), true); // eighth-note slices

        let elements = part.measure(0).unwrap().elements().to_vec();
        assert_eq!(elements.len(), 4); // 2 quarters / 0.5 = 4 pieces
        assert_eq!(elements[0].0, Fraction::new(0, 1));
        assert_eq!(
            elements[0].1.as_note().unwrap().tie().unwrap().type_,
            TieType::Start
        );
        assert_eq!(
            elements[1].1.as_note().unwrap().tie().unwrap().type_,
            TieType::Continue
        );
        assert_eq!(elements[3].0, Fraction::new(3, 2));
        assert_eq!(
            elements[3].1.as_note().unwrap().tie().unwrap().type_,
            TieType::Stop
        );
    }

    #[test]
    fn test_slice_by_beat_splits_note_crossing_beat_boundary() {
        use crate::core::{Duration, Note, Pitch, Step, TieType};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4)); // beats at 0,1,2,3
                                                         // A note from offset 0.5 to 1.5 crosses the beat boundary at 1.
        m1.insert(
            Fraction::new(1, 2),
            MusicElement::Note(Note::new(
                Pitch::from_parts(Step::C, Some(4), None),
                Duration::quarter(),
            )),
        );
        part.add_measure(m1);

        part.slice_by_beat();

        let elements = part.measure(0).unwrap().elements().to_vec();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].0, Fraction::new(1, 2));
        assert_eq!(
            elements[0].1.as_note().unwrap().quarter_length(),
            Fraction::new(1, 2)
        );
        assert_eq!(
            elements[0].1.as_note().unwrap().tie().unwrap().type_,
            TieType::Start
        );
        assert_eq!(elements[1].0, Fraction::new(1, 1));
        assert_eq!(
            elements[1].1.as_note().unwrap().quarter_length(),
            Fraction::new(1, 2)
        );
        assert_eq!(
            elements[1].1.as_note().unwrap().tie().unwrap().type_,
            TieType::Stop
        );
    }

    #[test]
    fn test_slice_at_offsets_splits_at_arbitrary_points() {
        use crate::core::{Duration, Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::new(
                Pitch::from_parts(Step::C, Some(4), None),
                Duration::whole(), // 4 quarters
            )),
        );
        part.add_measure(m1);

        part.slice_at_offsets(&[Fraction::new(1, 1), Fraction::new(3, 1)]);

        let elements = part.measure(0).unwrap().elements().to_vec();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0].1.as_note().unwrap().quarter_length(),
            Fraction::new(1, 1)
        );
        assert_eq!(
            elements[1].1.as_note().unwrap().quarter_length(),
            Fraction::new(2, 1)
        );
        assert_eq!(
            elements[2].1.as_note().unwrap().quarter_length(),
            Fraction::new(1, 1)
        );
    }

    #[test]
    fn test_slice_by_greatest_divisor_finds_common_unit() {
        use crate::core::{Duration, Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        // A half note (2) and a dotted-quarter-ish 3/2... use a half
        // note and a quarter note: gcd(2, 1) = 1 quarter length.
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(2, 1),
            MusicElement::Note(Note::new(
                Pitch::from_parts(Step::D, Some(4), None),
                Duration::quarter(),
            )),
        );
        part.add_measure(m1);

        part.slice_by_greatest_divisor();

        // The half note should now be split into 2 quarter-length pieces;
        // the quarter note (already at the gcd unit) stays a single piece.
        let elements = part.measure(0).unwrap().elements().to_vec();
        assert_eq!(elements.len(), 3);
        for (_, element) in &elements {
            assert_eq!(element.quarter_length(), Fraction::new(1, 1));
        }
    }

    #[test]
    fn test_make_beams_groups_eighth_notes_within_beats() {
        use crate::core::{Duration, Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        for i in 0..8 {
            m1.insert(
                Fraction::new(i, 2),
                MusicElement::Note(Note::new(
                    Pitch::from_parts(Step::C, Some(4), None),
                    Duration::eighth(),
                )),
            );
        }
        part.add_measure(m1);

        let beams = part.make_beams(0);
        assert_eq!(beams.len(), 4); // 4 beats, 2 eighths each
        for beam in &beams {
            assert_eq!(beam.indices().len(), 2);
        }
    }

    #[test]
    fn test_make_accidentals_reflects_key_signature_context() {
        use crate::core::{Accidental, Note, Pitch, Step};
        use crate::notation::KeySignature;

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_key_signature(KeySignature::g_major()); // F# implied
                                                       // F# matches the key signature (shouldn't need display); F
                                                       // natural contradicts it (must be displayed).
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(
                Step::F,
                Some(4),
                Some(Accidental::Sharp),
            ))),
        );
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::F, Some(4), None))),
        );
        part.add_measure(m1);

        let displays = part.make_accidentals(0);
        assert_eq!(displays.len(), 2);
        assert_eq!(displays[0].display_status, Some(false));
        assert_eq!(displays[1].display_status, Some(true));
    }

    #[test]
    fn test_make_notation_applies_ties_and_returns_beams_and_accidentals() {
        use crate::core::Note;
        use crate::core::{Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(2, 4));
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::C, Some(4), None))),
        );
        part.add_measure(m1);
        part.add_measure(Measure::new(2));

        let info = part.make_notation();

        // make_ties should have split the overflowing half note.
        assert!(part.measure(0).unwrap().elements()[0]
            .1
            .as_note()
            .unwrap()
            .tie()
            .is_some());
        assert_eq!(info.len(), 2);
    }

    #[test]
    fn test_make_ties_splits_note_overflowing_measure_boundary() {
        use crate::core::{Note, Pitch, Step, TieType};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(2, 4)); // only 2 quarters
                                                         // A half note (2 quarters) at offset 1 overflows by 1 quarter.
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::C, Some(4), None))),
        );
        part.add_measure(m1);
        part.add_measure(Measure::new(2));

        part.make_ties();

        let m1_note = part.measure(0).unwrap().elements()[0].1.as_note().unwrap();
        assert_eq!(m1_note.quarter_length(), Fraction::new(1, 1));
        assert_eq!(m1_note.tie().unwrap().type_, TieType::Start);

        let m2_note = part.measure(1).unwrap().elements()[0].1.as_note().unwrap();
        assert_eq!(m2_note.quarter_length(), Fraction::new(1, 1));
        assert_eq!(m2_note.tie().unwrap().type_, TieType::Stop);
        assert_eq!(m2_note.pitch().name(), "C");
    }

    #[test]
    fn test_make_ties_creates_new_measures_when_needed() {
        use crate::core::{Duration, Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(1, 4)); // 1 quarter per bar
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::new(
                Pitch::from_parts(Step::C, Some(4), None),
                Duration::whole(), // 4 quarters, needs 4 measures total
            )),
        );
        part.add_measure(m1);

        part.make_ties();

        assert_eq!(part.num_measures(), 4);
        for i in 0..4 {
            let note = part.measure(i).unwrap().elements()[0].1.as_note().unwrap();
            assert_eq!(note.quarter_length(), Fraction::new(1, 1));
        }
    }

    #[test]
    fn test_strip_ties_merges_tied_run_back_into_one_note() {
        use crate::core::{Note, Pitch, Step, Tie};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(2, 4));
        let mut first = Note::quarter(Pitch::from_parts(Step::C, Some(4), None));
        first.set_tie(Some(Tie::start()));
        m1.insert(Fraction::new(1, 1), MusicElement::Note(first));
        part.add_measure(m1);

        let mut m2 = Measure::new(2);
        let mut second = Note::quarter(Pitch::from_parts(Step::C, Some(4), None));
        second.set_tie(Some(Tie::stop()));
        m2.insert(Fraction::new(0, 1), MusicElement::Note(second));
        part.add_measure(m2);

        part.strip_ties();

        let merged = part.measure(0).unwrap().elements()[0].1.as_note().unwrap();
        assert_eq!(merged.quarter_length(), Fraction::new(2, 1));
        assert!(merged.tie().is_none());
        // The follower note in measure 2 must be gone.
        assert!(part.measure(1).unwrap().is_empty());
    }

    #[test]
    fn test_make_ties_then_strip_ties_round_trip() {
        use crate::core::{Duration, Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(2, 4));
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::C, Some(4), None))),
        );
        part.add_measure(m1);
        part.add_measure(Measure::new(2));

        part.make_ties();
        part.strip_ties();

        let note = part.measure(0).unwrap().elements()[0].1.as_note().unwrap();
        assert_eq!(note.quarter_length(), Duration::half().quarter_length());
        assert!(note.tie().is_none());
        assert!(part.measure(1).unwrap().is_empty());
    }

    #[test]
    fn test_extend_ties_connects_adjacent_same_pitch_notes() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        // A different pitch immediately after should NOT be tied.
        m1.insert(
            Fraction::new(2, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::D, Some(4), None))),
        );
        part.add_measure(m1);

        part.extend_ties();

        let elements = part.measure(0).unwrap().elements().to_vec();
        assert_eq!(
            elements[0].1.as_note().unwrap().tie().unwrap().type_,
            crate::core::TieType::Start
        );
        assert_eq!(
            elements[1].1.as_note().unwrap().tie().unwrap().type_,
            crate::core::TieType::Stop
        );
        assert!(elements[2].1.as_note().unwrap().tie().is_none());
    }

    #[test]
    fn test_make_measures_partitions_flat_elements_by_bar_duration() {
        use crate::core::{Note, Pitch, Step};

        // 8 quarter notes in 4/4 should become exactly 2 measures.
        let elements: Vec<(Fraction, MusicElement)> = (0..8)
            .map(|i| {
                (
                    Fraction::new(i, 1),
                    MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
                )
            })
            .collect();

        let part = Part::make_measures(&elements, TimeSignature::new(4, 4));
        assert_eq!(part.num_measures(), 2);
        assert_eq!(part.measure(0).unwrap().len(), 4);
        assert_eq!(part.measure(1).unwrap().len(), 4);
        assert_eq!(
            part.measure(0).unwrap().time_signature(),
            Some(&TimeSignature::new(4, 4))
        );

        // The 5th note (absolute offset 4) must land at local offset 0
        // in measure 2, not absolute offset 4 (which measure 2's own
        // Stream has no notion of).
        assert_eq!(
            part.measure(1).unwrap().elements()[0].0,
            Fraction::new(0, 1)
        );
    }

    #[test]
    fn test_make_measures_empty_input_yields_one_empty_measure() {
        let part = Part::make_measures(&[], TimeSignature::new(3, 4));
        assert_eq!(part.num_measures(), 1);
        assert!(part.measure(0).unwrap().is_empty());
    }

    #[test]
    fn test_get_time_signatures_only_explicit_changes() {
        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        part.add_measure(m1);
        part.add_measure(Measure::new(2)); // inherits 4/4, not explicit
        let mut m3 = Measure::new(3);
        m3.set_time_signature(TimeSignature::new(3, 4));
        part.add_measure(m3);

        assert_eq!(
            part.get_time_signatures(),
            vec![
                (Fraction::new(0, 1), TimeSignature::new(4, 4)),
                (Fraction::new(8, 1), TimeSignature::new(3, 4)),
            ]
        );
    }

    #[test]
    fn test_measure_offset_map() {
        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(3, 4));
        part.add_measure(m1);
        part.add_measure(Measure::new(2)); // inherits 3/4
        part.add_measure(Measure::new(3)); // inherits 3/4

        assert_eq!(
            part.measure_offset_map(),
            vec![
                (Fraction::new(0, 1), 1),
                (Fraction::new(3, 1), 2),
                (Fraction::new(6, 1), 3),
            ]
        );
    }

    #[test]
    fn test_flatten_and_recurse_use_absolute_offsets() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        part.add_measure(m1);

        let mut m2 = Measure::new(2);
        m2.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::D,
            Some(4),
            None,
        ))));
        part.add_measure(m2);

        let flat = part.flatten();
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].0, Fraction::new(0, 1));
        // The second measure's note starts at absolute offset 4 (after
        // the first measure's full 4/4 bar), not 0 (its own
        // within-measure offset) — this is what `flatten` must get right
        // that a naive per-measure concatenation without an absolute-
        // offset pass would not.
        assert_eq!(flat[1].0, Fraction::new(4, 1));

        let recursed = part.recurse();
        assert_eq!(recursed.len(), 2);
        assert_eq!(recursed[1].measure_number, 2);
        assert_eq!(recursed[1].offset_in_measure, Fraction::new(0, 1));
        assert_eq!(recursed[1].absolute_offset, Fraction::new(4, 1));
    }

    #[test]
    fn test_get_elements_by_offset_and_at_or_before() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(2, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::E, Some(4), None))),
        );
        part.add_measure(m1);

        let in_range = part.get_elements_by_offset(Fraction::new(1, 1), Fraction::new(3, 1));
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].0, Fraction::new(2, 1));

        let at_or_before = part.get_element_at_or_before(Fraction::new(3, 1)).unwrap();
        assert_eq!(at_or_before.0, Fraction::new(2, 1));

        assert!(part
            .get_element_at_or_before(Fraction::new(-1, 1))
            .is_none());
    }

    #[test]
    fn test_structural_checks_and_template() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        part.add_measure(m1);

        assert!(part.has_measures());
        assert!(!part.has_voices());
        assert!(part.is_well_formed_notation());

        let templated = part.template();
        assert_eq!(templated.num_measures(), 1);
        assert!(templated.measure(0).unwrap().is_empty());
        assert_eq!(
            templated.measure(0).unwrap().time_signature(),
            Some(&TimeSignature::new(4, 4))
        );
    }

    #[test]
    fn test_is_well_formed_notation_detects_overfull_measure() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(2, 4)); // only 2 quarters
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::D,
            Some(4),
            None,
        ))));
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::E,
            Some(4),
            None,
        )))); // overflows the 2/4 bar
        part.add_measure(m1);

        assert!(!part.is_well_formed_notation());
    }

    #[test]
    fn test_explode_splits_chords_into_separate_parts_high_to_low() {
        use crate::core::{Chord, Duration, Note, Pitch, Step};

        let mut part = Part::with_name("Choir");
        let mut m1 = Measure::new(1);
        m1.set_time_signature(TimeSignature::new(4, 4));
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Chord(Chord::from_pitches(
                vec![
                    Pitch::from_parts(Step::C, Some(4), None),
                    Pitch::from_parts(Step::E, Some(4), None),
                    Pitch::from_parts(Step::G, Some(4), None),
                ],
                Duration::quarter(),
            )),
        );
        // A plain note (not a chord) should land only in part 0.
        m1.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::D, Some(4), None))),
        );
        part.add_measure(m1);

        let exploded = part.explode();
        assert_eq!(exploded.len(), 3);

        let m0 = exploded[0].measure(0).unwrap();
        let m1_out = exploded[1].measure(0).unwrap();
        let m2_out = exploded[2].measure(0).unwrap();

        // Part 0 gets the highest pitch (G) at offset 0, plus the lone note at offset 1.
        assert_eq!(m0.elements()[0].1.as_note().unwrap().pitch().name(), "G");
        assert_eq!(m0.elements().len(), 2);
        assert_eq!(
            m1_out.elements()[0].1.as_note().unwrap().pitch().name(),
            "E"
        );
        assert_eq!(
            m2_out.elements()[0].1.as_note().unwrap().pitch().name(),
            "C"
        );
    }

    #[test]
    fn test_voices_to_parts_and_back() {
        use crate::core::{Note, Pitch, Step};

        let mut voice1 = Voice::new(1);
        voice1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        let mut voice2 = Voice::new(2);
        voice2.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::E, Some(4), None))),
        );

        let parts = voices_to_parts(&[voice1, voice2]);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name(), Some("Voice 1"));
        assert_eq!(
            parts[1].measure(0).unwrap().elements()[0]
                .1
                .as_note()
                .unwrap()
                .pitch()
                .name(),
            "E"
        );

        let voices_back = parts_to_voices(&[&parts[0], &parts[1]]);
        assert_eq!(voices_back.len(), 2);
        assert_eq!(
            voices_back[0].elements()[0]
                .1
                .as_note()
                .unwrap()
                .pitch()
                .name(),
            "C"
        );
        assert_eq!(
            voices_back[1].elements()[0]
                .1
                .as_note()
                .unwrap()
                .pitch()
                .name(),
            "E"
        );
    }

    #[test]
    fn test_renumber_measures_with_pickup() {
        // Regression test: the previous implementation had a dead
        // conditional (`- if has_pickup {0} else {0}`) that always
        // subtracted zero, so this path was never actually exercised
        // differently from the no-pickup case.
        let mut part = Part::new();
        part.add_measure(Measure::pickup());
        part.add_measure(Measure::new(50));
        part.add_measure(Measure::new(51));

        part.renumber_measures();

        assert_eq!(part.measure(0).unwrap().number(), 0);
        assert_eq!(part.measure(1).unwrap().number(), 1);
        assert_eq!(part.measure(2).unwrap().number(), 2);
    }

    #[test]
    fn test_expand_repeats_duplicates_marked_section() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.set_repeat_start(true);
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        part.add_measure(m1);

        let mut m2 = Measure::new(2);
        m2.set_repeat_end(true);
        m2.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::D,
            Some(4),
            None,
        ))));
        part.add_measure(m2);

        let mut m3 = Measure::new(3);
        m3.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::E,
            Some(4),
            None,
        ))));
        part.add_measure(m3);

        let expanded = part.expand_repeats();

        // C-D repeated once (C,D,C,D), then E: 5 measures total.
        assert_eq!(expanded.num_measures(), 5);
        let pitches: Vec<String> = (0..5)
            .map(|i| {
                expanded.measure(i).unwrap().elements()[0]
                    .1
                    .as_note()
                    .unwrap()
                    .pitch()
                    .name()
            })
            .collect();
        assert_eq!(pitches, vec!["C", "D", "C", "D", "E"]);
        // Measures are renumbered sequentially afterward.
        for i in 0..5 {
            assert_eq!(expanded.measure(i).unwrap().number(), i as u32 + 1);
        }
    }

    #[test]
    fn test_expand_repeats_no_repeats_is_identity() {
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        part.add_measure(m1);

        let expanded = part.expand_repeats();
        assert_eq!(expanded.num_measures(), 1);
    }

    #[test]
    fn test_analyze_dispatches_by_method_name() {
        use crate::analysis::PartAnalysisResult;
        use crate::core::{Note, Pitch, Step};

        let mut part = Part::new();
        let mut m1 = Measure::new(1);
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        m1.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::G,
            Some(5),
            None,
        ))));
        part.add_measure(m1);

        match part.analyze("ambitus").unwrap() {
            PartAnalysisResult::Ambitus(Some(interval)) => assert!(interval.semitones() > 0),
            other => panic!("expected Some(Ambitus), got {other:?}"),
        }

        assert!(matches!(
            part.analyze("key"),
            Some(PartAnalysisResult::Key(_))
        ));
        assert!(matches!(
            part.analyze("melodicIntervalDiversity"),
            Some(PartAnalysisResult::MelodicIntervalDiversity(_))
        ));
        assert!(part.analyze("nonsense").is_none());
    }
}
