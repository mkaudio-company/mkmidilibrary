//! MIDI track representation
//!
//! A track contains a sequence of MIDI events.

use std::fmt;

use super::event::MidiEvent;
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

    /// Sort events by tick time
    pub fn sort(&mut self) {
        if !self.sorted {
            // Assign sequence numbers for stable sort
            for (i, event) in self.events.iter_mut().enumerate() {
                event.set_seq(i as u32);
            }
            self.events.sort();
            self.sorted = true;
        }
    }

    /// Check if events are sorted
    pub fn is_sorted(&self) -> bool {
        self.sorted
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

            if let Some(channel) = event.channel() {
                if let Some(key) = event.key() {
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
    }

    /// Unlink all note events
    pub fn unlink_note_events(&mut self) {
        for event in &mut self.events {
            event.set_linked_event(None);
        }
    }

    /// Add a note (creates note on and note off events)
    pub fn add_note(
        &mut self,
        start_tick: u64,
        duration: u64,
        channel: u8,
        key: u8,
        velocity: u8,
    ) {
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
        let has_eot = self.events.iter().any(|e| {
            matches!(
                e.message(),
                MidiMessage::Meta(MetaEvent::EndOfTrack)
            )
        });

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
        self.events.iter().filter(move |e| e.channel() == Some(channel))
    }

    /// Get meta events
    pub fn meta_events(&self) -> impl Iterator<Item = &MidiEvent> {
        self.events.iter().filter(|e| e.is_meta())
    }

    /// Get tempo events
    pub fn tempo_events(&self) -> impl Iterator<Item = &MidiEvent> {
        self.events.iter().filter(|e| {
            matches!(
                e.message(),
                MidiMessage::Meta(MetaEvent::Tempo(_))
            )
        })
    }

    /// Extract track to a specific channel (creates new track with only that channel)
    pub fn extract_channel(&self, channel: u8) -> MidiTrack {
        let mut track = MidiTrack::new();
        for event in &self.events {
            // Include meta events and events for this channel
            if event.is_meta() || event.channel() == Some(channel) {
                track.add_event(event.clone());
            }
        }
        track.sort();
        track
    }

    /// Merge another track into this one
    pub fn merge(&mut self, other: &MidiTrack) {
        for event in &other.events {
            self.add_event(event.clone());
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
