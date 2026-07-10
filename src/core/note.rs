//! Note representation
//!
//! A Note combines a Pitch with a Duration and includes additional
//! notation properties like ties, articulations, and lyrics.

use std::cmp::Ordering;
use std::fmt;

use super::{Duration, Fraction, Interval, Pitch};

/// Tie type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TieType {
    /// Start of a tie
    Start,
    /// Continuation of a tie
    Continue,
    /// End of a tie
    Stop,
    /// Let ring (no explicit end)
    LetRing,
}

/// A tie connecting notes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tie {
    /// Type of tie
    pub type_: TieType,
    /// Tie placement (above/below)
    pub placement: Option<Placement>,
}

impl Tie {
    /// Create a new tie
    pub fn new(type_: TieType) -> Self {
        Self {
            type_,
            placement: None,
        }
    }

    /// Create a start tie
    pub fn start() -> Self {
        Self::new(TieType::Start)
    }

    /// Create a stop tie
    pub fn stop() -> Self {
        Self::new(TieType::Stop)
    }
}

/// Placement for notation elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    Above,
    Below,
}

/// Stem direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StemDirection {
    #[default]
    Auto,
    Up,
    Down,
    None,
}

/// Notehead type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NoteHeadType {
    #[default]
    Normal,
    Diamond,
    Square,
    Triangle,
    Slash,
    Cross,
    X,
    CircleX,
    Arrow,
    Cluster,
    None,
}

/// Notehead properties
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NoteHead {
    /// Notehead type
    pub type_: NoteHeadType,
    /// Whether the notehead is filled
    pub filled: Option<bool>,
    /// Whether the notehead has parentheses
    pub parenthesis: bool,
}

impl NoteHead {
    /// Create a new normal notehead
    pub fn normal() -> Self {
        Self::default()
    }

    /// Create a diamond notehead
    pub fn diamond() -> Self {
        Self {
            type_: NoteHeadType::Diamond,
            ..Default::default()
        }
    }

    /// Create an X notehead
    pub fn x() -> Self {
        Self {
            type_: NoteHeadType::X,
            ..Default::default()
        }
    }
}

/// Type of articulation mark. This is the single, unified articulation
/// enum for the whole crate: it used to be duplicated as a poorer
/// `core::note::ArticulationType` (used by `Note`/`Articulation`) and a
/// richer `notation::articulation::ArticulationMark` (with symbols,
/// velocity/duration multipliers, and fermata-shape variants) that nothing
/// actually attached to a `Note` could reach. `notation::articulation`
/// re-exports this type for backward-compatible access from that module
/// path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArticulationMark {
    /// Accent (>)
    Accent,
    /// Strong accent (^)
    StrongAccent,
    /// Staccato (.)
    Staccato,
    /// Staccatissimo (wedge)
    Staccatissimo,
    /// Tenuto (-)
    Tenuto,
    /// Detached legato (tenuto + staccato)
    DetachedLegato,
    /// Marcato (^)
    Marcato,
    /// Fermata
    Fermata,
    /// Short fermata
    ShortFermata,
    /// Long fermata
    LongFermata,
    /// Breath mark
    BreathMark,
    /// Caesura
    Caesura,
    /// Up bow (string)
    UpBow,
    /// Down bow (string)
    DownBow,
    /// Harmonic
    Harmonic,
    /// Open string
    OpenString,
    /// Stopped (brass)
    Stopped,
    /// Pizzicato
    Pizzicato,
    /// Snap pizzicato
    SnapPizzicato,
    /// Thumb position
    ThumbPosition,
    /// Pluck (guitar)
    Pluck,
    /// Double tongue
    DoubleTongue,
    /// Triple tongue
    TripleTongue,
    /// Heel (organ pedal)
    Heel,
    /// Toe (organ pedal)
    Toe,
    /// Spiccato (bounced bow)
    Spiccato,
    /// Scoop (slide into a note from below)
    Scoop,
    /// Plop (slide into a note from above)
    Plop,
    /// Doit (slide up and away after a note)
    Doit,
    /// Falloff (slide down and away after a note)
    Falloff,
    /// Stress (early-music metric-accent mark)
    Stress,
    /// Unstress (early-music de-emphasis mark)
    Unstress,
    /// Natural string harmonic (distinct from the generic `Harmonic`,
    /// which doesn't specify the production technique)
    StringHarmonic,
    /// Fret position indication for a fretted instrument (fret number)
    Fret(u8),
    /// String number indication for a fretted/bowed-string instrument
    StringNumber(u8),
    /// Handbell technique indication (e.g. martellato, mallet, damp) —
    /// simplified to a single generic marker rather than a full handbell
    /// technique taxonomy.
    HandbellIndication,
    /// Harp fingernails (pluck with fingernails for a metallic timbre)
    HarpFingerNails,
    /// Generic woodwind special-technique indication (e.g.
    /// flutter-tongue, half-hole) — simplified to a single marker rather
    /// than a full per-technique taxonomy.
    WoodwindIndication,
    /// Generic brass special-technique indication (e.g. muted, flutter,
    /// half-valve) — simplified to a single marker rather than a full
    /// per-technique taxonomy.
    BrassIndication,
}

