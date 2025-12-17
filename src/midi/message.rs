//! MIDI message types
//!
//! This module defines all MIDI message types including channel messages,
//! system messages, and meta events.

use std::fmt;

/// MIDI channel voice message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiMessage {
    /// Note Off event (key released)
    NoteOff {
        channel: u8,
        key: u8,
        velocity: u8,
    },
    /// Note On event (key pressed)
    NoteOn {
        channel: u8,
        key: u8,
        velocity: u8,
    },
    /// Polyphonic key pressure (aftertouch per note)
    PolyPressure {
        channel: u8,
        key: u8,
        pressure: u8,
    },
    /// Control change
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// Program change (instrument selection)
    ProgramChange {
        channel: u8,
        program: u8,
    },
    /// Channel pressure (aftertouch for whole channel)
    ChannelPressure {
        channel: u8,
        pressure: u8,
    },
    /// Pitch bend
    PitchBend {
        channel: u8,
        /// 14-bit value: 0x2000 = center, range 0x0000-0x3FFF
        value: u16,
    },
    /// System exclusive message
    SysEx(Vec<u8>),
    /// Meta event (in MIDI files only)
    Meta(MetaEvent),
    /// MIDI Time Code Quarter Frame
    MtcQuarterFrame(u8),
    /// Song Position Pointer
    SongPosition(u16),
    /// Song Select
    SongSelect(u8),
    /// Tune Request
    TuneRequest,
    /// Timing Clock
    TimingClock,
    /// Start
    Start,
    /// Continue
    Continue,
    /// Stop
    Stop,
    /// Active Sensing
    ActiveSensing,
    /// System Reset
    SystemReset,
}

impl MidiMessage {
    /// Create a Note On message
    pub fn note_on(channel: u8, key: u8, velocity: u8) -> Self {
        MidiMessage::NoteOn {
            channel: channel & 0x0F,
            key: key & 0x7F,
            velocity: velocity & 0x7F,
        }
    }

    /// Create a Note Off message
    pub fn note_off(channel: u8, key: u8, velocity: u8) -> Self {
        MidiMessage::NoteOff {
            channel: channel & 0x0F,
            key: key & 0x7F,
            velocity: velocity & 0x7F,
        }
    }

    /// Create a Control Change message
    pub fn control_change(channel: u8, controller: u8, value: u8) -> Self {
        MidiMessage::ControlChange {
            channel: channel & 0x0F,
            controller: controller & 0x7F,
            value: value & 0x7F,
        }
    }

    /// Create a Program Change message
    pub fn program_change(channel: u8, program: u8) -> Self {
        MidiMessage::ProgramChange {
            channel: channel & 0x0F,
            program: program & 0x7F,
        }
    }

    /// Create a Pitch Bend message
    pub fn pitch_bend(channel: u8, value: u16) -> Self {
        MidiMessage::PitchBend {
            channel: channel & 0x0F,
            value: value & 0x3FFF,
        }
    }

    /// Create a Pitch Bend message from signed value (-8192 to 8191)
    pub fn pitch_bend_signed(channel: u8, value: i16) -> Self {
        let unsigned = (value as i32 + 0x2000).clamp(0, 0x3FFF) as u16;
        Self::pitch_bend(channel, unsigned)
    }

    /// Get the channel for channel messages
    pub fn channel(&self) -> Option<u8> {
        match self {
            MidiMessage::NoteOff { channel, .. }
            | MidiMessage::NoteOn { channel, .. }
            | MidiMessage::PolyPressure { channel, .. }
            | MidiMessage::ControlChange { channel, .. }
            | MidiMessage::ProgramChange { channel, .. }
            | MidiMessage::ChannelPressure { channel, .. }
            | MidiMessage::PitchBend { channel, .. } => Some(*channel),
            _ => None,
        }
    }

    /// Check if this is a Note On message
    pub fn is_note_on(&self) -> bool {
        matches!(self, MidiMessage::NoteOn { velocity, .. } if *velocity > 0)
    }

    /// Check if this is a Note Off message (including Note On with velocity 0)
    pub fn is_note_off(&self) -> bool {
        matches!(
            self,
            MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { velocity: 0, .. }
        )
    }

