//! MIDI track representation
//!
//! A track contains a sequence of MIDI events.

use std::fmt;

use super::event::{MidiEvent, NoteSortOrder, compare_events};
use super::message::{MetaEvent, MidiMessage};

/// A MIDI track containing events
#[derive(Debug, Clone, Default)]
pub struct MidiTrack {
    /// Events in this track
    events: Vec<MidiEvent>,
    /// Track name
    name: Option<String>,
    /// Whether events are in absolute or delta time
    absolute_time: bool,
    /// Whether the track is sorted
    sorted: bool,
}

impl MidiTrack {
    /// Create a new empty track
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            name: None,
            absolute_time: true,
            sorted: true,
        }
    }

    /// Create a track with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            events: Vec::new(),
            name: Some(name.into()),
            absolute_time: true,
            sorted: true,
        }
    }

    /// Get the track name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the track name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get all events
    pub fn events(&self) -> &[MidiEvent] {
        &self.events
    }

    /// Get mutable events
    pub fn events_mut(&mut self) -> &mut Vec<MidiEvent> {
        self.sorted = false;
        &mut self.events
    }

    /// Get the number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Reserve capacity for at least `additional` more events.
    pub fn reserve(&mut self, additional: usize) {
        self.events.reserve(additional);
    }

    /// Check if the track is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Add an event to the track
    pub fn add_event(&mut self, event: MidiEvent) {
        let tick = event.tick();
        self.events.push(event);

        // Check if still sorted
        if self.sorted && self.events.len() > 1 {
            let prev_tick = self.events[self.events.len() - 2].tick();
            if tick < prev_tick {
                self.sorted = false;
            }
        }
    }

    /// Insert an event at a specific position
    pub fn insert_event(&mut self, index: usize, event: MidiEvent) {
        self.events.insert(index, event);
        self.sorted = false;
    }

    /// Remove an event at a specific position
    pub fn remove_event(&mut self, index: usize) -> Option<MidiEvent> {
        if index < self.events.len() {
            Some(self.events.remove(index))
        } else {
            None
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.sorted = true;
    }

    /// Sort events by tick time (note-ons before note-offs at the same tick,
    /// matching upstream midifile's default). Rust's `Vec::sort` is stable,
    /// so events that compare equal keep their current relative order
    /// without needing an internally-assigned sequence number; explicit
    /// sequence numbers (see `mark_sequence`) are only used when the caller
    /// has opted in.
    pub fn sort(&mut self) {
        self.sort_with_order(NoteSortOrder::NoteOnsBeforeOffs);
    }

    /// Sort events by tick time with an explicit note-on/off tie-break order.
    pub fn sort_with_order(&mut self, order: NoteSortOrder) {
        if !self.sorted || order != NoteSortOrder::NoteOnsBeforeOffs {
            self.events.sort_by(|a, b| compare_events(a, b, order));
            self.sorted = order == NoteSortOrder::NoteOnsBeforeOffs;
        }
    }

    /// Check if events are sorted
    pub fn is_sorted(&self) -> bool {
        self.sorted
    }

    /// Assign sequential, 1-based sequence numbers to events in their
    /// current order. Call right after reading a file (or whenever the
    /// current in-memory order should be preserved) so that a later `sort()`
    /// treats same-tick events as already correctly ordered rather than
    /// falling back to type-based priority. Mirrors upstream midifile's
    /// `markSequence()`.
    pub fn mark_sequence(&mut self) {
        for (i, event) in self.events.iter_mut().enumerate() {
            event.set_seq(i as u32 + 1);
        }
    }

    /// Clear sequence numbers assigned by `mark_sequence`, reverting to
    /// pure type-based tie-breaking on the next sort. Mirrors upstream
    /// midifile's `clearSequence()`.
    pub fn clear_sequence(&mut self) {
        for event in self.events.iter_mut() {
            event.set_seq(0);
        }
    }

    /// Get the last tick time
    pub fn last_tick(&self) -> u64 {
        self.events.last().map(|e| e.tick()).unwrap_or(0)
    }

    /// Check if times are absolute
    pub fn is_absolute_time(&self) -> bool {
        self.absolute_time
    }

    /// Convert delta times to absolute times
    pub fn make_absolute_times(&mut self) {
        if !self.absolute_time {
            let mut current_tick: u64 = 0;
            for event in &mut self.events {
                current_tick += event.tick();
                event.set_tick(current_tick);
            }
            self.absolute_time = true;
        }
    }

    /// Convert absolute times to delta times
    pub fn make_delta_times(&mut self) {
        if self.absolute_time {
            self.sort(); // Must be sorted first
            let mut prev_tick: u64 = 0;
            for event in &mut self.events {
                let abs_tick = event.tick();
                event.set_tick(abs_tick - prev_tick);
                prev_tick = abs_tick;
            }
            self.absolute_time = false;
        }
    }

    /// Link note on/off events
    pub fn link_note_events(&mut self) {
        self.sort();

        // Track active notes by (channel, key)
        let mut active_notes: Vec<Vec<usize>> = vec![vec![]; 16 * 128];

        for i in 0..self.events.len() {
            let event = &self.events[i];

            if let Some(channel) = event.channel()
                && let Some(key) = event.key()
            {
                let idx = (channel as usize) * 128 + (key as usize);

                if event.is_note_on() {
                    active_notes[idx].push(i);
                } else if event.is_note_off() && !active_notes[idx].is_empty() {
                    // FIFO linking
                    let on_idx = active_notes[idx].remove(0);
                    self.events[on_idx].set_linked_event(Some(i));
                    self.events[i].set_linked_event(Some(on_idx));
                }
            }
        }
    }

    /// Link note on/off events using LIFO (last-in-first-out) pairing
    /// instead of the FIFO pairing `link_note_events` uses. This correctly
    /// pairs overlapping/nested same-key note-on events (e.g. legato
    /// re-triggers or ornamentation), where FIFO pairing would associate a
    /// note-off with the wrong note-on and produce the wrong duration.
    pub fn link_note_pairs_lifo(&mut self) {
        self.sort();

        let mut active_notes: Vec<Vec<usize>> = vec![vec![]; 16 * 128];

        for i in 0..self.events.len() {
            let event = &self.events[i];

            if let Some(channel) = event.channel()
                && let Some(key) = event.key()
            {
                let idx = (channel as usize) * 128 + (key as usize);

                if event.is_note_on() {
                    active_notes[idx].push(i);
                } else if event.is_note_off()
                    && let Some(on_idx) = active_notes[idx].pop()
                {
                    self.events[on_idx].set_linked_event(Some(i));
                    self.events[i].set_linked_event(Some(on_idx));
                }
            }
        }
    }

    /// Link on/off transitions for pedal-style controllers (sustain,
    /// portamento, sostenuto, soft pedal, legato, hold 2, and the four
    /// general-purpose buttons), the same way `link_note_events` links note
    /// on/off pairs. A controller value >= 64 counts as "on"; < 64 as "off".
    pub fn link_controller_pairs(&mut self) {
        const PEDAL_CONTROLLERS: [u8; 10] = [64, 65, 66, 67, 68, 69, 80, 81, 82, 83];

        self.sort();
        let mut pending: std::collections::HashMap<(u8, u8), usize> =
            std::collections::HashMap::new();

        for i in 0..self.events.len() {
            let (channel, controller, value) = {
                let event = &self.events[i];
                match (
                    event.channel(),
                    event.message().get_controller_number(),
                    event.message().get_controller_value(),
                ) {
                    (Some(ch), Some(cc), Some(val)) if PEDAL_CONTROLLERS.contains(&cc) => {
                        (ch, cc, val)
                    }
                    _ => continue,
                }
            };

            let key = (channel, controller);
            if value >= 64 {
                pending.insert(key, i);
            } else if let Some(on_idx) = pending.remove(&key) {
                self.events[on_idx].set_linked_event(Some(i));
                self.events[i].set_linked_event(Some(on_idx));
            }
        }
    }

    /// Unlink all note events
    pub fn unlink_note_events(&mut self) {
        for event in &mut self.events {
            event.set_linked_event(None);
        }
    }

    /// Add a note (creates note on and note off events)
    pub fn add_note(&mut self, start_tick: u64, duration: u64, channel: u8, key: u8, velocity: u8) {
        let on_idx = self.events.len();
        let mut on_event = MidiEvent::note_on(start_tick, channel, key, velocity);
        let mut off_event = MidiEvent::note_off(start_tick + duration, channel, key, 0);

        on_event.set_linked_event(Some(on_idx + 1));
        off_event.set_linked_event(Some(on_idx));

        self.events.push(on_event);
        self.events.push(off_event);
        self.sorted = false;
    }

    /// Add a control change event
    pub fn add_control_change(&mut self, tick: u64, channel: u8, controller: u8, value: u8) {
        self.add_event(MidiEvent::control_change(tick, channel, controller, value));
    }

    /// Add a program change event
    pub fn add_program_change(&mut self, tick: u64, channel: u8, program: u8) {
        self.add_event(MidiEvent::program_change(tick, channel, program));
    }

    /// Add a tempo event
    pub fn add_tempo(&mut self, tick: u64, bpm: f64) {
        let meta = MetaEvent::tempo_from_bpm(bpm);
        self.add_event(MidiEvent::new(tick, MidiMessage::Meta(meta)));
    }

    /// Add a time signature event
    pub fn add_time_signature(&mut self, tick: u64, numerator: u8, denominator: u8) {
        let meta = MetaEvent::time_signature(numerator, denominator);
        self.add_event(MidiEvent::new(tick, MidiMessage::Meta(meta)));
    }

    /// Add a key signature event
    pub fn add_key_signature(&mut self, tick: u64, sharps: i8, minor: bool) {
        let meta = MetaEvent::key_signature(sharps, minor);
        self.add_event(MidiEvent::new(tick, MidiMessage::Meta(meta)));
    }

    /// Add a compound-meter time signature event (e.g. 6/8), using the
    /// upstream-matching default of 36 clocks per metronome click.
    pub fn add_compound_time_signature(&mut self, tick: u64, numerator: u8, denominator: u8) {
        let meta = MetaEvent::compound_time_signature(numerator, denominator);
        self.add_event(MidiEvent::new(tick, MidiMessage::Meta(meta)));
    }

    /// Add a pitch bend event
    pub fn add_pitch_bend(&mut self, tick: u64, channel: u8, value: u16) {
        self.add_event(MidiEvent::new(
            tick,
            MidiMessage::pitch_bend(channel, value),
        ));
    }

    /// Add a controller (control change) event
    pub fn add_controller(&mut self, tick: u64, channel: u8, controller: u8, value: u8) {
        self.add_control_change(tick, channel, controller, value);
    }

    /// Add a sustain pedal event with an explicit value
    pub fn add_sustain(&mut self, tick: u64, channel: u8, value: u8) {
        self.add_event(MidiEvent::new(
            tick,
            MidiMessage::make_sustain(channel, value),
        ));
    }

    /// Add a sustain pedal on event
    pub fn add_sustain_on(&mut self, tick: u64, channel: u8) {
        self.add_sustain(tick, channel, 127);
    }

    /// Add a sustain pedal off event
    pub fn add_sustain_off(&mut self, tick: u64, channel: u8) {
        self.add_sustain(tick, channel, 0);
    }

    /// Add a sustain pedal event. Alias of `add_sustain`.
    pub fn add_pedal(&mut self, tick: u64, channel: u8, value: u8) {
        self.add_sustain(tick, channel, value);
    }

    /// Add a sustain pedal on event. Alias of `add_sustain_on`.
    pub fn add_pedal_on(&mut self, tick: u64, channel: u8) {
        self.add_sustain_on(tick, channel);
    }

    /// Add a sustain pedal off event. Alias of `add_sustain_off`.
    pub fn add_pedal_off(&mut self, tick: u64, channel: u8) {
        self.add_sustain_off(tick, channel);
    }

    /// Add a patch (program) change event. Alias of `add_program_change`.
    pub fn add_patch_change(&mut self, tick: u64, channel: u8, program: u8) {
        self.add_program_change(tick, channel, program);
    }

    /// Add a timbre change event. Alias of `add_program_change`.
    pub fn add_timbre(&mut self, tick: u64, channel: u8, program: u8) {
        self.add_program_change(tick, channel, program);
    }

    /// Add a generic meta event
    pub fn add_meta_event(&mut self, tick: u64, meta: MetaEvent) {
        self.add_event(MidiEvent::new(tick, MidiMessage::Meta(meta)));
    }

    /// Add a text meta event
    pub fn add_text(&mut self, tick: u64, text: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::Text(text.into()));
    }

    /// Add a copyright meta event
    pub fn add_copyright(&mut self, tick: u64, text: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::Copyright(text.into()));
    }

    /// Add a track name meta event
    pub fn add_track_name(&mut self, tick: u64, name: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::TrackName(name.into()));
    }

    /// Add an instrument name meta event
    pub fn add_instrument_name(&mut self, tick: u64, name: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::InstrumentName(name.into()));
    }

    /// Add a lyric meta event
    pub fn add_lyric(&mut self, tick: u64, text: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::Lyric(text.into()));
    }

    /// Add a marker meta event
    pub fn add_marker(&mut self, tick: u64, text: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::Marker(text.into()));
    }

    /// Add a cue point meta event
    pub fn add_cue(&mut self, tick: u64, text: impl Into<String>) {
        self.add_meta_event(tick, MetaEvent::CuePoint(text.into()));
    }

    /// Write the RPN (Registered Parameter Number) sequence to set the
    /// pitch-bend range on a channel, in semitones (+ optional cents via the
    /// LSB data-entry value). This is the standard RPN 0 (pitch bend range)
    /// sequence: select RPN 0 via CC 101/100, then set the value via CC 6/38,
    /// then deselect the RPN (CC 101/100 = 127/127) so subsequent data-entry
    /// messages don't accidentally alter it.
    pub fn add_pitch_bend_range(&mut self, tick: u64, channel: u8, semitones: u8, cents: u8) {
        self.add_controller(tick, channel, 101, 0); // RPN MSB = 0
        self.add_controller(tick, channel, 100, 0); // RPN LSB = 0 (pitch bend range)
        self.add_controller(tick, channel, 6, semitones.min(127)); // Data entry MSB
        self.add_controller(tick, channel, 38, cents.min(127)); // Data entry LSB
        self.add_controller(tick, channel, 101, 127); // Deselect RPN
        self.add_controller(tick, channel, 100, 127);
    }

    /// Add an end of track marker
    pub fn add_end_of_track(&mut self) {
        let last_tick = self.last_tick();
        self.add_event(MidiEvent::new(
            last_tick,
            MidiMessage::Meta(MetaEvent::EndOfTrack),
        ));
    }

    /// Ensure the track has an end of track marker
    pub fn ensure_end_of_track(&mut self) {
        let has_eot = self
            .events
            .iter()
            .any(|e| matches!(e.message(), MidiMessage::Meta(MetaEvent::EndOfTrack)));

        if !has_eot {
            self.add_end_of_track();
        }
    }

    /// Get all note events (note on with velocity > 0)
    pub fn note_events(&self) -> impl Iterator<Item = &MidiEvent> {
        self.events.iter().filter(|e| e.is_note_on())
    }

    /// Get events for a specific channel
    pub fn channel_events(&self, channel: u8) -> impl Iterator<Item = &MidiEvent> {
        self.events
            .iter()
            .filter(move |e| e.channel() == Some(channel))
    }

    /// Get meta events
    pub fn meta_events(&self) -> impl Iterator<Item = &MidiEvent> {
        self.events.iter().filter(|e| e.is_meta())
    }

    /// Get tempo events
    pub fn tempo_events(&self) -> impl Iterator<Item = &MidiEvent> {
        self.events
            .iter()
            .filter(|e| matches!(e.message(), MidiMessage::Meta(MetaEvent::Tempo(_))))
    }

    /// Extract track to a specific channel (creates new track with only that channel)
    pub fn extract_channel(&self, channel: u8) -> MidiTrack {
        let mut track = MidiTrack::new();
        for event in &self.events {
            // Include meta events and events for this channel
            if event.is_meta() || event.channel() == Some(channel) {
                // linked_event indices refer to positions in *this* track's
                // event vector; they would silently point at the wrong (or
                // out-of-bounds) event once copied into the new, differently
                // indexed vector, so clear them here. Call `link_note_events()`
                // on the result if linked note durations are needed.
                let mut event = event.clone();
                event.unlink_event();
                track.add_event(event);
            }
        }
        track.sort();
        track
    }

    /// Merge another track into this one. As with `extract_channel`, the
    /// merged-in events' `linked_event` indices are cleared since they would
    /// otherwise point at the wrong events in the combined vector; call
    /// `link_note_events()` afterward if linked durations are needed.
    pub fn merge(&mut self, other: &MidiTrack) {
        for event in &other.events {
            let mut event = event.clone();
            event.unlink_event();
            self.add_event(event);
        }
        self.sorted = false;
    }
}