impl ArticulationMark {
    /// Get the symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            ArticulationMark::Accent => ">",
            ArticulationMark::StrongAccent => "^",
            ArticulationMark::Staccato => ".",
            ArticulationMark::Staccatissimo => "▼",
            ArticulationMark::Tenuto => "-",
            ArticulationMark::DetachedLegato => "-.",
            ArticulationMark::Marcato => "^",
            ArticulationMark::Fermata => "𝄐",
            ArticulationMark::ShortFermata => "𝄑",
            ArticulationMark::LongFermata => "𝄒",
            ArticulationMark::BreathMark => ",",
            ArticulationMark::Caesura => "//",
            ArticulationMark::UpBow => "∨",
            ArticulationMark::DownBow => "∏",
            ArticulationMark::Harmonic => "○",
            ArticulationMark::OpenString => "○",
            ArticulationMark::Stopped => "+",
            ArticulationMark::Pizzicato => "+",
            ArticulationMark::SnapPizzicato => "⊙",
            ArticulationMark::ThumbPosition => "◯",
            ArticulationMark::Pluck => "i",
            ArticulationMark::DoubleTongue => "‥",
            ArticulationMark::TripleTongue => "…",
            ArticulationMark::Heel => "U",
            ArticulationMark::Toe => "^",
            ArticulationMark::Spiccato => "'",
            ArticulationMark::Scoop => "⌒",
            ArticulationMark::Plop => "⌒",
            ArticulationMark::Doit => "⌒",
            ArticulationMark::Falloff => "⌒",
            ArticulationMark::Stress => "▵",
            ArticulationMark::Unstress => "▿",
            ArticulationMark::StringHarmonic => "◇",
            ArticulationMark::Fret(_) => "fr.",
            ArticulationMark::StringNumber(_) => "#",
            ArticulationMark::HandbellIndication => "hb.",
            ArticulationMark::HarpFingerNails => "n.",
            ArticulationMark::WoodwindIndication => "ww.",
            ArticulationMark::BrassIndication => "br.",
        }
    }

    /// Get the name
    pub fn name(&self) -> &'static str {
        match self {
            ArticulationMark::Accent => "accent",
            ArticulationMark::StrongAccent => "strong accent",
            ArticulationMark::Staccato => "staccato",
            ArticulationMark::Staccatissimo => "staccatissimo",
            ArticulationMark::Tenuto => "tenuto",
            ArticulationMark::DetachedLegato => "detached legato",
            ArticulationMark::Marcato => "marcato",
            ArticulationMark::Fermata => "fermata",
            ArticulationMark::ShortFermata => "short fermata",
            ArticulationMark::LongFermata => "long fermata",
            ArticulationMark::BreathMark => "breath mark",
            ArticulationMark::Caesura => "caesura",
            ArticulationMark::UpBow => "up bow",
            ArticulationMark::DownBow => "down bow",
            ArticulationMark::Harmonic => "harmonic",
            ArticulationMark::OpenString => "open string",
            ArticulationMark::Stopped => "stopped",
            ArticulationMark::Pizzicato => "pizzicato",
            ArticulationMark::SnapPizzicato => "snap pizzicato",
            ArticulationMark::ThumbPosition => "thumb position",
            ArticulationMark::Pluck => "pluck",
            ArticulationMark::DoubleTongue => "double tongue",
            ArticulationMark::TripleTongue => "triple tongue",
            ArticulationMark::Heel => "heel",
            ArticulationMark::Toe => "toe",
            ArticulationMark::Spiccato => "spiccato",
            ArticulationMark::Scoop => "scoop",
            ArticulationMark::Plop => "plop",
            ArticulationMark::Doit => "doit",
            ArticulationMark::Falloff => "falloff",
            ArticulationMark::Stress => "stress",
            ArticulationMark::Unstress => "unstress",
            ArticulationMark::StringHarmonic => "string harmonic",
            ArticulationMark::Fret(_) => "fret",
            ArticulationMark::StringNumber(_) => "string number",
            ArticulationMark::HandbellIndication => "handbell indication",
            ArticulationMark::HarpFingerNails => "harp fingernails",
            ArticulationMark::WoodwindIndication => "woodwind indication",
            ArticulationMark::BrassIndication => "brass indication",
        }
    }

    /// The fret number, for a `Fret` marking.
    pub fn fret_number(&self) -> Option<u8> {
        match self {
            ArticulationMark::Fret(n) => Some(*n),
            _ => None,
        }
    }

    /// The string number, for a `StringNumber` marking.
    pub fn string_number(&self) -> Option<u8> {
        match self {
            ArticulationMark::StringNumber(n) => Some(*n),
            _ => None,
        }
    }

    /// Additive velocity shift (added directly to the base MIDI velocity,
    /// unlike `velocity_multiplier`'s proportional scaling), used
    /// alongside the multiplicative model rather than replacing it.
    /// Mirrors music21's `Accent`, whose accent effect is modeled
    /// additively — a fixed boost sounds musically correct at both a
    /// quiet base dynamic (where a multiplier alone barely moves the
    /// needle) and a loud one (where a multiplier alone can overshoot).
    pub fn volume_shift(&self) -> i8 {
        match self {
            ArticulationMark::Accent => 16,
            ArticulationMark::StrongAccent | ArticulationMark::Marcato => 24,
            _ => 0,
        }
    }

    /// Get the velocity multiplier used by `Volume::get_realized`
    pub fn velocity_multiplier(&self) -> f64 {
        match self {
            ArticulationMark::Accent => 1.2,
            ArticulationMark::StrongAccent | ArticulationMark::Marcato => 1.4,
            ArticulationMark::Staccato => 0.9,
            ArticulationMark::Staccatissimo => 1.1,
            ArticulationMark::Tenuto => 1.0,
            ArticulationMark::DetachedLegato => 0.95,
            _ => 1.0,
        }
    }

    /// Get the duration multiplier used by `Note::realized_quarter_length`
    pub fn duration_multiplier(&self) -> f64 {
        match self {
            ArticulationMark::Staccato => 0.5,
            ArticulationMark::Staccatissimo => 0.25,
            ArticulationMark::Tenuto => 1.0,
            ArticulationMark::DetachedLegato => 0.75,
            _ => 1.0,
        }
    }

    /// Check if this affects note duration
    pub fn affects_duration(&self) -> bool {
        matches!(
            self,
            ArticulationMark::Staccato
                | ArticulationMark::Staccatissimo
                | ArticulationMark::DetachedLegato
        )
    }

    /// Check if this is a fermata type
    pub fn is_fermata(&self) -> bool {
        matches!(
            self,
            ArticulationMark::Fermata
                | ArticulationMark::ShortFermata
                | ArticulationMark::LongFermata
        )
    }
}