    /// Check if this is a channel message
    pub fn is_channel_message(&self) -> bool {
        self.channel().is_some()
    }

    /// Check if this is a system message
    pub fn is_system_message(&self) -> bool {
        matches!(
            self,
            MidiMessage::SysEx(_)
                | MidiMessage::MtcQuarterFrame(_)
                | MidiMessage::SongPosition(_)
                | MidiMessage::SongSelect(_)
                | MidiMessage::TuneRequest
                | MidiMessage::TimingClock
                | MidiMessage::Start
                | MidiMessage::Continue
                | MidiMessage::Stop
                | MidiMessage::ActiveSensing
                | MidiMessage::SystemReset
        )
    }

    /// Check if this is a meta event
    pub fn is_meta(&self) -> bool {
        matches!(self, MidiMessage::Meta(_))
    }

    /// Get the status byte for this message
    pub fn status_byte(&self) -> Option<u8> {
        match self {
            MidiMessage::NoteOff { channel, .. } => Some(0x80 | channel),
            MidiMessage::NoteOn { channel, .. } => Some(0x90 | channel),
            MidiMessage::PolyPressure { channel, .. } => Some(0xA0 | channel),
            MidiMessage::ControlChange { channel, .. } => Some(0xB0 | channel),
            MidiMessage::ProgramChange { channel, .. } => Some(0xC0 | channel),
            MidiMessage::ChannelPressure { channel, .. } => Some(0xD0 | channel),
            MidiMessage::PitchBend { channel, .. } => Some(0xE0 | channel),
            MidiMessage::SysEx(_) => Some(0xF0),
            MidiMessage::MtcQuarterFrame(_) => Some(0xF1),
            MidiMessage::SongPosition(_) => Some(0xF2),
            MidiMessage::SongSelect(_) => Some(0xF3),
            MidiMessage::TuneRequest => Some(0xF6),
            MidiMessage::TimingClock => Some(0xF8),
            MidiMessage::Start => Some(0xFA),
            MidiMessage::Continue => Some(0xFB),
            MidiMessage::Stop => Some(0xFC),
            MidiMessage::ActiveSensing => Some(0xFE),
            MidiMessage::SystemReset => Some(0xFF),
            MidiMessage::Meta(_) => Some(0xFF), // Meta events use 0xFF prefix
        }
    }

