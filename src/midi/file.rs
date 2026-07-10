//! MIDI file I/O
//!
//! This module provides reading and writing of Standard MIDI Files (SMF).

use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use super::event::{MidiEvent, NoteSortOrder};
use super::message::{MetaEvent, MidiMessage};
use super::track::MidiTrack;
use super::{MidiError, MidiFormat};

/// Whether a `MidiFile`'s tracks are stored separately or have been merged
/// into a single track by `join_tracks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackState {
    /// Tracks are stored separately (the normal state for a multi-track file).
    #[default]
    Split,
    /// All events have been merged into a single track (`tracks[0]`), with
    /// each event's `track()` field recording which original track it came
    /// from, so `split_tracks()` can restore the original layout exactly.
    Joined,
}

/// The tick representation currently used across a `MidiFile`'s tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickState {
    /// Every track uses absolute tick timing.
    Absolute,
    /// Every track uses delta tick timing.
    Delta,
    /// Tracks disagree (some absolute, some delta) — not a normal state.
    Mixed,
}

/// A Standard MIDI File
#[derive(Debug, Clone)]
pub struct MidiFile {
    /// MIDI file format (0, 1, or 2)
    format: MidiFormat,
    /// Ticks per quarter note (timing resolution)
    ticks_per_quarter: u16,
    /// Tracks in this file
    tracks: Vec<MidiTrack>,
    /// Whether `tracks` currently holds separate tracks or a single
    /// `join_tracks`-merged track that can be restored with `split_tracks`.
    track_state: TrackState,
    /// Time map for tick-to-seconds conversion. Lazily rebuilt by `build_time_map`,
    /// which needs to run from `&self` accessors (e.g. `ticks_to_seconds`), hence
    /// the interior mutability.
    time_map: RefCell<Option<TimeMap>>,
}

impl MidiFile {
    /// Create a new empty MIDI file
    pub fn new() -> Self {
        Self {
            format: MidiFormat::MultiTrack,
            ticks_per_quarter: 480,
            tracks: Vec::new(),
            track_state: TrackState::Split,
            time_map: RefCell::new(None),
        }
    }

    /// Create a MIDI file with specified format and resolution
    pub fn with_format(format: MidiFormat, ticks_per_quarter: u16) -> Self {
        Self {
            format,
            ticks_per_quarter,
            tracks: Vec::new(),
            track_state: TrackState::Split,
            time_map: RefCell::new(None),
        }
    }

