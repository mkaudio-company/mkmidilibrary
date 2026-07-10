//! MIDI event representation
//!
//! A MIDI event combines a timestamp with a MIDI message.

use std::cmp::Ordering;
use std::fmt;

use super::message::{MetaEvent, MidiMessage};

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

    /// Clear the linked-event index (e.g. before a track-restructuring
    /// operation that would leave it dangling).
    pub fn unlink_event(&mut self) {
        self.linked_event = None;
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
        compare_events(self, other, NoteSortOrder::NoteOnsBeforeOffs)
    }
}

/// Controls whether note-on or note-off events sort first when they share a
/// tick and no explicit sequence number breaks the tie. Mirrors upstream
/// midifile's `sortTrackNoteOnsBeforeOffs`/`sortTrackNoteOffsBeforeOns`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteSortOrder {
    /// Note-on before note-off at the same tick (upstream's default).
    #[default]
    NoteOnsBeforeOffs,
    /// Note-off before note-on at the same tick.
    NoteOffsBeforeOns,
}

/// Compare two events for sorting under an explicit note-on/off tie-break
/// order, matching upstream midifile's default comparator
/// (`eventCompareNoteOnsBeforeOffs` / `eventCompareNoteOffsBeforeOns`):
/// end-of-track meta events always sort last; other meta events, then
/// non-note channel/system messages, then notes (ordered per `order`).
/// If both events carry an explicit nonzero sequence number (e.g. assigned by
/// `MidiTrack::mark_sequence`, which happens automatically right after
/// reading a file), that original file order takes precedence over the
/// type-based rules above, matching upstream's `seq`-aware comparator.
pub fn compare_events(a: &MidiEvent, b: &MidiEvent, order: NoteSortOrder) -> Ordering {
    match a.tick.cmp(&b.tick) {
        Ordering::Equal => {}
        ord => return ord,
    }

    if a.seq != 0 && b.seq != 0 {
        return a.seq.cmp(&b.seq);
    }

    let priority = match order {
        NoteSortOrder::NoteOnsBeforeOffs => event_priority,
        NoteSortOrder::NoteOffsBeforeOns => event_priority_note_offs_before_ons,
    };
    match priority(&a.message).cmp(&priority(&b.message)) {
        Ordering::Equal => {}
        ord => return ord,
    }

    a.seq.cmp(&b.seq)
}

/// Sorting priority for event types with note-ons before note-offs (default).
fn event_priority(msg: &MidiMessage) -> i32 {
    if let MidiMessage::Meta(meta) = msg {
        return if matches!(meta, MetaEvent::EndOfTrack) { i32::MAX } else { 0 };
    }
    if msg.is_note_on() {
        return 2;
    }
    if msg.is_note_off() {
        return 3;
    }
    // Non-note channel messages (control change, program change, pitch bend,
    // pressure) and system messages sort before both note-ons and note-offs.
    1
}

/// Sorting priority for event types with note-offs before note-ons.
fn event_priority_note_offs_before_ons(msg: &MidiMessage) -> i32 {
    if let MidiMessage::Meta(meta) = msg {
        return if matches!(meta, MetaEvent::EndOfTrack) { i32::MAX } else { 0 };
    }
    if msg.is_note_off() {
        return 2;
    }
    if msg.is_note_on() {
        return 3;
    }
    1
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
            MidiEvent::note_off(100, 0, 60, 0),
            MidiEvent::note_on(100, 0, 60, 100),
            MidiEvent::note_on(0, 0, 60, 100),
        ];

        events.sort();

        // First event should be note on at tick 0
        assert_eq!(events[0].tick(), 0);
        // At tick 100, note on comes before note off by default, matching
        // upstream midifile's `eventCompareNoteOnsBeforeOffs`.
        assert!(events[1].is_note_on());
        assert!(events[2].is_note_off());
    }

    #[test]
    fn test_event_ordering_control_change_before_notes() {
        // Non-note channel messages (CC/PC/pitch-bend/pressure) sort before
        // both note-ons and note-offs at the same tick.
        let mut events = vec![
            MidiEvent::note_on(0, 0, 60, 100),
            MidiEvent::control_change(0, 0, 64, 127),
        ];
        events.sort();
        assert!(matches!(events[0].message(), MidiMessage::ControlChange { .. }));
        assert!(events[1].is_note_on());
    }

    #[test]
    fn test_event_ordering_end_of_track_always_last() {
        let mut events = vec![
            MidiEvent::new(10, MidiMessage::Meta(MetaEvent::EndOfTrack)),
            MidiEvent::new(10, MidiMessage::Meta(MetaEvent::Marker("x".into()))),
            MidiEvent::note_on(10, 0, 60, 100),
        ];
        events.sort();
        assert!(matches!(events[2].message(), MidiMessage::Meta(MetaEvent::EndOfTrack)));
    }

    #[test]
    fn test_event_ordering_seq_overrides_type_priority() {
        // With explicit nonzero sequence numbers (as assigned by
        // MidiTrack::mark_sequence), original file order wins over
        // type-based tie-breaking, even across different message types.
        let mut note_on = MidiEvent::note_on(0, 0, 60, 100);
        let mut cc = MidiEvent::control_change(0, 0, 64, 127);
        note_on.set_seq(1);
        cc.set_seq(2);

        let mut events = vec![cc.clone(), note_on.clone()];
        events.sort();
        assert!(events[0].is_note_on());
        assert!(matches!(events[1].message(), MidiMessage::ControlChange { .. }));
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

}
