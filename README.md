# mkmidilibrary

A comprehensive Rust library for music notation, MIDI file I/O, and real-time MIDI communication.

## Overview

mkmidilibrary is a Rust translation and unification of three popular music libraries:
- **music21** (Python) → Music notation and analysis
- **midifile** (C++) → MIDI file reading/writing
- **rtmidi** (C++) → Real-time MIDI I/O

## Features

- **Core Music Primitives**: Pitch, Duration, Note, Rest, Chord, Interval
- **Stream Hierarchy**: Score, Part, Measure, Voice containers
- **MIDI File I/O**: Read and write Standard MIDI Files (SMF)
- **Real-time MIDI**: Cross-platform MIDI input/output (macOS, Linux, Windows)
- **Music Notation**: Clefs, key signatures, time signatures, dynamics, articulations
- **Score Rendering**: Graphical rendering via mkgraphic (optional)
- **Music Analysis**: Chord identification, roman numeral analysis

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mkmidilibrary = "0.1"
```

### Feature Flags

- `realtime` - Real-time MIDI I/O (enabled by default)
- `graphics` - Score rendering with mkgraphic (enabled by default)

To disable optional features:

```toml
[dependencies]
mkmidilibrary = { version = "0.1", default-features = false }
```

## Quick Start

### Creating Notes and Chords

```rust
use mkmidilibrary::prelude::*;

// Create a pitch (Middle C)
let pitch = Pitch::new(Step::C, 4);

// Create notes with different durations
let quarter_note = Note::quarter(pitch);
let half_note = Note::half(pitch);
let dotted_quarter = Note::dotted_quarter(pitch);

// Create a C major chord
let c_major = Chord::major_triad(Pitch::new(Step::C, 4));
```

### Building a Score

```rust
use mkmidilibrary::prelude::*;

// Create a new score
let mut score = Score::new();
score.set_title("My Composition");

// Create a part
let mut part = Part::with_name("Piano");

// Create a measure with time signature
let mut measure = Measure::new(1);
measure.set_time_signature(TimeSignature::new(4, 4));

// Add notes
measure.append(MusicElement::Note(Note::quarter(Pitch::new(Step::C, 4))));
measure.append(MusicElement::Note(Note::quarter(Pitch::new(Step::E, 4))));
measure.append(MusicElement::Note(Note::quarter(Pitch::new(Step::G, 4))));
measure.append(MusicElement::Note(Note::quarter(Pitch::new(Step::C, 5))));

part.add_measure(measure);
score.add_part(part);
```

### MIDI File I/O

```rust
use mkmidilibrary::midi::{MidiFile, MidiMessage};

// Read a MIDI file
let midi = MidiFile::read("song.mid")?;
println!("Tracks: {}", midi.num_tracks());
println!("Ticks per quarter: {}", midi.ticks_per_quarter());

// Create a new MIDI file
let mut midi = MidiFile::new();
midi.set_ticks_per_quarter(480);

let track = midi.add_track();
track.add_note(0, 480, 0, 60, 100); // C4 quarter note
track.add_note(480, 480, 0, 64, 100); // E4 quarter note

midi.write("output.mid")?;
```

### Real-time MIDI

```rust
use mkmidilibrary::realtime::{MidiInput, MidiOutput};

// List available ports
let input = MidiInput::new("My App")?;
for port in input.ports() {
    println!("Input: {}", port.name());
}

let mut output = MidiOutput::new("My App")?;
for port in output.ports() {
    println!("Output: {}", port.name());
}

// Open a port and send a note
output.open_port(0, "Out")?;
output.send_message(&[0x90, 60, 100])?; // Note On
output.send_message(&[0x80, 60, 0])?;   // Note Off
```

### Score Rendering

```rust
use mkmidilibrary::render::{ScoreRenderer, RenderConfig, render_score_to_image};

// Create a renderer
let renderer = ScoreRenderer::new();

// Render to PNG
let config = RenderConfig::default();
if let Some(png_data) = render_score_to_image(&score, &config) {
    std::fs::write("score.png", png_data)?;
}
```

## Module Structure

```
mkmidilibrary/
├── core/           # Music primitives (Pitch, Duration, Note, etc.)
├── stream/         # Container hierarchy (Score, Part, Measure, Voice)
├── midi/           # MIDI file I/O and translation
├── realtime/       # Real-time MIDI I/O (platform-specific)
├── notation/       # Musical notation elements
├── render/         # Graphical rendering (requires "graphics" feature)
└── analysis/       # Music analysis tools
```

## Platform Support

| Platform | Real-time MIDI Backend |
|----------|----------------------|
| macOS    | CoreMIDI             |
| Linux    | ALSA                 |
| Windows  | Windows MM           |

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