    /// Convert to bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            MidiMessage::NoteOff {
                channel,
                key,
                velocity,
            } => vec![0x80 | channel, *key, *velocity],
            MidiMessage::NoteOn {
                channel,
                key,
                velocity,
            } => vec![0x90 | channel, *key, *velocity],
            MidiMessage::PolyPressure {
                channel,
                key,
                pressure,
            } => vec![0xA0 | channel, *key, *pressure],
            MidiMessage::ControlChange {
                channel,
                controller,
                value,
            } => vec![0xB0 | channel, *controller, *value],
            MidiMessage::ProgramChange { channel, program } => vec![0xC0 | channel, *program],
            MidiMessage::ChannelPressure { channel, pressure } => vec![0xD0 | channel, *pressure],
            MidiMessage::PitchBend { channel, value } => {
                vec![0xE0 | channel, (*value & 0x7F) as u8, (*value >> 7) as u8]
            }
            MidiMessage::SysEx(data) => {
                let mut bytes = vec![0xF0];
                bytes.extend(data);
                bytes.push(0xF7);
                bytes
            }
            MidiMessage::MtcQuarterFrame(data) => vec![0xF1, *data],
            MidiMessage::SongPosition(pos) => {
                vec![0xF2, (*pos & 0x7F) as u8, (*pos >> 7) as u8]
            }
            MidiMessage::SongSelect(song) => vec![0xF3, *song],
            MidiMessage::TuneRequest => vec![0xF6],
            MidiMessage::TimingClock => vec![0xF8],
            MidiMessage::Start => vec![0xFA],
            MidiMessage::Continue => vec![0xFB],
            MidiMessage::Stop => vec![0xFC],
            MidiMessage::ActiveSensing => vec![0xFE],
            MidiMessage::SystemReset => vec![0xFF],
            MidiMessage::Meta(meta) => meta.to_bytes(),
        }
    }

    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Option<(MidiMessage, usize)> {
        if data.is_empty() {
            return None;
        }

        let status = data[0];
        let channel = status & 0x0F;

        match status & 0xF0 {
            0x80 if data.len() >= 3 => Some((
                MidiMessage::NoteOff {
                    channel,
                    key: data[1],
                    velocity: data[2],
                },
                3,
            )),
            0x90 if data.len() >= 3 => Some((
                MidiMessage::NoteOn {
                    channel,
                    key: data[1],
                    velocity: data[2],
                },
                3,
            )),
            0xA0 if data.len() >= 3 => Some((
                MidiMessage::PolyPressure {
                    channel,
                    key: data[1],
                    pressure: data[2],
                },
                3,
            )),
            0xB0 if data.len() >= 3 => Some((
                MidiMessage::ControlChange {
                    channel,
                    controller: data[1],
                    value: data[2],
                },
                3,
            )),
            0xC0 if data.len() >= 2 => Some((
                MidiMessage::ProgramChange {
                    channel,
                    program: data[1],
                },
                2,
            )),
            0xD0 if data.len() >= 2 => Some((
                MidiMessage::ChannelPressure {
                    channel,
                    pressure: data[1],
                },
                2,
            )),
            0xE0 if data.len() >= 3 => Some((
                MidiMessage::PitchBend {
                    channel,
                    value: (data[1] as u16) | ((data[2] as u16) << 7),
                },
                3,
            )),
            0xF0 => match status {
                0xF0 => {
                    // SysEx - find end marker
                    if let Some(end) = data.iter().position(|&b| b == 0xF7) {
                        Some((MidiMessage::SysEx(data[1..end].to_vec()), end + 1))
                    } else {
                        None
                    }
                }
                0xF1 if data.len() >= 2 => Some((MidiMessage::MtcQuarterFrame(data[1]), 2)),
                0xF2 if data.len() >= 3 => Some((
                    MidiMessage::SongPosition((data[1] as u16) | ((data[2] as u16) << 7)),
                    3,
                )),
                0xF3 if data.len() >= 2 => Some((MidiMessage::SongSelect(data[1]), 2)),
                0xF6 => Some((MidiMessage::TuneRequest, 1)),
                0xF8 => Some((MidiMessage::TimingClock, 1)),
                0xFA => Some((MidiMessage::Start, 1)),
                0xFB => Some((MidiMessage::Continue, 1)),
                0xFC => Some((MidiMessage::Stop, 1)),
                0xFE => Some((MidiMessage::ActiveSensing, 1)),
                0xFF => {
                    // Could be system reset or meta event
                    if data.len() >= 2 && data[1] < 0x80 {
                        // Meta event
                        MetaEvent::from_bytes(&data[1..]).map(|(meta, len)| {
                            (MidiMessage::Meta(meta), len + 1)
                        })
                    } else {
                        Some((MidiMessage::SystemReset, 1))
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for MidiMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MidiMessage::NoteOff {
                channel,
                key,
                velocity,
            } => {
                write!(f, "NoteOff(ch={}, key={}, vel={})", channel, key, velocity)
            }
            MidiMessage::NoteOn {
                channel,
                key,
                velocity,
            } => {
                write!(f, "NoteOn(ch={}, key={}, vel={})", channel, key, velocity)
            }
            MidiMessage::ControlChange {
                channel,
                controller,
                value,
            } => write!(f, "CC(ch={}, cc={}, val={})", channel, controller, value),
            MidiMessage::ProgramChange { channel, program } => {
                write!(f, "PC(ch={}, prog={})", channel, program)
            }
            MidiMessage::PitchBend { channel, value } => {
                write!(f, "PitchBend(ch={}, val={})", channel, value)
            }
            MidiMessage::Meta(meta) => write!(f, "{}", meta),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// MIDI Meta Event (used in Standard MIDI Files)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaEvent {
    /// Sequence number
    SequenceNumber(u16),
    /// Text event
    Text(String),
    /// Copyright notice
    Copyright(String),
    /// Track/sequence name
    TrackName(String),
    /// Instrument name
    InstrumentName(String),
    /// Lyric
    Lyric(String),
    /// Marker
    Marker(String),
    /// Cue point
    CuePoint(String),
    /// Program name
    ProgramName(String),
    /// Device name
    DeviceName(String),
    /// MIDI channel prefix
    ChannelPrefix(u8),
    /// MIDI port
    MidiPort(u8),
    /// End of track
    EndOfTrack,
    /// Tempo in microseconds per quarter note
    Tempo(u32),
    /// SMPTE offset
    SmpteOffset {
        hours: u8,
        minutes: u8,
        seconds: u8,
        frames: u8,
        subframes: u8,
    },
    /// Time signature
    TimeSignature {
        numerator: u8,
        denominator_power: u8, // actual denominator = 2^denominator_power
        clocks_per_click: u8,
        notated_32nd_per_quarter: u8,
    },
    /// Key signature
    KeySignature {
        /// Negative = flats, positive = sharps
        sharps_flats: i8,
        /// 0 = major, 1 = minor
        minor: bool,
    },
    /// Sequencer-specific meta event
    SequencerSpecific(Vec<u8>),
    /// Unknown meta event
    Unknown { type_: u8, data: Vec<u8> },
}

impl MetaEvent {
    /// Create a tempo meta event from BPM
    pub fn tempo_from_bpm(bpm: f64) -> Self {
        let microseconds = (60_000_000.0 / bpm).round() as u32;
        MetaEvent::Tempo(microseconds)
    }

    /// Get BPM from tempo meta event
    pub fn tempo_to_bpm(&self) -> Option<f64> {
        if let MetaEvent::Tempo(us) = self {
            Some(60_000_000.0 / *us as f64)
        } else {
            None
        }
    }

    /// Create a time signature meta event
    pub fn time_signature(numerator: u8, denominator: u8) -> Self {
        // Convert denominator to power of 2
        let denominator_power = (denominator as f64).log2() as u8;
        MetaEvent::TimeSignature {
            numerator,
            denominator_power,
            clocks_per_click: 24,
            notated_32nd_per_quarter: 8,
        }
    }

    /// Get the actual denominator from time signature
    pub fn time_signature_denominator(&self) -> Option<u8> {
        if let MetaEvent::TimeSignature {
            denominator_power, ..
        } = self
        {
            Some(1 << denominator_power)
        } else {
            None
        }
    }

    /// Create a key signature meta event
    pub fn key_signature(sharps: i8, minor: bool) -> Self {
        MetaEvent::KeySignature {
            sharps_flats: sharps,
            minor,
        }
    }

    /// Get the meta event type byte
    pub fn type_byte(&self) -> u8 {
        match self {
            MetaEvent::SequenceNumber(_) => 0x00,
            MetaEvent::Text(_) => 0x01,
            MetaEvent::Copyright(_) => 0x02,
            MetaEvent::TrackName(_) => 0x03,
            MetaEvent::InstrumentName(_) => 0x04,
            MetaEvent::Lyric(_) => 0x05,
            MetaEvent::Marker(_) => 0x06,
            MetaEvent::CuePoint(_) => 0x07,
            MetaEvent::ProgramName(_) => 0x08,
            MetaEvent::DeviceName(_) => 0x09,
            MetaEvent::ChannelPrefix(_) => 0x20,
            MetaEvent::MidiPort(_) => 0x21,
            MetaEvent::EndOfTrack => 0x2F,
            MetaEvent::Tempo(_) => 0x51,
            MetaEvent::SmpteOffset { .. } => 0x54,
            MetaEvent::TimeSignature { .. } => 0x58,
            MetaEvent::KeySignature { .. } => 0x59,
            MetaEvent::SequencerSpecific(_) => 0x7F,
            MetaEvent::Unknown { type_, .. } => *type_,
        }
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0xFF, self.type_byte()];

        let data: Vec<u8> = match self {
            MetaEvent::SequenceNumber(n) => vec![(*n >> 8) as u8, *n as u8],
            MetaEvent::Text(s)
            | MetaEvent::Copyright(s)
            | MetaEvent::TrackName(s)
            | MetaEvent::InstrumentName(s)
            | MetaEvent::Lyric(s)
            | MetaEvent::Marker(s)
            | MetaEvent::CuePoint(s)
            | MetaEvent::ProgramName(s)
            | MetaEvent::DeviceName(s) => s.as_bytes().to_vec(),
            MetaEvent::ChannelPrefix(ch) => vec![*ch],
            MetaEvent::MidiPort(port) => vec![*port],
            MetaEvent::EndOfTrack => vec![],
            MetaEvent::Tempo(us) => vec![(*us >> 16) as u8, (*us >> 8) as u8, *us as u8],
            MetaEvent::SmpteOffset {
                hours,
                minutes,
                seconds,
                frames,
                subframes,
            } => vec![*hours, *minutes, *seconds, *frames, *subframes],
            MetaEvent::TimeSignature {
                numerator,
                denominator_power,
                clocks_per_click,
                notated_32nd_per_quarter,
            } => vec![
                *numerator,
                *denominator_power,
                *clocks_per_click,
                *notated_32nd_per_quarter,
            ],
            MetaEvent::KeySignature {
                sharps_flats,
                minor,
            } => vec![*sharps_flats as u8, if *minor { 1 } else { 0 }],
            MetaEvent::SequencerSpecific(data) | MetaEvent::Unknown { data, .. } => data.clone(),
        };

        // Write variable-length quantity for data length
        bytes.extend(Self::encode_varlen(data.len() as u32));
        bytes.extend(data);
        bytes
    }

    /// Parse from bytes (starting after 0xFF prefix)
    pub fn from_bytes(data: &[u8]) -> Option<(MetaEvent, usize)> {
        if data.is_empty() {
            return None;
        }

        let type_byte = data[0];
        let (length, varlen_size) = Self::decode_varlen(&data[1..])?;
        let data_start = 1 + varlen_size;
        let data_end = data_start + length as usize;

        if data.len() < data_end {
            return None;
        }

        let event_data = &data[data_start..data_end];

        let event = match type_byte {
            0x00 if event_data.len() >= 2 => {
                MetaEvent::SequenceNumber(((event_data[0] as u16) << 8) | event_data[1] as u16)
            }
            0x01 => MetaEvent::Text(String::from_utf8_lossy(event_data).to_string()),
            0x02 => MetaEvent::Copyright(String::from_utf8_lossy(event_data).to_string()),
            0x03 => MetaEvent::TrackName(String::from_utf8_lossy(event_data).to_string()),
            0x04 => MetaEvent::InstrumentName(String::from_utf8_lossy(event_data).to_string()),
            0x05 => MetaEvent::Lyric(String::from_utf8_lossy(event_data).to_string()),
            0x06 => MetaEvent::Marker(String::from_utf8_lossy(event_data).to_string()),
            0x07 => MetaEvent::CuePoint(String::from_utf8_lossy(event_data).to_string()),
            0x08 => MetaEvent::ProgramName(String::from_utf8_lossy(event_data).to_string()),
            0x09 => MetaEvent::DeviceName(String::from_utf8_lossy(event_data).to_string()),
            0x20 if !event_data.is_empty() => MetaEvent::ChannelPrefix(event_data[0]),
            0x21 if !event_data.is_empty() => MetaEvent::MidiPort(event_data[0]),
            0x2F => MetaEvent::EndOfTrack,
            0x51 if event_data.len() >= 3 => MetaEvent::Tempo(
                ((event_data[0] as u32) << 16)
                    | ((event_data[1] as u32) << 8)
                    | event_data[2] as u32,
            ),
            0x54 if event_data.len() >= 5 => MetaEvent::SmpteOffset {
                hours: event_data[0],
                minutes: event_data[1],
                seconds: event_data[2],
                frames: event_data[3],
                subframes: event_data[4],
            },
            0x58 if event_data.len() >= 4 => MetaEvent::TimeSignature {
                numerator: event_data[0],
                denominator_power: event_data[1],
                clocks_per_click: event_data[2],
                notated_32nd_per_quarter: event_data[3],
            },
            0x59 if event_data.len() >= 2 => MetaEvent::KeySignature {
                sharps_flats: event_data[0] as i8,
                minor: event_data[1] != 0,
            },
            0x7F => MetaEvent::SequencerSpecific(event_data.to_vec()),
            _ => MetaEvent::Unknown {
                type_: type_byte,
                data: event_data.to_vec(),
            },
        };

        Some((event, data_end))
    }

    /// Encode a variable-length quantity
    fn encode_varlen(mut value: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push((value & 0x7F) as u8);
        value >>= 7;

        while value > 0 {
            bytes.push(((value & 0x7F) | 0x80) as u8);
            value >>= 7;
        }

        bytes.reverse();
        bytes
    }

    /// Decode a variable-length quantity
    fn decode_varlen(data: &[u8]) -> Option<(u32, usize)> {
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
}

impl fmt::Display for MetaEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaEvent::Tempo(us) => {
                let bpm = 60_000_000.0 / *us as f64;
                write!(f, "Tempo({:.2} BPM)", bpm)
            }
            MetaEvent::TimeSignature {
                numerator,
                denominator_power,
                ..
            } => {
                let denom = 1 << denominator_power;
                write!(f, "TimeSignature({}/{})", numerator, denom)
            }
            MetaEvent::KeySignature {
                sharps_flats,
                minor,
            } => {
                let mode = if *minor { "minor" } else { "major" };
                write!(f, "KeySignature({} {})", sharps_flats, mode)
            }
            MetaEvent::TrackName(name) => write!(f, "TrackName({})", name),
            MetaEvent::EndOfTrack => write!(f, "EndOfTrack"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_off() {
        let on = MidiMessage::note_on(0, 60, 100);
        assert!(on.is_note_on());
        assert!(!on.is_note_off());
        assert_eq!(on.channel(), Some(0));

        let off = MidiMessage::note_off(0, 60, 0);
        assert!(off.is_note_off());
        assert!(!off.is_note_on());
    }

    #[test]
    fn test_message_to_bytes() {
        let on = MidiMessage::note_on(0, 60, 100);
        let bytes = on.to_bytes();
        assert_eq!(bytes, vec![0x90, 60, 100]);

        let (parsed, _) = MidiMessage::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, on);
    }

    #[test]
    fn test_pitch_bend() {
        let center = MidiMessage::pitch_bend(0, 0x2000);
        assert_eq!(center.channel(), Some(0));

        if let MidiMessage::PitchBend { value, .. } = center {
            assert_eq!(value, 0x2000);
        }

        let signed = MidiMessage::pitch_bend_signed(0, 0);
        if let MidiMessage::PitchBend { value, .. } = signed {
            assert_eq!(value, 0x2000);
        }
    }

    #[test]
    fn test_tempo_meta() {
        let tempo = MetaEvent::tempo_from_bpm(120.0);
        if let MetaEvent::Tempo(us) = tempo {
            assert_eq!(us, 500_000);
        }

        assert!((tempo.tempo_to_bpm().unwrap() - 120.0).abs() < 0.01);
    }

    #[test]
    fn test_time_signature_meta() {
        let ts = MetaEvent::time_signature(4, 4);
        assert_eq!(ts.time_signature_denominator(), Some(4));

        let ts = MetaEvent::time_signature(6, 8);
        assert_eq!(ts.time_signature_denominator(), Some(8));
    }

    #[test]
    fn test_meta_event_roundtrip() {
        let events = vec![
            MetaEvent::TrackName("Test Track".to_string()),
            MetaEvent::Tempo(500_000),
            MetaEvent::time_signature(4, 4),
            MetaEvent::key_signature(0, false),
            MetaEvent::EndOfTrack,
        ];

        for event in events {
            let bytes = event.to_bytes();
            let (parsed, _) = MetaEvent::from_bytes(&bytes[1..]).unwrap();
            assert_eq!(parsed, event);
        }
    }
}