impl fmt::Display for MidiTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("Unnamed");
        write!(f, "Track '{}' ({} events)", name, self.events.len())
    }
}

impl FromIterator<MidiEvent> for MidiTrack {
    fn from_iter<T: IntoIterator<Item = MidiEvent>>(iter: T) -> Self {
        let mut track = MidiTrack::new();
        for event in iter {
            track.add_event(event);
        }
        track
    }
}

impl IntoIterator for MidiTrack {
    type Item = MidiEvent;
    type IntoIter = std::vec::IntoIter<MidiEvent>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl<'a> IntoIterator for &'a MidiTrack {
    type Item = &'a MidiEvent;
    type IntoIter = std::slice::Iter<'a, MidiEvent>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_bend_range_rpn_sequence() {
        let mut track = MidiTrack::new();
        track.add_pitch_bend_range(0, 2, 12, 0);

        let ccs: Vec<(u8, u8)> = track
            .events()
            .iter()
            .filter_map(|e| match e.message() {
                MidiMessage::ControlChange {
                    controller, value, ..
                } => Some((*controller, *value)),
                _ => None,
            })
            .collect();

        assert_eq!(
            ccs,
            vec![(101, 0), (100, 0), (6, 12), (38, 0), (101, 127), (100, 127)]
        );
    }