impl fmt::Display for ArticulationMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// An articulation marking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Articulation {
    /// Articulation type
    pub type_: ArticulationMark,
    /// Placement (above/below)
    pub placement: Option<Placement>,
}

impl Articulation {
    /// Create a new articulation
    pub fn new(type_: ArticulationMark) -> Self {
        Self {
            type_,
            placement: None,
        }
    }

    /// Create a staccato
    pub fn staccato() -> Self {
        Self::new(ArticulationMark::Staccato)
    }

    /// Create an accent
    pub fn accent() -> Self {
        Self::new(ArticulationMark::Accent)
    }

    /// Create a tenuto
    pub fn tenuto() -> Self {
        Self::new(ArticulationMark::Tenuto)
    }

    /// Create a fermata
    pub fn fermata() -> Self {
        Self::new(ArticulationMark::Fermata)
    }
}

/// Expression type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionType {
    Trill,
    Turn,
    InvertedTurn,
    Mordent,
    InvertedMordent,
    Tremolo,
    Vibrato,
    Glissando,
    Slide,
    ArpeggioUp,
    ArpeggioDown,
}

/// An expression marking
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// Expression type
    pub type_: ExpressionType,
    /// Placement (above/below)
    pub placement: Option<Placement>,
}

impl Expression {
    /// Create a new expression
    pub fn new(type_: ExpressionType) -> Self {
        Self {
            type_,
            placement: None,
        }
    }