    /// Read a MIDI file from disk
    pub fn read(path: impl AsRef<Path>) -> Result<Self, MidiError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::from_bytes(&data)
    }

    /// Parse MIDI file from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, MidiError> {
        if data.len() < 14 {
            return Err(MidiError::InvalidHeader);
        }

        // Parse header chunk
        if &data[0..4] != b"MThd" {
            return Err(MidiError::InvalidHeader);
        }

        let header_len = read_u32_be(&data[4..8]);
        if header_len < 6 {
            return Err(MidiError::InvalidHeader);
        }

        let format = MidiFormat::try_from(read_u16_be(&data[8..10]))?;
        let num_tracks = read_u16_be(&data[10..12]) as usize;
        let raw_division = read_u16_be(&data[12..14]);

        // SMPTE-style timing: top byte is frames-per-second stored as a negative
        // two's-complement i8 (e.g. -24/-25/-29/-30), bottom byte is the subframe
        // (ticks-per-frame) resolution. Convert to an effective ticks-per-quarter
        // value (frames_per_second * subframes), matching upstream midifile's
        // handling. This is a one-way conversion: writing the file back out emits
        // a plain (non-SMPTE) PPQN header at this resolution rather than
        // reproducing the original SMPTE division bytes.
        let ticks_per_quarter = if raw_division & 0x8000 != 0 {
            let frames_per_second = ((raw_division >> 8) as u8 as i8).unsigned_abs() as u16;
            let subframes = raw_division & 0xFF;
            frames_per_second.saturating_mul(subframes)
        } else {
            raw_division
        };

        let mut midi_file = Self {
            format,
            ticks_per_quarter,
            tracks: Vec::with_capacity(num_tracks),
            track_state: TrackState::Split,
            time_map: RefCell::new(None),
        };

        // Parse track chunks
        let mut pos = 8 + header_len as usize;
        while pos < data.len() && midi_file.tracks.len() < num_tracks {
            if pos + 8 > data.len() {
                break;
            }

            if &data[pos..pos + 4] != b"MTrk" {
                return Err(MidiError::InvalidTrackHeader);
            }

            let track_len = read_u32_be(&data[pos + 4..pos + 8]) as usize;
            pos += 8;

            if pos + track_len > data.len() {
                return Err(MidiError::UnexpectedEof);
            }

            let mut track = parse_track(&data[pos..pos + track_len])?;
            // Preserve the file's original event order as a tie-break, matching
            // upstream midifile's automatic markSequence() right after reading.
            track.mark_sequence();
            midi_file.tracks.push(track);
            pos += track_len;
        }

        Ok(midi_file)
    }

    /// Write MIDI file to disk
    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), MidiError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let bytes = self.to_bytes();
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Header chunk
        data.extend(b"MThd");
        data.extend(&6u32.to_be_bytes());
        data.extend(&(self.format as u16).to_be_bytes());
        data.extend(&(self.tracks.len() as u16).to_be_bytes());
        data.extend(&self.ticks_per_quarter.to_be_bytes());

        // Track chunks
        for track in &self.tracks {
            let track_data = encode_track(track);
            data.extend(b"MTrk");
            data.extend(&(track_data.len() as u32).to_be_bytes());
            data.extend(track_data);
        }

        data
    }

    /// Get the format
    pub fn format(&self) -> MidiFormat {
        self.format
    }

    /// Set the format
    pub fn set_format(&mut self, format: MidiFormat) {
        self.format = format;
    }

    /// Get ticks per quarter note
    pub fn ticks_per_quarter(&self) -> u16 {
        self.ticks_per_quarter
    }

    /// Set ticks per quarter note
    pub fn set_ticks_per_quarter(&mut self, tpq: u16) {
        self.ticks_per_quarter = tpq;
        *self.time_map.borrow_mut() = None; // Invalidate time map
    }

    /// Get all tracks
    pub fn tracks(&self) -> &[MidiTrack] {
        &self.tracks
    }

    /// Get mutable tracks
    pub fn tracks_mut(&mut self) -> &mut Vec<MidiTrack> {
        *self.time_map.borrow_mut() = None; // Invalidate time map
        &mut self.tracks
    }

    /// Get a specific track
    pub fn track(&self, index: usize) -> Option<&MidiTrack> {
        self.tracks.get(index)
    }

    /// Get a mutable specific track
    pub fn track_mut(&mut self, index: usize) -> Option<&mut MidiTrack> {
        *self.time_map.borrow_mut() = None;
        self.tracks.get_mut(index)
    }

    /// Get number of tracks
    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }

    /// Add a new track and return a mutable reference to it
    pub fn add_track(&mut self) -> &mut MidiTrack {
        *self.time_map.borrow_mut() = None;
        self.tracks.push(MidiTrack::new());
        self.tracks.last_mut().unwrap()
    }

    /// Add an existing track
    pub fn add_track_from(&mut self, track: MidiTrack) {
        *self.time_map.borrow_mut() = None;
        self.tracks.push(track);
    }

    /// Delete a track
    pub fn delete_track(&mut self, index: usize) -> Option<MidiTrack> {
        if index < self.tracks.len() {
            *self.time_map.borrow_mut() = None;
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }

    /// Merge all tracks into a single track (Format 0)
    pub fn merge_tracks(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let mut merged = MidiTrack::new();
        for track in &self.tracks {
            merged.merge(track);
        }
        merged.sort();

        self.tracks.clear();
        self.tracks.push(merged);
        self.format = MidiFormat::SingleTrack;
        // This is a lossy, one-way merge (unlike `join_tracks`), so it does
        // not leave the file in the reversible "joined" state.
        self.track_state = TrackState::Split;
        *self.time_map.borrow_mut() = None;
    }

    /// Get the current track layout state (see `TrackState`).
    pub fn track_state(&self) -> TrackState {
        self.track_state
    }

    /// Whether `join_tracks` has merged all tracks into a single track.
    pub fn has_joined_tracks(&self) -> bool {
        self.track_state == TrackState::Joined
    }

    /// Whether tracks are currently stored separately (the normal state).
    pub fn has_split_tracks(&self) -> bool {
        self.track_state == TrackState::Split
    }

    /// Get the original track index an event belonged to before `join_tracks`
    /// merged everything into one track (recorded via `MidiEvent::track`).
    pub fn get_split_track(&self, event: &MidiEvent) -> usize {
        event.track()
    }

    /// Merge all tracks into a single track (`tracks[0]`), recording each
    /// event's original track index (via `MidiEvent::set_track`) so
    /// `split_tracks()` can restore the exact original layout later. Unlike
    /// `merge_tracks`, this is reversible and does not change `format`. A
    /// no-op if tracks are already joined.
    pub fn join_tracks(&mut self) {
        if self.track_state == TrackState::Joined {
            return;
        }

        let mut joined = MidiTrack::new();
        for (track_index, track) in self.tracks.iter().enumerate() {
            for event in track.events() {
                let mut event = event.clone();
                event.set_track(track_index);
                event.unlink_event();
                joined.add_event(event);
            }
        }
        joined.mark_sequence();
        joined.sort();

        self.tracks = vec![joined];
        self.track_state = TrackState::Joined;
        *self.time_map.borrow_mut() = None;
    }

    /// Restore the track layout that was in effect before `join_tracks()`,
    /// using each event's recorded original track index. A no-op if tracks
    /// are not currently joined.
    pub fn split_tracks(&mut self) {
        if self.track_state != TrackState::Joined {
            return;
        }

        let joined = match self.tracks.first() {
            Some(t) => t,
            None => return,
        };

        let max_track = joined.events().iter().map(|e| e.track()).max().unwrap_or(0);
        let mut split: Vec<MidiTrack> = (0..=max_track).map(|_| MidiTrack::new()).collect();
        for event in joined.events() {
            let track_index = event.track();
            let mut event = event.clone();
            event.unlink_event();
            split[track_index].add_event(event);
        }
        for track in &mut split {
            track.sort();
        }

        self.tracks = split;
        self.track_state = TrackState::Split;
        *self.time_map.borrow_mut() = None;
    }

    /// Split track 0 by channel (for Format 0 -> Format 1 conversion). Joins
    /// all tracks first (matching upstream midifile's `splitTracksByChannel`,
    /// which calls `joinTracks()` internally), so this produces correct
    /// results regardless of how many tracks currently exist — previously
    /// this only read `tracks[0]` and silently discarded every other track.
    pub fn split_tracks_by_channel(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        self.join_tracks();

        let source = &self.tracks[0];
        let mut channel_tracks: Vec<MidiTrack> = (0..16).map(|_| MidiTrack::new()).collect();
        let mut tempo_track = MidiTrack::with_name("Tempo");

        for event in source.events() {
            let mut event = event.clone();
            event.unlink_event();
            if event.is_meta() {
                tempo_track.add_event(event);
            } else if let Some(ch) = event.channel() {
                channel_tracks[ch as usize].add_event(event);
            }
        }

        // Remove empty tracks
        let mut new_tracks = vec![tempo_track];
        for (i, track) in channel_tracks.into_iter().enumerate() {
            if !track.is_empty() {
                let mut t = track;
                t.set_name(format!("Channel {}", i));
                new_tracks.push(t);
            }
        }

        self.tracks = new_tracks;
        self.format = MidiFormat::MultiTrack;
        self.track_state = TrackState::Split;
        *self.time_map.borrow_mut() = None;
    }

    /// Get the total duration in ticks
    pub fn total_ticks(&self) -> u64 {
        self.tracks.iter().map(|t| t.last_tick()).max().unwrap_or(0)
    }

    /// Get the total duration expressed in quarter notes.
    pub fn get_file_duration_in_quarters(&self) -> f64 {
        self.total_ticks() as f64 / self.ticks_per_quarter as f64
    }

    /// Whether every track currently uses absolute tick timing.
    pub fn is_absolute_ticks(&self) -> bool {
        self.tracks.iter().all(|t| t.is_absolute_time())
    }

    /// Whether every track currently uses delta tick timing.
    pub fn is_delta_ticks(&self) -> bool {
        self.tracks.iter().all(|t| !t.is_absolute_time())
    }

    /// Get the current tick representation across all tracks.
    pub fn get_tick_state(&self) -> TickState {
        if self.is_absolute_ticks() {
            TickState::Absolute
        } else if self.is_delta_ticks() {
            TickState::Delta
        } else {
            TickState::Mixed
        }
    }

    /// Convert every track to absolute tick timing.
    pub fn make_absolute_ticks(&mut self) {
        for track in &mut self.tracks {
            track.make_absolute_times();
        }
    }

    /// Convert every track to delta tick timing.
    pub fn make_delta_ticks(&mut self) {
        for track in &mut self.tracks {
            track.make_delta_times();
        }
        *self.time_map.borrow_mut() = None;
    }

    /// Append `count` new empty tracks.
    pub fn add_tracks(&mut self, count: usize) {
        for _ in 0..count {
            self.tracks.push(MidiTrack::new());
        }
        *self.time_map.borrow_mut() = None;
    }

    /// Reserve capacity for at least `additional` more events in a track.
    pub fn allocate_events(&mut self, track: usize, additional: usize) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.reserve(additional);
        Ok(())
    }

    /// Remove tracks that contain no events.
    pub fn remove_empties(&mut self) {
        self.tracks.retain(|t| !t.is_empty());
        *self.time_map.borrow_mut() = None;
    }

    /// The number of tracks this file would have if split into Format 1
    /// layout by channel (one tempo/meta track plus one per channel in use),
    /// without actually performing the split.
    pub fn get_track_count_as_type1(&self) -> usize {
        let mut channels_used = [false; 16];
        for track in &self.tracks {
            for event in track.events() {
                if let Some(ch) = event.channel() {
                    channels_used[ch as usize] = true;
                }
            }
        }
        1 + channels_used.iter().filter(|&&used| used).count()
    }

    /// Merge the track at `index2` into the track at `index1` and remove
    /// `index2`, unlike `merge_tracks` which merges every track at once.
    pub fn merge_two_tracks(&mut self, index1: usize, index2: usize) -> Result<(), MidiError> {
        let len = self.tracks.len();
        if index1 >= len || index2 >= len {
            return Err(MidiError::TrackOutOfBounds(index1.max(index2)));
        }
        if index1 == index2 {
            return Ok(());
        }

        let other = self.tracks[index2].clone();
        self.tracks[index1].merge(&other);
        self.tracks[index1].sort();
        self.tracks.remove(index2);
        *self.time_map.borrow_mut() = None;
        Ok(())
    }

    /// Rescale the file so that each tick represents exactly one
    /// millisecond, based on the first tempo event found (or 120 BPM if
    /// none). Existing event ticks are rescaled proportionally so absolute
    /// timing is preserved.
    pub fn set_millisecond_ticks(&mut self) {
        let old_tpq = self.ticks_per_quarter as f64;
        let us_per_quarter = self
            .tracks
            .iter()
            .flat_map(|t| t.events())
            .find_map(|e| match e.message() {
                MidiMessage::Meta(MetaEvent::Tempo(us)) => Some(*us),
                _ => None,
            })
            .unwrap_or(500_000) as f64;

        let new_tpq = (us_per_quarter / 1000.0).round().max(1.0);
        let scale = new_tpq / old_tpq;

        for track in &mut self.tracks {
            for event in track.events_mut() {
                let new_tick = (event.tick() as f64 * scale).round() as u64;
                event.set_tick(new_tick);
            }
        }

        self.ticks_per_quarter = new_tpq as u16;
        *self.time_map.borrow_mut() = None;
    }

    /// Get the total duration in seconds
    pub fn total_seconds(&self) -> f64 {
        self.build_time_map();
        self.ticks_to_seconds(self.total_ticks())
    }

    /// Convert ticks to seconds
    pub fn ticks_to_seconds(&self, ticks: u64) -> f64 {
        self.build_time_map();
        self.time_map
            .borrow()
            .as_ref()
            .unwrap()
            .ticks_to_seconds(ticks)
    }

    /// Convert seconds to ticks
    pub fn seconds_to_ticks(&self, seconds: f64) -> u64 {
        self.build_time_map();
        self.time_map
            .borrow()
            .as_ref()
            .unwrap()
            .seconds_to_ticks(seconds)
    }

    /// Build the time map for tempo conversion
    fn build_time_map(&self) {
        if self.time_map.borrow().is_some() {
            return;
        }

        // Collect all tempo events from all tracks
        let mut tempo_events: Vec<(u64, u32)> = Vec::new();
        for track in &self.tracks {
            for event in track.events() {
                if let MidiMessage::Meta(MetaEvent::Tempo(us)) = event.message() {
                    tempo_events.push((event.tick(), *us));
                }
            }
        }

        tempo_events.sort_by_key(|(tick, _)| *tick);

        // Default tempo if none specified
        if tempo_events.is_empty() {
            tempo_events.push((0, 500_000)); // 120 BPM
        }

        *self.time_map.borrow_mut() = Some(TimeMap::new(tempo_events, self.ticks_per_quarter));
    }

    /// Add a note to a track
    pub fn add_note(
        &mut self,
        track: usize,
        start_tick: u64,
        duration: u64,
        channel: u8,
        key: u8,
        velocity: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_note(start_tick, duration, channel, key, velocity);
        *self.time_map.borrow_mut() = None;
        Ok(())
    }

    /// Add a tempo change
    pub fn add_tempo(&mut self, track: usize, tick: u64, bpm: f64) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_tempo(tick, bpm);
        *self.time_map.borrow_mut() = None;
        Ok(())
    }

    /// Add a time signature
    pub fn add_time_signature(
        &mut self,
        track: usize,
        tick: u64,
        numerator: u8,
        denominator: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_time_signature(tick, numerator, denominator);
        *self.time_map.borrow_mut() = None;
        Ok(())
    }

    /// Add a compound-meter time signature (e.g. 6/8)
    pub fn add_compound_time_signature(
        &mut self,
        track: usize,
        tick: u64,
        numerator: u8,
        denominator: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_compound_time_signature(tick, numerator, denominator);
        Ok(())
    }

    /// Add a key signature
    pub fn add_key_signature(
        &mut self,
        track: usize,
        tick: u64,
        sharps: i8,
        minor: bool,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_key_signature(tick, sharps, minor);
        Ok(())
    }

    /// Add a pitch bend event
    pub fn add_pitch_bend(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
        value: u16,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_pitch_bend(tick, channel, value);
        Ok(())
    }

    /// Add a controller (control change) event
    pub fn add_controller(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
        controller: u8,
        value: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_controller(tick, channel, controller, value);
        Ok(())
    }

    /// Add a sustain pedal event with an explicit value
    pub fn add_sustain(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
        value: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_sustain(tick, channel, value);
        Ok(())
    }

    /// Add a sustain pedal on event
    pub fn add_sustain_on(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
    ) -> Result<(), MidiError> {
        self.add_sustain(track, tick, channel, 127)
    }

    /// Add a sustain pedal off event
    pub fn add_sustain_off(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
    ) -> Result<(), MidiError> {
        self.add_sustain(track, tick, channel, 0)
    }

    /// Add a patch (program) change event
    pub fn add_patch_change(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
        program: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_patch_change(tick, channel, program);
        Ok(())
    }

    /// Add a generic meta event
    pub fn add_meta_event(
        &mut self,
        track: usize,
        tick: u64,
        meta: MetaEvent,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_meta_event(tick, meta);
        Ok(())
    }

    /// Add a text meta event
    pub fn add_text(
        &mut self,
        track: usize,
        tick: u64,
        text: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::Text(text.into()))
    }

    /// Add a copyright meta event
    pub fn add_copyright(
        &mut self,
        track: usize,
        tick: u64,
        text: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::Copyright(text.into()))
    }

    /// Add a track name meta event
    pub fn add_track_name(
        &mut self,
        track: usize,
        tick: u64,
        name: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::TrackName(name.into()))
    }

    /// Add an instrument name meta event
    pub fn add_instrument_name(
        &mut self,
        track: usize,
        tick: u64,
        name: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::InstrumentName(name.into()))
    }

    /// Add a lyric meta event
    pub fn add_lyric(
        &mut self,
        track: usize,
        tick: u64,
        text: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::Lyric(text.into()))
    }

    /// Add a marker meta event
    pub fn add_marker(
        &mut self,
        track: usize,
        tick: u64,
        text: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::Marker(text.into()))
    }

    /// Add a cue point meta event
    pub fn add_cue(
        &mut self,
        track: usize,
        tick: u64,
        text: impl Into<String>,
    ) -> Result<(), MidiError> {
        self.add_meta_event(track, tick, MetaEvent::CuePoint(text.into()))
    }

    /// Write the RPN sequence to set the pitch-bend range on a channel (see
    /// `MidiTrack::add_pitch_bend_range`).
    pub fn set_pitch_bend_range(
        &mut self,
        track: usize,
        tick: u64,
        channel: u8,
        semitones: u8,
        cents: u8,
    ) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(track)
            .ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_pitch_bend_range(tick, channel, semitones, cents);
        Ok(())
    }

    /// Sort a single track with an explicit note-on/off tie-break order.
    pub fn sort_track(&mut self, index: usize, order: NoteSortOrder) -> Result<(), MidiError> {
        let track = self
            .tracks
            .get_mut(index)
            .ok_or(MidiError::TrackOutOfBounds(index))?;
        track.sort_with_order(order);
        *self.time_map.borrow_mut() = None;
        Ok(())
    }

    /// Sort all tracks with an explicit note-on/off tie-break order.
    pub fn sort_tracks(&mut self, order: NoteSortOrder) {
        for track in &mut self.tracks {
            track.sort_with_order(order);
        }
        *self.time_map.borrow_mut() = None;
    }

    /// Ensure all tracks have end of track markers
    pub fn finalize(&mut self) {
        for track in &mut self.tracks {
            track.ensure_end_of_track();
        }
    }

    /// Link all note events across all tracks
    pub fn link_note_events(&mut self) {
        for track in &mut self.tracks {
            track.link_note_events();
        }
    }

    /// Update seconds for all events based on tempo map
    pub fn update_seconds(&mut self) {
        self.build_time_map();

        // Collect ticks first to avoid borrow conflict
        let ticks_per_track: Vec<Vec<u64>> = self
            .tracks
            .iter()
            .map(|track| track.events().iter().map(|e| e.tick()).collect())
            .collect();

        // Calculate seconds for each tick
        let time_map_ref = self.time_map.borrow();
        let time_map = time_map_ref.as_ref().unwrap();
        let seconds_per_track: Vec<Vec<f64>> = ticks_per_track
            .iter()
            .map(|ticks| {
                ticks
                    .iter()
                    .map(|&t| time_map.ticks_to_seconds(t))
                    .collect()
            })
            .collect();
        drop(time_map_ref);

        for (track_idx, track) in self.tracks.iter_mut().enumerate() {
            for (event_idx, event) in track.events_mut().iter_mut().enumerate() {
                event.set_seconds(seconds_per_track[track_idx][event_idx]);
            }
        }
    }
}