    #[test]
    fn test_meta_convenience_adders() {
        let mut track = MidiTrack::new();
        track.add_text(0, "hello");
        track.add_lyric(10, "la");
        track.add_marker(20, "verse 1");
        track.add_track_name(0, "Piano");

        assert!(
            matches!(track.events()[0].message(), MidiMessage::Meta(MetaEvent::Text(s)) if s == "hello")
        );
        assert!(
            matches!(track.events()[1].message(), MidiMessage::Meta(MetaEvent::Lyric(s)) if s == "la")
        );
        assert!(
            matches!(track.events()[2].message(), MidiMessage::Meta(MetaEvent::Marker(s)) if s == "verse 1")
        );
        assert!(
            matches!(track.events()[3].message(), MidiMessage::Meta(MetaEvent::TrackName(s)) if s == "Piano")
        );
    }

    #[test]
    fn test_sustain_and_compound_time_signature_adders() {
        let mut track = MidiTrack::new();
        track.add_sustain_on(0, 0);
        track.add_sustain_off(480, 0);
        track.add_compound_time_signature(0, 6, 8);

        assert!(track.events()[0].message().is_sustain_on());
        assert!(track.events()[1].message().is_sustain_off());
        assert!(matches!(
            track.events()[2].message(),
            MidiMessage::Meta(MetaEvent::TimeSignature {
                numerator: 6,
                clocks_per_click: 36,
                ..
            })
        ));
    }