    /// Create a trill
    pub fn trill() -> Self {
        Self::new(ExpressionType::Trill)
    }

    /// Create a mordent
    pub fn mordent() -> Self {
        Self::new(ExpressionType::Mordent)
    }
}

/// Syllabic type for lyrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Syllabic {
    #[default]
    Single,
    Begin,
    Middle,
    End,
}

/// A lyric syllable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lyric {
    /// The lyric text
    pub text: String,
    /// Syllabic type
    pub syllabic: Syllabic,
    /// Verse number (1-indexed)
    pub number: u8,
    /// Optional verse identifier/name (distinct from the numeric
    /// `number`), e.g. "chorus" or "refrain".
    pub identifier: Option<String>,
}

impl Lyric {
    /// Create a new lyric
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            syllabic: Syllabic::Single,
            number: 1,
            identifier: None,
        }
    }

    /// Set the syllabic type
    pub fn with_syllabic(mut self, syllabic: Syllabic) -> Self {
        self.syllabic = syllabic;
        self
    }

    /// Set the verse number
    pub fn with_number(mut self, number: u8) -> Self {
        self.number = number;
        self
    }

    /// Set the verse identifier/name.
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Get the verse identifier/name.
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Parse raw hyphenated lyric text into one or more syllable-tagged
    /// `Lyric`s, e.g. `"con-tra-ry"` -> `[Begin("con"), Middle("tra"),
    /// End("ry")]`. Text with no hyphens produces a single
    /// `Syllabic::Single` lyric. Mirrors music21's `Lyric.setTextAndSyllabic`
    /// raw-text auto-syllabification.
    pub fn from_raw_text(raw_text: &str, number: u8) -> Vec<Lyric> {
        let parts: Vec<&str> = raw_text.split('-').collect();
        if parts.len() == 1 {
            vec![Lyric::new(raw_text).with_number(number)]
        } else {
            let last = parts.len() - 1;
            parts
                .into_iter()
                .enumerate()
                .map(|(i, part)| {
                    let syllabic = if i == 0 {
                        Syllabic::Begin
                    } else if i == last {
                        Syllabic::End
                    } else {
                        Syllabic::Middle
                    };
                    Lyric::new(part).with_syllabic(syllabic).with_number(number)
                })
                .collect()
        }
    }
}

/// Whether a set of lyrics (e.g. as produced by `Lyric::from_raw_text`)
/// represents a composite (multi-syllable) lyric line rather than a single
/// whole-word lyric.
pub fn is_composite_lyric_set(lyrics: &[Lyric]) -> bool {
    lyrics.len() > 1
}

/// Volume/velocity information
#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    /// MIDI velocity (0-127)
    pub velocity: u8,
    /// Volume scalar (0.0-1.0)
    pub realized_volume: f64,
}

impl Volume {
    /// Create from MIDI velocity
    pub fn from_velocity(velocity: u8) -> Self {
        Self {
            velocity,
            realized_volume: velocity as f64 / 127.0,
        }
    }

    /// Create from scalar (0.0-1.0)
    pub fn from_scalar(scalar: f64) -> Self {
        Self {
            velocity: (scalar.clamp(0.0, 1.0) * 127.0) as u8,
            realized_volume: scalar.clamp(0.0, 1.0),
        }
    }

