//! MIDI event representation
//!
//! A MIDI event combines a timestamp with a MIDI message.

use std::cmp::Ordering;
use std::fmt;

use super::message::MidiMessage;

/// A MIDI event with timing information
#[derive(Debug, Clone, PartialEq)]
pub struct MidiEvent {
    /// Tick time (absolute or delta)
    tick: u64,
    /// The MIDI message
    message: MidiMessage,
    /// Track number (for multi-track files)
    track: usize,
    /// Time in seconds (computed)
    seconds: Option<f64>,
    /// Sequence number for stable sorting
    seq: u32,
    /// Linked event index (for note on/off pairing)
    linked_event: Option<usize>,
}

impl MidiEvent {
    /// Create a new MIDI event
    pub fn new(tick: u64, message: MidiMessage) -> Self {
        Self {
            tick,
            message,
            track: 0,
            seconds: None,
            seq: 0,
            linked_event: None,
        }
    }

    /// Create a MIDI event with track information
    pub fn with_track(tick: u64, message: MidiMessage, track: usize) -> Self {
        Self {
            tick,
            message,
            track,
            seconds: None,
            seq: 0,
            linked_event: None,
        }
    }

    /// Get the tick time
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Set the tick time
    pub fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    /// Get the message
    pub fn message(&self) -> &MidiMessage {
        &self.message
    }

    /// Get mutable message
    pub fn message_mut(&mut self) -> &mut MidiMessage {
        &mut self.message
    }

    /// Set the message
    pub fn set_message(&mut self, message: MidiMessage) {
        self.message = message;
    }

    /// Get the track number
    pub fn track(&self) -> usize {
        self.track
    }

    /// Set the track number
    pub fn set_track(&mut self, track: usize) {
        self.track = track;
    }

    /// Get the time in seconds (if computed)
    pub fn seconds(&self) -> Option<f64> {
        self.seconds
    }

    /// Set the time in seconds
    pub fn set_seconds(&mut self, seconds: f64) {
        self.seconds = Some(seconds);
    }

    /// Get the sequence number
    pub fn seq(&self) -> u32 {
        self.seq
    }

    /// Set the sequence number
    pub fn set_seq(&mut self, seq: u32) {
        self.seq = seq;
    }

    /// Get the linked event index
    pub fn linked_event(&self) -> Option<usize> {
        self.linked_event
    }

    /// Set the linked event index
    pub fn set_linked_event(&mut self, index: Option<usize>) {
        self.linked_event = index;
    }

    /// Check if this event is linked
    pub fn is_linked(&self) -> bool {
        self.linked_event.is_some()
    }

    /// Get the tick duration (for note events with linked note-off)
    pub fn tick_duration(&self, events: &[MidiEvent]) -> Option<u64> {
        self.linked_event.and_then(|idx| {
            events.get(idx).map(|linked| {
                if linked.tick() > self.tick {
                    linked.tick() - self.tick
                } else {
                    0
                }
            })
        })
    }

    /// Get the duration in seconds (for note events with linked note-off)
    pub fn duration_seconds(&self, events: &[MidiEvent]) -> Option<f64> {
        self.linked_event.and_then(|idx| {
            events.get(idx).and_then(|linked| {
                match (self.seconds, linked.seconds) {
                    (Some(start), Some(end)) => Some(end - start),
                    _ => None,
                }
            })
        })
    }

    /// Check if this is a Note On event
    pub fn is_note_on(&self) -> bool {
        self.message.is_note_on()
    }

    /// Check if this is a Note Off event
    pub fn is_note_off(&self) -> bool {
        self.message.is_note_off()
    }

    /// Check if this is a meta event
    pub fn is_meta(&self) -> bool {
        self.message.is_meta()
    }

    /// Get the channel (for channel messages)
    pub fn channel(&self) -> Option<u8> {
        self.message.channel()
    }

    /// Get the key/note number (for note events)
    pub fn key(&self) -> Option<u8> {
        match &self.message {
            MidiMessage::NoteOn { key, .. } | MidiMessage::NoteOff { key, .. } => Some(*key),
            _ => None,
        }
    }

    /// Get the velocity (for note events)
    pub fn velocity(&self) -> Option<u8> {
        match &self.message {
            MidiMessage::NoteOn { velocity, .. } | MidiMessage::NoteOff { velocity, .. } => {
                Some(*velocity)
            }
            _ => None,
        }
    }