    #[test]
    fn test_link_note_pairs_lifo_vs_fifo() {
        // Two overlapping note-ons on the same key (e.g. a legato re-trigger)
        // followed by two note-offs. FIFO pairing would link the first
        // note-on to the first note-off (duration 100), which is wrong for
        // nested/overlapping notes; LIFO pairing links the most recent
        // note-on to the next note-off (duration 50), matching the nesting.
        let mut track = MidiTrack::new();
        track.add_event(MidiEvent::note_on(0, 0, 60, 100));
        track.add_event(MidiEvent::note_on(50, 0, 60, 100));
        track.add_event(MidiEvent::note_off(100, 0, 60, 0));
        track.add_event(MidiEvent::note_off(150, 0, 60, 0));

        track.link_note_pairs_lifo();
        track.sort();

        let events = track.events();
        // Second note-on (tick 50) pairs with the first note-off (tick 100).
        let second_on = &events[1];
        assert_eq!(second_on.tick(), 50);
        let linked = second_on.linked_event().unwrap();
        assert_eq!(events[linked].tick(), 100);
    }

    #[test]
    fn test_link_controller_pairs_sustain() {
        let mut track = MidiTrack::new();
        track.add_sustain_on(0, 0);
        track.add_sustain_off(480, 0);

        track.link_controller_pairs();

        let on = &track.events()[0];
        assert!(on.is_linked());
        let off_idx = on.linked_event().unwrap();
        assert_eq!(track.events()[off_idx].tick(), 480);
    }

