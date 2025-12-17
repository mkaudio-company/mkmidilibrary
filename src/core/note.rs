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

/// Articulation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArticulationType {
    Accent,
    StrongAccent,
    Staccato,
    Staccatissimo,
    Tenuto,
    DetachedLegato,
    Spiccato,
    Scoop,
    Plop,
    Doit,
    Falloff,
    BreathMark,
    Caesura,
    Fermata,
    UpBow,
    DownBow,
    Harmonic,
    OpenString,
    Pizzicato,
    SnapPizzicato,
    Stopped,
}

/// An articulation marking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Articulation {
    /// Articulation type
    pub type_: ArticulationType,
    /// Placement (above/below)
    pub placement: Option<Placement>,
}

impl Articulation {
    /// Create a new articulation
    pub fn new(type_: ArticulationType) -> Self {
        Self {
            type_,
            placement: None,
        }
    }

    /// Create a staccato
    pub fn staccato() -> Self {
        Self::new(ArticulationType::Staccato)
    }

    /// Create an accent
    pub fn accent() -> Self {
        Self::new(ArticulationType::Accent)
    }

    /// Create a tenuto
    pub fn tenuto() -> Self {
        Self::new(ArticulationType::Tenuto)
    }

    /// Create a fermata
    pub fn fermata() -> Self {
        Self::new(ArticulationType::Fermata)
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
}

impl Lyric {
    /// Create a new lyric
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            syllabic: Syllabic::Single,
            number: 1,
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
    /// Whether this is a grace note
    is_grace: bool,
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
            is_grace: false,
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
        self.is_grace
    }

    /// Convert to a grace note
    pub fn to_grace(&self) -> Note {
        let mut grace = self.clone();
        grace.is_grace = true;
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
    }
}