impl Default for MidiFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Time map for tick-to-seconds conversion
#[derive(Debug, Clone)]
struct TimeMap {
    /// Tempo change points: (tick, seconds at that tick, microseconds per quarter)
    points: Vec<(u64, f64, u32)>,
    ticks_per_quarter: u16,
}

impl TimeMap {
    fn new(tempo_events: Vec<(u64, u32)>, ticks_per_quarter: u16) -> Self {
        let mut points = Vec::new();
        let mut current_seconds = 0.0;
        let mut prev_tick: u64 = 0;
        let mut prev_tempo: u32 = 500_000; // Default 120 BPM

        for (tick, tempo) in tempo_events {
            // Calculate time elapsed since last tempo change
            let tick_delta = tick - prev_tick;
            let seconds_per_tick = prev_tempo as f64 / 1_000_000.0 / ticks_per_quarter as f64;
            current_seconds += tick_delta as f64 * seconds_per_tick;

            points.push((tick, current_seconds, tempo));
            prev_tick = tick;
            prev_tempo = tempo;
        }

        Self {
            points,
            ticks_per_quarter,
        }
    }

    fn ticks_to_seconds(&self, ticks: u64) -> f64 {
        if self.points.is_empty() {
            // Default 120 BPM
            let seconds_per_tick = 0.5 / self.ticks_per_quarter as f64;
            return ticks as f64 * seconds_per_tick;
        }

        // Find the tempo region
        let mut base_tick: u64 = 0;
        let mut base_seconds = 0.0;
        let mut tempo: u32 = 500_000;

        for &(point_tick, point_seconds, point_tempo) in &self.points {
            if point_tick > ticks {
                break;
            }
            base_tick = point_tick;
            base_seconds = point_seconds;
            tempo = point_tempo;
        }

        let tick_delta = ticks - base_tick;
        let seconds_per_tick = tempo as f64 / 1_000_000.0 / self.ticks_per_quarter as f64;
        base_seconds + tick_delta as f64 * seconds_per_tick
    }