    #[test]
    fn test_track_creation() {
        let track = MidiTrack::with_name("Piano");
        assert_eq!(track.name(), Some("Piano"));
        assert!(track.is_empty());
    }

    #[test]
    fn test_track_add_events() {
        let mut track = MidiTrack::new();
        track.add_event(MidiEvent::note_on(0, 0, 60, 100));
        track.add_event(MidiEvent::note_off(480, 0, 60, 0));

        assert_eq!(track.len(), 2);
        assert_eq!(track.last_tick(), 480);
    }

    #[test]
    fn test_track_add_note() {
        let mut track = MidiTrack::new();
        track.add_note(0, 480, 0, 60, 100);

        assert_eq!(track.len(), 2);
        assert!(track.events()[0].is_note_on());
        assert!(track.events()[1].is_note_off());
        assert!(track.events()[0].is_linked());
    }

    #[test]
    fn test_track_sorting() {
        let mut track = MidiTrack::new();
        track.add_event(MidiEvent::note_on(480, 0, 62, 100));
        track.add_event(MidiEvent::note_on(0, 0, 60, 100));
        track.add_event(MidiEvent::note_on(240, 0, 61, 100));

        assert!(!track.is_sorted());
        track.sort();
        assert!(track.is_sorted());

        assert_eq!(track.events()[0].tick(), 0);
        assert_eq!(track.events()[1].tick(), 240);
        assert_eq!(track.events()[2].tick(), 480);
    }

