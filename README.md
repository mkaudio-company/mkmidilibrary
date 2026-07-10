[![](https://img.shields.io/crates/v/mkmidilibrary.svg)](https://crates.io/crates/mkmidilibrary)
[![](https://img.shields.io/crates/l/mkmidilibrary.svg)](https://crates.io/crates/mkmidilibrary)
[![](https://docs.rs/mkmidilibrary/badge.svg)](https://docs.rs/mkmidilibrary/)

# mkmidilibrary

A comprehensive Rust library for music notation, MIDI file I/O, and real-time MIDI communication.

## Overview

mkmidilibrary is a Rust translation and unification of three popular music libraries:
- **music21** (Python) → Music notation and analysis
- **midifile** (C++) → MIDI file reading/writing
- **rtmidi** (C++) → Real-time MIDI I/O

## Features

- **Core Music Primitives**: Pitch, Duration, Note, Rest, Chord, Interval, Scale, Unpitched
- **Stream Hierarchy**: Score, Part, Measure, Voice containers, with a full `music21`-style
  operation set (`flatten`/`recurse`, `chordify`/`implode`, `explode`/`voicesToParts`,
  `makeMeasures`/`makeTies`/`makeBeams`/`makeAccidentals`/`makeNotation`, `quantize`/`sliceBy*`,
  whole-container `transpose`/`augmentOrDiminish`, `expandRepeats`, and more)
- **MIDI File I/O**: Read and write Standard MIDI Files (SMF), including Format 0/1
  track join/split, tick-state conversion, and tempo-aware time mapping
- **Real-time MIDI**: MIDI input/output on macOS (CoreMIDI); Linux/Windows backends are stubs (see below)
- **Music Notation**: Clefs, key signatures, meter (including additive/summed time signatures and
  beat-hierarchy/beaming), dynamics (including Spanner-based hairpins), articulations, and
  ornament realization (trills/turns/mordents)
- **Score Rendering**: Graphical rendering via mkgraphic (optional)
- **Music Analysis**: Chord identification and Forte set-class labels, roman numeral analysis
  (parsing and pitch realization, both directions), 5 key-finding algorithms, floating-key/
  modulation tracking, chord reduction, and melodic analysis (ambitus, interval diversity)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mkmidilibrary = "0.2.0"
```

### Feature Flags

- `realtime` - Real-time MIDI I/O (enabled by default)
- `graphics` - Score rendering with mkgraphic (enabled by default)

To disable optional features:

```toml
[dependencies]
mkmidilibrary = { version = "0.2.0", default-features = false }
```

## Quick Start

### Creating Notes and Chords

```rust
use mkmidilibrary::prelude::*;

// Create a pitch (Middle C)
let pitch = Pitch::from_parts(Step::C, Some(4), None);

// Create notes with different durations
let quarter_note = Note::quarter(pitch.clone());
let half_note = Note::half(pitch.clone());
let dotted_quarter = Note::new(pitch.clone(), Duration::from_type(DurationType::Quarter, 1));

// Create a C major chord
let c_major = Chord::major_triad(Pitch::from_parts(Step::C, Some(4), None));
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
measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))));
measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(Step::E, Some(4), None))));
measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(Step::G, Some(4), None))));
measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(5), None))));

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
| macOS    | CoreMIDI (fully implemented and tested) |
| Linux    | ALSA (stub — port enumeration/open/send are no-ops; not yet implemented) |
| Windows  | Windows MM (stub — port enumeration/open/send are no-ops; not yet implemented) |

## Known Limitations / Out of Scope

A few upstream (music21/midifile/rtmidi) capabilities are intentionally not implemented, rather than silently missing:

- **Binasc** (midifile's ASCII&#8596;binary MIDI text format) is not implemented.
- **MTS tuning SysEx** (MIDI Tuning Standard bulk/single-note tuning dump builders) is not implemented.
- **ALSA/Windows MM backends** (`realtime::alsa_impl`/`realtime::winmm_impl`) are stubs — real-time MIDI I/O is only functional on macOS today. Extending these needs a Linux/Windows environment to verify against real hardware/drivers.
- **Forte set-class labels** (`Chord::forte_class`) only cover trichords and tetrachords (Forte 3-1..3-12, 4-1..4-Z29) — pentachords and larger return `None` rather than a hand-transcribed (and unverifiable in this environment) larger table.
- Several analysis/notation routines (`Score::chordify`, `analysis::discrete`'s key-finding profiles, `Part::best_time_signature`, etc.) are scoped, documented subsets of their music21 namesakes — see each function's doc comment for the specific simplification.

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