    /// Default mezzo-forte velocity
    pub fn mf() -> Self {
        Self::from_velocity(80)
    }

    /// Compute the realized (effective) volume after applying the given
    /// articulations' velocity multipliers (e.g. an accent boosts velocity).
    /// Multipliers compound multiplicatively across all attached
    /// articulations and the result is clamped to the valid MIDI velocity
    /// range. Use `Note::realized_volume` to apply a note's own
    /// articulations automatically.
    pub fn get_realized(&self, articulations: &[Articulation]) -> Volume {
        let multiplier: f64 = articulations
            .iter()
            .map(|a| a.type_.velocity_multiplier())
            .product();
        let shift: i32 = articulations.iter().map(|a| a.type_.volume_shift() as i32).sum();
        let shifted = (self.velocity as f64 * multiplier).round() as i32 + shift;
        Volume::from_velocity(shifted.clamp(0, 127) as u8)
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::mf()
    }
}

/// A musical note (pitch + duration)
#[derive(Debug, Clone)]
pub struct Note {
    /// The pitch
    pitch: Pitch,
    /// The duration
    duration: Duration,
    /// Offset within the stream (in quarter lengths)
    offset: Fraction,
    /// Tie information
    tie: Option<Tie>,
    /// Lyrics
    lyrics: Vec<Lyric>,
    /// Articulations
    articulations: Vec<Articulation>,
    /// Expressions
    expressions: Vec<Expression>,
    /// Volume/velocity
    volume: Volume,
    /// Notehead
    notehead: NoteHead,
    /// Stem direction
    stem_direction: StemDirection,
    /// Whether this is a grace note, and if so, whether it is slashed
    /// (`Some(true)` = acciaccatura/short grace, `Some(false)` =
    /// appoggiatura/long grace). `None` means this is not a grace note.
    grace_slash: Option<bool>,
}

impl Note {
    /// Create a new note
    pub fn new(pitch: Pitch, duration: Duration) -> Self {
        Self {
            pitch,
            duration,
            offset: Fraction::new(0, 1),
            tie: None,
            lyrics: Vec::new(),
            articulations: Vec::new(),
            expressions: Vec::new(),
            volume: Volume::default(),
            notehead: NoteHead::default(),
            stem_direction: StemDirection::default(),
            grace_slash: None,
        }
    }

    /// Create a note from pitch string and duration type
    pub fn from_str(pitch: &str, duration: Duration) -> Result<Self, super::ParseError> {
        Ok(Self::new(pitch.parse()?, duration))
    }

    /// Create a quarter note
    pub fn quarter(pitch: Pitch) -> Self {
        Self::new(pitch, Duration::quarter())
    }

    /// Create a half note
    pub fn half(pitch: Pitch) -> Self {
        Self::new(pitch, Duration::half())
    }

    /// Create a whole note
    pub fn whole(pitch: Pitch) -> Self {
        Self::new(pitch, Duration::whole())
    }

    /// Create an eighth note
    pub fn eighth(pitch: Pitch) -> Self {
        Self::new(pitch, Duration::eighth())
    }

    /// Get the pitch
    pub fn pitch(&self) -> &Pitch {
        &self.pitch
    }

    /// Get mutable pitch
    pub fn pitch_mut(&mut self) -> &mut Pitch {
        &mut self.pitch
    }

    /// Set the pitch
    pub fn set_pitch(&mut self, pitch: Pitch) {
        self.pitch = pitch;
    }

    /// Get the duration
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    /// Get mutable duration
    pub fn duration_mut(&mut self) -> &mut Duration {
        &mut self.duration
    }

    /// Set the duration
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    /// Get the offset
    pub fn offset(&self) -> Fraction {
        self.offset
    }

    /// Set the offset
    pub fn set_offset(&mut self, offset: Fraction) {
        self.offset = offset;
    }

    /// Get the quarter length (convenience for duration.quarter_length())
    pub fn quarter_length(&self) -> Fraction {
        self.duration.quarter_length()
    }

    /// Get the MIDI note number
    pub fn midi(&self) -> u8 {
        self.pitch.midi()
    }