    fn seconds_to_ticks(&self, seconds: f64) -> u64 {
        if self.points.is_empty() {
            // Default 120 BPM
            let ticks_per_second = self.ticks_per_quarter as f64 * 2.0;
            return (seconds * ticks_per_second) as u64;
        }

        // Find the tempo region
        let mut base_tick: u64 = 0;
        let mut base_seconds = 0.0;
        let mut tempo: u32 = 500_000;

        for &(point_tick, point_seconds, point_tempo) in &self.points {
            if point_seconds > seconds {
                break;
            }
            base_tick = point_tick;
            base_seconds = point_seconds;
            tempo = point_tempo;
        }

        let seconds_delta = seconds - base_seconds;
        let ticks_per_second = self.ticks_per_quarter as f64 * 1_000_000.0 / tempo as f64;
        base_tick + (seconds_delta * ticks_per_second) as u64
    }
}

// Helper functions

fn read_u16_be(data: &[u8]) -> u16 {
    ((data[0] as u16) << 8) | data[1] as u16
}

fn read_u32_be(data: &[u8]) -> u32 {
    ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | data[3] as u32
}

fn read_varlen(data: &[u8]) -> Option<(u32, usize)> {
    let mut value: u32 = 0;
    let mut bytes_read = 0;

    for &byte in data.iter().take(4) {
        bytes_read += 1;
        value = (value << 7) | (byte & 0x7F) as u32;

        if byte & 0x80 == 0 {
            return Some((value, bytes_read));
        }
    }

    None
}

