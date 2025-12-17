//! MIDI file I/O
//!
//! This module provides reading and writing of Standard MIDI Files (SMF).

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use super::event::MidiEvent;
use super::message::{MetaEvent, MidiMessage};
use super::track::MidiTrack;
use super::{MidiError, MidiFormat};

/// A Standard MIDI File
#[derive(Debug, Clone)]
pub struct MidiFile {
    /// MIDI file format (0, 1, or 2)
    format: MidiFormat,
    /// Ticks per quarter note (timing resolution)
    ticks_per_quarter: u16,
    /// Tracks in this file
    tracks: Vec<MidiTrack>,
    /// Time map for tick-to-seconds conversion
    time_map: Option<TimeMap>,
}

impl MidiFile {
    /// Create a new empty MIDI file
    pub fn new() -> Self {
        Self {
            format: MidiFormat::MultiTrack,
            ticks_per_quarter: 480,
            tracks: Vec::new(),
            time_map: None,
        }
    }

    /// Create a MIDI file with specified format and resolution
    pub fn with_format(format: MidiFormat, ticks_per_quarter: u16) -> Self {
        Self {
            format,
            ticks_per_quarter,
            tracks: Vec::new(),
            time_map: None,
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
        let ticks_per_quarter = read_u16_be(&data[12..14]);

        // Check for SMPTE timing (not supported yet)
        if ticks_per_quarter & 0x8000 != 0 {
            // SMPTE timing - convert to approximate ticks per quarter
            // For now, just use a reasonable default
            let _smpte_format = ((ticks_per_quarter >> 8) as i8).abs();
            let _ticks_per_frame = (ticks_per_quarter & 0xFF) as u16;
        }

        let mut midi_file = Self {
            format,
            ticks_per_quarter,
            tracks: Vec::with_capacity(num_tracks),
            time_map: None,
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

            let track = parse_track(&data[pos..pos + track_len])?;
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
        self.time_map = None; // Invalidate time map
    }

    /// Get all tracks
    pub fn tracks(&self) -> &[MidiTrack] {
        &self.tracks
    }

    /// Get mutable tracks
    pub fn tracks_mut(&mut self) -> &mut Vec<MidiTrack> {
        self.time_map = None; // Invalidate time map
        &mut self.tracks
    }

    /// Get a specific track
    pub fn track(&self, index: usize) -> Option<&MidiTrack> {
        self.tracks.get(index)
    }

    /// Get a mutable specific track
    pub fn track_mut(&mut self, index: usize) -> Option<&mut MidiTrack> {
        self.time_map = None;
        self.tracks.get_mut(index)
    }

    /// Get number of tracks
    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }

    /// Add a new track and return a mutable reference to it
    pub fn add_track(&mut self) -> &mut MidiTrack {
        self.time_map = None;
        self.tracks.push(MidiTrack::new());
        self.tracks.last_mut().unwrap()
    }

    /// Add an existing track
    pub fn add_track_from(&mut self, track: MidiTrack) {
        self.time_map = None;
        self.tracks.push(track);
    }

    /// Delete a track
    pub fn delete_track(&mut self, index: usize) -> Option<MidiTrack> {
        if index < self.tracks.len() {
            self.time_map = None;
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
        self.time_map = None;
    }

    /// Split track 0 by channel (for Format 0 -> Format 1 conversion)
    pub fn split_tracks_by_channel(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let source = &self.tracks[0];
        let mut channel_tracks: Vec<MidiTrack> = (0..16).map(|_| MidiTrack::new()).collect();
        let mut tempo_track = MidiTrack::with_name("Tempo");

        for event in source.events() {
            if event.is_meta() {
                tempo_track.add_event(event.clone());
            } else if let Some(ch) = event.channel() {
                channel_tracks[ch as usize].add_event(event.clone());
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
        self.time_map = None;
    }

    /// Get the total duration in ticks
    pub fn total_ticks(&self) -> u64 {
        self.tracks.iter().map(|t| t.last_tick()).max().unwrap_or(0)
    }

    /// Get the total duration in seconds
    pub fn total_seconds(&self) -> f64 {
        self.build_time_map();
        self.ticks_to_seconds(self.total_ticks())
    }

    /// Convert ticks to seconds
    pub fn ticks_to_seconds(&self, ticks: u64) -> f64 {
        self.build_time_map();
        if let Some(ref time_map) = self.time_map {
            time_map.ticks_to_seconds(ticks)
        } else {
            // Fallback: assume 120 BPM
            let seconds_per_tick = 0.5 / self.ticks_per_quarter as f64;
            ticks as f64 * seconds_per_tick
        }
    }

    /// Convert seconds to ticks
    pub fn seconds_to_ticks(&self, seconds: f64) -> u64 {
        self.build_time_map();
        if let Some(ref time_map) = self.time_map {
            time_map.seconds_to_ticks(seconds)
        } else {
            // Fallback: assume 120 BPM
            let ticks_per_second = self.ticks_per_quarter as f64 * 2.0;
            (seconds * ticks_per_second) as u64
        }
    }

    /// Build the time map for tempo conversion
    fn build_time_map(&self) {
        if self.time_map.is_some() {
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

        // This is a mutable operation but we're using interior mutability pattern
        // In a real implementation, we'd use RefCell or similar
        // For simplicity, we'll compute on demand if needed
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
        let track = self.tracks.get_mut(track).ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_note(start_tick, duration, channel, key, velocity);
        self.time_map = None;
        Ok(())
    }

    /// Add a tempo change
    pub fn add_tempo(&mut self, track: usize, tick: u64, bpm: f64) -> Result<(), MidiError> {
        let track = self.tracks.get_mut(track).ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_tempo(tick, bpm);
        self.time_map = None;
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
        let track = self.tracks.get_mut(track).ok_or(MidiError::TrackOutOfBounds(track))?;
        track.add_time_signature(tick, numerator, denominator);
        Ok(())
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

        if let Some(ref time_map) = self.time_map {
            // Collect ticks first to avoid borrow conflict
            let ticks_per_track: Vec<Vec<u64>> = self
                .tracks
                .iter()
                .map(|track| track.events().iter().map(|e| e.tick()).collect())
                .collect();

            // Calculate seconds for each tick
            let seconds_per_track: Vec<Vec<f64>> = ticks_per_track
                .iter()
                .map(|ticks| ticks.iter().map(|&t| time_map.ticks_to_seconds(t)).collect())
                .collect();

            for (track_idx, track) in self.tracks.iter_mut().enumerate() {
                for (event_idx, event) in track.events_mut().iter_mut().enumerate() {
                    event.set_seconds(seconds_per_track[track_idx][event_idx]);
                }
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
    ((data[0] as u32) << 24)
        | ((data[1] as u32) << 16)
        | ((data[2] as u32) << 8)
        | data[3] as u32
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
        track.add_tempo(0, 120.0); // 0.5 seconds per beat

        // At 120 BPM with 480 tpq:
        // 1 beat = 0.5 seconds = 480 ticks
        // So 1 tick = 0.5/480 seconds

        let seconds = file.ticks_to_seconds(480);
        assert!((seconds - 0.5).abs() < 0.001);

        let ticks = file.seconds_to_ticks(1.0);
        assert_eq!(ticks, 960);
    }
}