    /// Get the name (e.g., "C4")
    pub fn name(&self) -> String {
        self.pitch.name_with_octave()
    }

    /// Get the tie
    pub fn tie(&self) -> Option<&Tie> {
        self.tie.as_ref()
    }

    /// Set the tie
    pub fn set_tie(&mut self, tie: Option<Tie>) {
        self.tie = tie;
    }

    /// Get the lyrics
    pub fn lyrics(&self) -> &[Lyric] {
        &self.lyrics
    }

    /// Add a lyric
    pub fn add_lyric(&mut self, lyric: Lyric) {
        self.lyrics.push(lyric);
    }

    /// Add a lyric from text
    pub fn add_lyric_text(&mut self, text: impl Into<String>) {
        self.lyrics.push(Lyric::new(text));
    }

    /// Get the articulations
    pub fn articulations(&self) -> &[Articulation] {
        &self.articulations
    }

    /// Add an articulation
    pub fn add_articulation(&mut self, articulation: Articulation) {
        self.articulations.push(articulation);
    }

    /// Get the expressions
    pub fn expressions(&self) -> &[Expression] {
        &self.expressions
    }

    /// Add an expression
    pub fn add_expression(&mut self, expression: Expression) {
        self.expressions.push(expression);
    }

    /// Get the volume
    pub fn volume(&self) -> &Volume {
        &self.volume
    }

    /// Set the volume
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
    }

    /// Set the velocity
    pub fn set_velocity(&mut self, velocity: u8) {
        self.volume = Volume::from_velocity(velocity);
    }

    /// Get the realized (effective) volume after applying this note's own
    /// attached articulations' velocity multipliers (see
    /// `Volume::get_realized`).
    pub fn realized_volume(&self) -> Volume {
        self.volume.get_realized(&self.articulations)
    }

    /// Get the realized (effective) quarter-note length after applying this
    /// note's attached articulations' duration multipliers (e.g. staccato
    /// shortens the sounding duration without changing the notated
    /// duration returned by `duration()`).
    pub fn realized_quarter_length(&self) -> f64 {
        let multiplier: f64 = self
            .articulations
            .iter()
            .map(|a| a.type_.duration_multiplier())
            .product();
        self.duration.quarter_length_f64() * multiplier
    }

    /// Get the notehead
    pub fn notehead(&self) -> &NoteHead {
        &self.notehead
    }

    /// Set the notehead
    pub fn set_notehead(&mut self, notehead: NoteHead) {
        self.notehead = notehead;
    }

    /// Get the stem direction
    pub fn stem_direction(&self) -> StemDirection {
        self.stem_direction
    }

    /// Set the stem direction
    pub fn set_stem_direction(&mut self, direction: StemDirection) {
        self.stem_direction = direction;
    }

    /// Check if this is a grace note
    pub fn is_grace(&self) -> bool {
        self.grace_slash.is_some()
    }

    /// Whether this grace note is slashed (acciaccatura, `Some(true)`) or
    /// unslashed (appoggiatura, `Some(false)`); `None` if this isn't a
    /// grace note at all.
    pub fn is_grace_slashed(&self) -> Option<bool> {
        self.grace_slash
    }

    /// Convert to a slashed grace note (acciaccatura, the common "short"
    /// grace note). Use `to_appoggiatura` for an unslashed grace note.
    pub fn to_grace(&self) -> Note {
        let mut grace = self.clone();
        grace.grace_slash = Some(true);
        grace.duration = Duration::zero();
        grace
    }

    /// Convert to an unslashed appoggiatura-style grace note, which
    /// traditionally steals notated time from the note it precedes (unlike
    /// a slashed acciaccatura, which is not counted against the beat).
    pub fn to_appoggiatura(&self) -> Note {
        let mut grace = self.clone();
        grace.grace_slash = Some(false);
        grace.duration = Duration::zero();
        grace
    }

    /// Transpose the note
    pub fn transpose(&self, interval: &Interval) -> Note {
        let mut transposed = self.clone();
        transposed.pitch = self.pitch.transpose(interval);
        transposed
    }

    /// Transpose by semitones
    pub fn transpose_semitones(&self, semitones: i32) -> Note {
        let mut transposed = self.clone();
        transposed.pitch = self.pitch.transpose_semitones(semitones);
        transposed
    }

    /// Transpose by an interval given as a string (e.g. "P5", "-m3"),
    /// parsed via `Interval`'s `FromStr` implementation.
    pub fn transpose_str(&self, interval: &str) -> Result<Note, super::ParseError> {
        let interval: Interval = interval.parse()?;
        Ok(self.transpose(&interval))
    }

    /// Get this note's pitch as a single-element vector, for writing code
    /// that treats `Note` and `Chord` uniformly (a `Chord` has multiple
    /// pitches; a `Note` always has exactly one).
    pub fn pitches(&self) -> Vec<&Pitch> {
        vec![&self.pitch]
    }

    /// Get a human-readable summary of this note, e.g.
    /// "C-sharp in octave 4 Quarter Note".
    pub fn full_name(&self) -> String {
        format!(
            "{} in octave {} {}",
            self.pitch.name(),
            self.pitch.implicit_octave(),
            self.duration.full_name()
        )
    }

    /// Scale the duration
    pub fn augment_or_diminish(&self, scalar: Fraction) -> Note {
        let mut scaled = self.clone();
        scaled.duration = self.duration.augment_or_diminish(scalar);
        scaled
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::new(Pitch::default(), Duration::default())
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.pitch, self.duration)
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.pitch == other.pitch && self.duration == other.duration
    }
}