fn write_varlen(value: u32) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }

    let mut bytes = Vec::new();
    let mut v = value;

    bytes.push((v & 0x7F) as u8);
    v >>= 7;

    while v > 0 {
        bytes.push(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }

    bytes.reverse();
    bytes
}

fn parse_track(data: &[u8]) -> Result<MidiTrack, MidiError> {
    let mut track = MidiTrack::new();
    let mut pos = 0;
    let mut running_status: Option<u8> = None;
    let mut current_tick: u64 = 0;

    while pos < data.len() {
        // Read delta time
        let (delta, delta_len) = read_varlen(&data[pos..]).ok_or(MidiError::InvalidVarLen)?;
        pos += delta_len;
        current_tick += delta as u64;

        if pos >= data.len() {
            break;
        }

        let status = data[pos];

        // Check for meta event
        if status == 0xFF {
            pos += 1;
            if pos >= data.len() {
                return Err(MidiError::UnexpectedEof);
            }

            let (meta, meta_len) =
                MetaEvent::from_bytes(&data[pos..]).ok_or(MidiError::UnexpectedEof)?;
            pos += meta_len;

            let event = MidiEvent::new(current_tick, MidiMessage::Meta(meta));
            track.add_event(event);
            running_status = None;
            continue;
        }

        // Check for SysEx
        if status == 0xF0 || status == 0xF7 {
            pos += 1;
            let (length, len_bytes) = read_varlen(&data[pos..]).ok_or(MidiError::InvalidVarLen)?;
            pos += len_bytes;

            let sysex_data = data[pos..pos + length as usize].to_vec();
            pos += length as usize;

            let event = MidiEvent::new(current_tick, MidiMessage::SysEx(sysex_data));
            track.add_event(event);
            running_status = None;
            continue;
        }

        // Channel message
        let (actual_status, data_start) = if status & 0x80 != 0 {
            running_status = Some(status);
            pos += 1;
            (status, pos)
        } else {
            // Use running status
            let rs = running_status.ok_or(MidiError::InvalidRunningStatus)?;
            (rs, pos)
        };

        let channel = actual_status & 0x0F;
        let message = match actual_status & 0xF0 {
            0x80 => {
                if data_start + 2 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 2;
                MidiMessage::NoteOff {
                    channel,
                    key: data[data_start],
                    velocity: data[data_start + 1],
                }
            }
            0x90 => {
                if data_start + 2 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 2;
                MidiMessage::NoteOn {
                    channel,
                    key: data[data_start],
                    velocity: data[data_start + 1],
                }
            }
            0xA0 => {
                if data_start + 2 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 2;
                MidiMessage::PolyPressure {
                    channel,
                    key: data[data_start],
                    pressure: data[data_start + 1],
                }
            }
            0xB0 => {
                if data_start + 2 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 2;
                MidiMessage::ControlChange {
                    channel,
                    controller: data[data_start],
                    value: data[data_start + 1],
                }
            }
            0xC0 => {
                if data_start + 1 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 1;
                MidiMessage::ProgramChange {
                    channel,
                    program: data[data_start],
                }
            }
            0xD0 => {
                if data_start + 1 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 1;
                MidiMessage::ChannelPressure {
                    channel,
                    pressure: data[data_start],
                }
            }
            0xE0 => {
                if data_start + 2 > data.len() {
                    return Err(MidiError::UnexpectedEof);
                }
                pos = data_start + 2;
                MidiMessage::PitchBend {
                    channel,
                    value: (data[data_start] as u16) | ((data[data_start + 1] as u16) << 7),
                }
            }
            _ => return Err(MidiError::InvalidStatus(actual_status)),
        };

        let event = MidiEvent::new(current_tick, message);
        track.add_event(event);
    }

    Ok(track)
}