    #[test]
    fn test_track_delta_absolute() {
        let mut track = MidiTrack::new();
        track.add_event(MidiEvent::note_on(0, 0, 60, 100));
        track.add_event(MidiEvent::note_off(480, 0, 60, 0));
        track.add_event(MidiEvent::note_on(480, 0, 62, 100));
        track.add_event(MidiEvent::note_off(960, 0, 62, 0));

        // Convert to delta
        track.make_delta_times();
        assert!(!track.is_absolute_time());
        assert_eq!(track.events()[0].tick(), 0);
        assert_eq!(track.events()[1].tick(), 480);
        assert_eq!(track.events()[2].tick(), 0);
        assert_eq!(track.events()[3].tick(), 480);

        // Convert back to absolute
        track.make_absolute_times();
        assert!(track.is_absolute_time());
        assert_eq!(track.events()[0].tick(), 0);
        assert_eq!(track.events()[1].tick(), 480);
        assert_eq!(track.events()[2].tick(), 480);
        assert_eq!(track.events()[3].tick(), 960);
    }

    #[test]
    fn test_track_link_notes() {
        let mut track = MidiTrack::new();
        track.add_event(MidiEvent::note_on(0, 0, 60, 100));
        track.add_event(MidiEvent::note_off(480, 0, 60, 0));

        track.link_note_events();

        assert!(track.events()[0].is_linked());
        assert!(track.events()[1].is_linked());
        assert_eq!(track.events()[0].linked_event(), Some(1));
        assert_eq!(track.events()[1].linked_event(), Some(0));
    }

    #[test]
    fn test_track_tempo() {
        let mut track = MidiTrack::new();
        track.add_tempo(0, 120.0);

        let tempo_events: Vec<_> = track.tempo_events().collect();
        assert_eq!(tempo_events.len(), 1);
    }
}