impl Eq for Note {}

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Note {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pitch.cmp(&other.pitch)
    }
}

/// A percussion-style note with no real pitch: only a display position on
/// the staff (e.g. for a specific drum/percussion instrument line) and a
/// duration. Mirrors music21's `note.Unpitched`.
#[derive(Debug, Clone, PartialEq)]
pub struct Unpitched {
    /// Staff display position (in the same line/space units as a staff
    /// position; 0 = middle line).
    display_position: i8,
    /// Duration
    duration: Duration,
}

impl Unpitched {
    /// Create a new unpitched note with the given duration, displayed on
    /// the middle line by default.
    pub fn new(duration: Duration) -> Self {
        Self {
            display_position: 0,
            duration,
        }
    }

    /// Get the staff display position.
    pub fn display_position(&self) -> i8 {
        self.display_position
    }

    /// Set the staff display position.
    pub fn set_display_position(&mut self, position: i8) {
        self.display_position = position;
    }

    /// Get the duration.
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    /// Set the duration.
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    /// Get the quarter length.
    pub fn quarter_length(&self) -> Fraction {
        self.duration.quarter_length()
    }

    /// Human-readable display name — there's no real pitch, so this
    /// describes the staff position instead (mirroring music21's
    /// `Unpitched.displayName`).
    pub fn display_name(&self) -> String {
        format!("staff position {}", self.display_position)
    }
}

impl fmt::Display for Unpitched {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unpitched({})", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Step;

    #[test]
    fn test_note_creation() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::quarter(pitch);