fn encode_track(track: &MidiTrack) -> Vec<u8> {
    let mut data = Vec::new();
    let mut prev_tick: u64 = 0;

    // Clone and sort if needed
    let mut events: Vec<MidiEvent> = track.events().to_vec();
    events.sort();

    for event in &events {
        // Write delta time
        let delta = event.tick().saturating_sub(prev_tick);
        data.extend(write_varlen(delta as u32));
        prev_tick = event.tick();

        // Write message
        data.extend(event.message().to_bytes());
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_split_tracks_roundtrip() {
        let mut file = MidiFile::new();
        let track0 = file.add_track();
        track0.add_note(0, 480, 0, 60, 100);
        let track1 = file.add_track();
        track1.add_note(0, 480, 1, 64, 90);

        assert!(file.has_split_tracks());
        file.join_tracks();
        assert!(file.has_joined_tracks());
        assert_eq!(file.num_tracks(), 1);

        // Every event should remember which original track it came from.
        for event in file.track(0).unwrap().events() {
            let ch = event.channel().unwrap();
            assert_eq!(file.get_split_track(event), ch as usize);
        }

        file.split_tracks();
        assert!(file.has_split_tracks());
        assert_eq!(file.num_tracks(), 2);
        assert_eq!(file.track(0).unwrap().events()[0].channel(), Some(0));
        assert_eq!(file.track(1).unwrap().events()[0].channel(), Some(1));
    }

    #[test]
    fn test_split_tracks_by_channel_does_not_drop_other_tracks() {
        // Regression test: split_tracks_by_channel used to only read
        // tracks[0] and silently discard every other track.
        let mut file = MidiFile::new();
        let track0 = file.add_track();
        track0.add_note(0, 480, 0, 60, 100);
        let track1 = file.add_track();
        track1.add_note(0, 480, 1, 64, 90);

        file.split_tracks_by_channel();

        let has_channel = |ch: u8| {
            file.tracks()
                .iter()
                .any(|t| t.events().iter().any(|e| e.channel() == Some(ch)))
        };
        assert!(has_channel(0), "channel 0 note from track 0 was dropped");
        assert!(has_channel(1), "channel 1 note from track 1 was dropped");
    }

    #[test]
    fn test_merge_and_extract_clear_dangling_links() {
        // Regression test: merge()/extract_channel() used to clone events
        // (including their linked_event index) into a differently indexed
        // Vec, leaving linked_event pointing at the wrong (or out of bounds)
        // event.
        let mut track = MidiTrack::new();
        track.add_note(0, 480, 0, 60, 100);
        track.link_note_events();
        assert!(track.events()[0].is_linked());

        let mut merged = MidiTrack::new();
        merged.merge(&track);
        for event in merged.events() {
            assert!(
                !event.is_linked(),
                "merge() must clear stale linked_event indices"
            );
        }

        let extracted = track.extract_channel(0);
        for event in extracted.events() {
            assert!(
                !event.is_linked(),
                "extract_channel() must clear stale linked_event indices"
            );
        }
    }

    #[test]
    fn test_tick_state_and_conversion() {
        let mut file = MidiFile::new();
        let track = file.add_track();
        track.add_note(0, 480, 0, 60, 100);
        track.add_note(480, 480, 0, 64, 100);

        assert_eq!(file.get_tick_state(), TickState::Absolute);
        file.make_delta_ticks();
        assert_eq!(file.get_tick_state(), TickState::Delta);
        file.make_absolute_ticks();
        assert_eq!(file.get_tick_state(), TickState::Absolute);
        assert_eq!(file.track(0).unwrap().events()[0].tick(), 0);
    }

    #[test]
    fn test_add_tracks_remove_empties_and_count_as_type1() {
        let mut file = MidiFile::new();
        file.add_tracks(3);
        assert_eq!(file.num_tracks(), 3);

        file.track_mut(0).unwrap().add_note(0, 480, 0, 60, 100);
        file.track_mut(1).unwrap().add_note(0, 480, 1, 64, 100);
        // track 2 stays empty

        assert_eq!(file.get_track_count_as_type1(), 3); // tempo track + ch0 + ch1

        file.remove_empties();
        assert_eq!(file.num_tracks(), 2);
    }

    #[test]
    fn test_merge_two_tracks() {
        let mut file = MidiFile::new();
        file.add_tracks(3);
        file.track_mut(0).unwrap().add_note(0, 480, 0, 60, 100);
        file.track_mut(1).unwrap().add_note(0, 480, 1, 64, 100);
        file.track_mut(2).unwrap().add_note(0, 480, 2, 67, 100);

        file.merge_two_tracks(0, 2).unwrap();

        assert_eq!(file.num_tracks(), 2);
        let channels: Vec<u8> = file
            .track(0)
            .unwrap()
            .events()
            .iter()
            .filter_map(|e| e.channel())
            .collect();
        assert!(channels.contains(&0));
        assert!(channels.contains(&2));
        // The untouched track (originally index 1) is still present.
        assert!(
            file.track(1)
                .unwrap()
                .events()
                .iter()
                .any(|e| e.channel() == Some(1))
        );
    }

    #[test]
    fn test_set_millisecond_ticks() {
        let mut file = MidiFile::new();
        file.set_ticks_per_quarter(480);
        let track = file.add_track();
        track.add_tempo(0, 120.0); // 500,000 us/quarter -> 500 ticks/quarter for 1ms ticks
        track.add_note(0, 480, 0, 60, 100); // originally exactly one quarter note

        file.set_millisecond_ticks();

        assert_eq!(file.ticks_per_quarter(), 500);
        // A quarter note at 120 BPM lasts 500ms, so the rescaled duration
        // should land at tick 500 (i.e. 500ms).
        let note_off_tick = file
            .track(0)
            .unwrap()
            .events()
            .iter()
            .find(|e| e.is_note_off())
            .unwrap()
            .tick();
        assert_eq!(note_off_tick, 500);
    }

    #[test]
    fn test_get_file_duration_in_quarters() {
        let mut file = MidiFile::new();
        file.set_ticks_per_quarter(480);
        let track = file.add_track();
        track.add_note(0, 480, 0, 60, 100);
        track.add_note(480, 960, 0, 64, 100); // ends at tick 1440

        assert!((file.get_file_duration_in_quarters() - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_midi_file_creation() {
        let file = MidiFile::new();
        assert_eq!(file.format(), MidiFormat::MultiTrack);
        assert_eq!(file.ticks_per_quarter(), 480);
        assert_eq!(file.num_tracks(), 0);
    }

    #[test]
    fn test_midi_file_add_track() {
        let mut file = MidiFile::new();
        let track = file.add_track();
        track.set_name("Piano");
        track.add_note(0, 480, 0, 60, 100);

        assert_eq!(file.num_tracks(), 1);
        assert_eq!(file.track(0).unwrap().name(), Some("Piano"));
    }

    #[test]
    fn test_midi_file_roundtrip() {
        let mut file = MidiFile::new();
        file.set_ticks_per_quarter(960);

        let track = file.add_track();
        track.add_tempo(0, 120.0);
        track.add_note(0, 480, 0, 60, 100);
        track.add_note(480, 480, 0, 64, 100);
        track.add_note(960, 480, 0, 67, 100);
        track.add_end_of_track();

        let bytes = file.to_bytes();
        let parsed = MidiFile::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.format(), file.format());
        assert_eq!(parsed.ticks_per_quarter(), file.ticks_per_quarter());
        assert_eq!(parsed.num_tracks(), file.num_tracks());
    }

    #[test]
    fn test_smpte_division_parsing() {
        // 25 fps, 40 subframes: top byte 0xE7 == -25 as i8, bottom byte 0x28 == 40.
        // Effective ticks-per-quarter = 25 * 40 = 1000 (matches upstream midifile's
        // frames_per_second * subframes convention). Before the fix this fell
        // through to the raw signed bit pattern (0xE728 = 59176) instead.
        let mut header = Vec::new();
        header.extend(b"MThd");
        header.extend(&6u32.to_be_bytes());
        header.extend(&0u16.to_be_bytes()); // format 0
        header.extend(&0u16.to_be_bytes()); // 0 tracks
        header.extend(&[0xE7, 0x28]); // SMPTE division: -25 fps, 40 subframes

        let parsed = MidiFile::from_bytes(&header).unwrap();
        assert_eq!(parsed.ticks_per_quarter(), 1000);
    }

    #[test]
    fn test_varlen() {
        // Test encoding
        assert_eq!(write_varlen(0), vec![0x00]);
        assert_eq!(write_varlen(127), vec![0x7F]);
        assert_eq!(write_varlen(128), vec![0x81, 0x00]);
        assert_eq!(write_varlen(16383), vec![0xFF, 0x7F]);

        // Test decoding
        assert_eq!(read_varlen(&[0x00]), Some((0, 1)));
        assert_eq!(read_varlen(&[0x7F]), Some((127, 1)));
        assert_eq!(read_varlen(&[0x81, 0x00]), Some((128, 2)));
    }

    #[test]
    fn test_time_conversion() {
        let mut file = MidiFile::new();
        file.set_ticks_per_quarter(480);

        let track = file.add_track();
        track.add_tempo(0, 90.0); // 60/90 = 0.6667 seconds per beat

        // At 90 BPM with 480 tpq:
        // 1 beat = 60/90 seconds = 480 ticks
        // So 1 tick = (60/90)/480 seconds

        let seconds = file.ticks_to_seconds(480);
        assert!((seconds - 60.0 / 90.0).abs() < 0.001);

        let ticks = file.seconds_to_ticks(60.0 / 90.0 * 2.0);
        assert!(
            (ticks as i64 - 960).abs() <= 1,
            "expected ~960 ticks, got {ticks}"
        );
    }

    #[test]
    fn test_time_conversion_with_tempo_change() {
        // Regression test for the bug where build_time_map computed tempo_events
        // but never stored them, so tick->seconds conversion silently ignored
        // every real tempo event and always assumed 120 BPM.
        let mut file = MidiFile::new();
        file.set_ticks_per_quarter(480);

        let track = file.add_track();
        track.add_tempo(0, 60.0); // 1 second per beat for the first 480 ticks
        track.add_tempo(480, 120.0); // then 0.5 seconds per beat

        // First beat at 60 BPM takes 1.0s.
        let seconds_at_480 = file.ticks_to_seconds(480);
        assert!(
            (seconds_at_480 - 1.0).abs() < 0.001,
            "expected ~1.0s at tick 480, got {seconds_at_480}"
        );

        // Second beat at 120 BPM takes another 0.5s, landing at 1.5s total.
        let seconds_at_960 = file.ticks_to_seconds(960);
        assert!(
            (seconds_at_960 - 1.5).abs() < 0.001,
            "expected ~1.5s at tick 960, got {seconds_at_960}"
        );
    }
}