    /// Create convenience note on event
    pub fn note_on(tick: u64, channel: u8, key: u8, velocity: u8) -> Self {
        Self::new(tick, MidiMessage::note_on(channel, key, velocity))
    }

    /// Create convenience note off event
    pub fn note_off(tick: u64, channel: u8, key: u8, velocity: u8) -> Self {
        Self::new(tick, MidiMessage::note_off(channel, key, velocity))
    }

    /// Create convenience control change event
    pub fn control_change(tick: u64, channel: u8, controller: u8, value: u8) -> Self {
        Self::new(tick, MidiMessage::control_change(channel, controller, value))
    }

    /// Create convenience program change event
    pub fn program_change(tick: u64, channel: u8, program: u8) -> Self {
        Self::new(tick, MidiMessage::program_change(channel, program))
    }
}

impl fmt::Display for MidiEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.tick, self.message)
    }
}

impl Eq for MidiEvent {}

impl PartialOrd for MidiEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MidiEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Primary: sort by tick
        match self.tick.cmp(&other.tick) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Secondary: Note offs before note ons at same tick
        let self_priority = event_priority(&self.message);
        let other_priority = event_priority(&other.message);
        match self_priority.cmp(&other_priority) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Tertiary: by sequence number for stability
        self.seq.cmp(&other.seq)
    }
}

/// Get sorting priority for event types
fn event_priority(msg: &MidiMessage) -> i32 {
    match msg {
        // Meta events first (tempo changes affect timing)
        MidiMessage::Meta(_) => 0,
        // Note offs before note ons
        MidiMessage::NoteOff { .. } => 1,
        MidiMessage::NoteOn { velocity: 0, .. } => 1,
        // Then program changes
        MidiMessage::ProgramChange { .. } => 2,
        // Then control changes
        MidiMessage::ControlChange { .. } => 3,
        // Then note ons
        MidiMessage::NoteOn { .. } => 4,
        // Everything else
        _ => 5,
    }
}

/// Builder for creating MIDI events
pub struct MidiEventBuilder {
    tick: u64,
    message: Option<MidiMessage>,
    track: usize,
    seq: u32,
}

impl MidiEventBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            tick: 0,
            message: None,
            track: 0,
            seq: 0,
        }
    }

    /// Set the tick time
    pub fn tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    /// Set the message
    pub fn message(mut self, message: MidiMessage) -> Self {
        self.message = Some(message);
        self
    }

    /// Set the track
    pub fn track(mut self, track: usize) -> Self {
        self.track = track;
        self
    }

    /// Set the sequence number
    pub fn seq(mut self, seq: u32) -> Self {
        self.seq = seq;
        self
    }

    /// Build the event
    pub fn build(self) -> Option<MidiEvent> {
        self.message.map(|msg| {
            let mut event = MidiEvent::with_track(self.tick, msg, self.track);
            event.set_seq(self.seq);
            event
        })
    }
}

impl Default for MidiEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = MidiEvent::note_on(0, 0, 60, 100);
        assert!(event.is_note_on());
        assert_eq!(event.tick(), 0);
        assert_eq!(event.key(), Some(60));
        assert_eq!(event.velocity(), Some(100));
        assert_eq!(event.channel(), Some(0));
    }

    #[test]
    fn test_event_ordering() {
        let mut events = vec![
            MidiEvent::note_on(100, 0, 60, 100),
            MidiEvent::note_off(100, 0, 60, 0),
            MidiEvent::note_on(0, 0, 60, 100),
        ];

        events.sort();

        // First event should be note on at tick 0
        assert_eq!(events[0].tick(), 0);
        // At tick 100, note off should come before note on
        assert!(events[1].is_note_off());
        assert!(events[2].is_note_on());
    }

    #[test]
    fn test_event_linking() {
        let mut on = MidiEvent::note_on(0, 0, 60, 100);
        let off = MidiEvent::note_off(100, 0, 60, 0);
        on.set_linked_event(Some(1));

        assert!(on.is_linked());
        let events = vec![on.clone(), off];
        assert_eq!(on.tick_duration(&events), Some(100));
    }

    #[test]
    fn test_event_builder() {
        let event = MidiEventBuilder::new()
            .tick(480)
            .message(MidiMessage::note_on(0, 60, 80))
            .track(1)
            .seq(5)
            .build()
            .unwrap();

        assert_eq!(event.tick(), 480);
        assert_eq!(event.track(), 1);
        assert_eq!(event.seq(), 5);
    }
}