        assert_eq!(note.midi(), 60);
        assert_eq!(note.quarter_length(), Fraction::new(1, 1));
    }

    #[test]
    fn test_note_transpose() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::quarter(pitch);
        let transposed = note.transpose(&Interval::perfect_fifth());

        assert_eq!(transposed.pitch().step(), Step::G);
        assert_eq!(transposed.pitch().octave(), Some(4));
    }

    #[test]
    fn test_note_articulations() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let mut note = Note::quarter(pitch);
        note.add_articulation(Articulation::staccato());
        note.add_articulation(Articulation::accent());

        assert_eq!(note.articulations().len(), 2);
    }

    #[test]
    fn test_articulation_metadata_reachable_from_note() {
        // Regression test: before unifying the two Articulation enums,
        // Note::add_articulation only accepted the poorer
        // core::note::ArticulationType, so the richer metadata (symbols,
        // velocity/duration multipliers) on notation::articulation::
        // ArticulationMark was never reachable from an actual Note.
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let mut note = Note::quarter(pitch);
        note.add_articulation(Articulation::staccato());

        assert_eq!(note.articulations()[0].type_.symbol(), ".");
        assert!(note.articulations()[0].type_.affects_duration());
    }

    #[test]
    fn test_volume_get_realized_applies_velocity_multiplier() {
        let volume = Volume::from_velocity(50);
        let accented = volume.get_realized(&[Articulation::accent()]);
        // Accent has a 1.2x velocity multiplier plus a +16 additive
        // volume_shift (50 * 1.2 = 60, + 16 = 76).
        assert_eq!(accented.velocity, 76);

        let unarticulated = volume.get_realized(&[]);
        assert_eq!(unarticulated.velocity, 50);
    }

    #[test]
    fn test_note_realized_volume_and_duration() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let mut note = Note::quarter(pitch);
        note.set_velocity(100);
        note.add_articulation(Articulation::staccato());

        // Staccato: 0.9x velocity, 0.5x duration.
        assert_eq!(note.realized_volume().velocity, 90);
        assert!((note.realized_quarter_length() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_note_lyrics() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let mut note = Note::quarter(pitch);
        note.add_lyric_text("la");

        assert_eq!(note.lyrics().len(), 1);
        assert_eq!(note.lyrics()[0].text, "la");
    }

    #[test]
    fn test_note_grace() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::quarter(pitch);
        let grace = note.to_grace();

        assert!(grace.is_grace());
        assert_eq!(grace.quarter_length(), Fraction::new(0, 1));
        assert_eq!(grace.is_grace_slashed(), Some(true));
    }

    #[test]
    fn test_note_appoggiatura() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::quarter(pitch);
        let appoggiatura = note.to_appoggiatura();

        assert!(appoggiatura.is_grace());
        assert_eq!(appoggiatura.is_grace_slashed(), Some(false));

        assert_eq!(note.is_grace_slashed(), None);
    }

    #[test]
    fn test_note_full_name() {
        let pitch = Pitch::from_parts(Step::C, Some(4), Some(crate::core::Accidental::Sharp));
        let note = Note::quarter(pitch);
        assert_eq!(note.full_name(), "C# in octave 4 quarter");
    }

    #[test]
    fn test_note_transpose_str() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::quarter(pitch);
        let transposed = note.transpose_str("P5").unwrap();
        assert_eq!(transposed.pitch().step(), Step::G);

        assert!(note.transpose_str("bogus").is_err());
    }

    #[test]
    fn test_note_pitches_accessor() {
        let pitch = Pitch::from_parts(Step::D, Some(4), None);
        let note = Note::quarter(pitch);
        let pitches = note.pitches();
        assert_eq!(pitches.len(), 1);
        assert_eq!(pitches[0].step(), Step::D);
    }

    #[test]
    fn test_unpitched() {
        let mut unpitched = Unpitched::new(Duration::quarter());
        assert_eq!(unpitched.display_position(), 0);
        assert_eq!(unpitched.quarter_length(), Fraction::new(1, 1));

        unpitched.set_display_position(3);
        assert_eq!(unpitched.display_position(), 3);
        assert_eq!(unpitched.display_name(), "staff position 3");
    }

    #[test]
    fn test_lyric_from_raw_text_single_syllable() {
        let lyrics = Lyric::from_raw_text("hello", 1);
        assert_eq!(lyrics.len(), 1);
        assert_eq!(lyrics[0].syllabic, Syllabic::Single);
        assert!(!is_composite_lyric_set(&lyrics));
    }

    #[test]
    fn test_lyric_from_raw_text_multi_syllable() {
        let lyrics = Lyric::from_raw_text("con-tra-ry", 2);
        assert_eq!(lyrics.len(), 3);
        assert_eq!(lyrics[0].text, "con");
        assert_eq!(lyrics[0].syllabic, Syllabic::Begin);
        assert_eq!(lyrics[1].text, "tra");
        assert_eq!(lyrics[1].syllabic, Syllabic::Middle);
        assert_eq!(lyrics[2].text, "ry");
        assert_eq!(lyrics[2].syllabic, Syllabic::End);
        assert!(lyrics.iter().all(|l| l.number == 2));
        assert!(is_composite_lyric_set(&lyrics));
    }

    #[test]
    fn test_lyric_identifier() {
        let lyric = Lyric::new("la").with_identifier("chorus");
        assert_eq!(lyric.identifier(), Some("chorus"));
    }
}
