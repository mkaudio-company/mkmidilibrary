//! Expression markings and ornament realization.
//!
//! `core::note::{Expression, ExpressionType}` provide the plain labels
//! (Trill/Turn/Mordent/etc.) attachable to a `Note`. This module adds the
//! actual *realization* logic that was previously entirely missing —
//! expanding an ornament into the sequence of performed notes it
//! represents, given the main note and a tonal context (`Key`) to
//! determine the diatonic auxiliary pitch — plus new expression/spanner
//! types not modeled by a plain label alone.

use crate::core::{Accidental, Duration, Fraction, Interval, Note, Pitch};
use crate::notation::{Key, Scale, Spanner, SpannerAnchor};

/// Whether an ornament's auxiliary note is a diatonic step (determined by
/// the prevailing key) or an explicit chromatic half/whole step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OrnamentSize {
    /// Use whatever step the key's scale gives (the usual case).
    #[default]
    Diatonic,
    /// Force a chromatic half step, regardless of the key.
    Half,
    /// Force a whole step, regardless of the key.
    Whole,
}

/// Which family of ornament this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrnamentKind {
    Trill,
    Turn,
    InvertedTurn,
    Mordent,
    InvertedMordent,
}

/// A realizable ornament: an upper/lower-neighbor decoration of a main
/// note, expandable into the actual sequence of performed pitches/notes
/// given a tonal context. Mirrors (a scoped subset of) music21's
/// `expressions.Ornament` and its `realize`/`resolveOrnamentalPitches`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ornament {
    kind: OrnamentKind,
    size: OrnamentSize,
    /// Explicit accidental override for the auxiliary note's diatonic
    /// step, if any (bypasses the key-derived accidental but keeps the
    /// key-derived letter name).
    accidental: Option<Accidental>,
}

impl Ornament {
    /// Create a new ornament of the given kind, with default (diatonic,
    /// no accidental override) sizing.
    pub fn new(kind: OrnamentKind) -> Self {
        Self {
            kind,
            size: OrnamentSize::Diatonic,
            accidental: None,
        }
    }

    /// Create a trill.
    pub fn trill() -> Self {
        Self::new(OrnamentKind::Trill)
    }

    /// Create a turn (upper neighbor, main, lower neighbor, main).
    pub fn turn() -> Self {
        Self::new(OrnamentKind::Turn)
    }

    /// Create an inverted turn (lower neighbor, main, upper neighbor, main).
    pub fn inverted_turn() -> Self {
        Self::new(OrnamentKind::InvertedTurn)
    }

    /// Create a (lower) mordent: main, lower neighbor, main.
    pub fn mordent() -> Self {
        Self::new(OrnamentKind::Mordent)
    }

    /// Create an inverted (upper) mordent: main, upper neighbor, main.
    pub fn inverted_mordent() -> Self {
        Self::new(OrnamentKind::InvertedMordent)
    }

    /// Get the ornament kind.
    pub fn kind(&self) -> OrnamentKind {
        self.kind
    }

    /// Set the auxiliary-note sizing (diatonic/half/whole step).
    pub fn with_size(mut self, size: OrnamentSize) -> Self {
        self.size = size;
        self
    }

    /// Get the auxiliary-note sizing.
    pub fn size(&self) -> OrnamentSize {
        self.size
    }

    /// Set an explicit accidental override for the auxiliary note.
    pub fn with_accidental(mut self, accidental: Accidental) -> Self {
        self.accidental = Some(accidental);
        self
    }

    /// Get the accidental override, if any.
    pub fn accidental(&self) -> Option<Accidental> {
        self.accidental
    }

    /// The distinct pitches this ornament moves between, in melodic order
    /// for a single cycle: `[main, upper]` for a trill (the caller
    /// repeats this to fill the note's duration — see `realize`),
    /// `[main, lower, main]` for a mordent, `[upper, main, lower, main]`
    /// for a turn, and so on. Mirrors music21's
    /// `resolveOrnamentalPitches`.
    pub fn resolve_ornamental_pitches(&self, main_pitch: &Pitch, key: &Key) -> Vec<Pitch> {
        let upper = self.neighbor_pitch(main_pitch, key, true);
        let lower = self.neighbor_pitch(main_pitch, key, false);
        match self.kind {
            OrnamentKind::Trill => vec![main_pitch.clone(), upper],
            OrnamentKind::Mordent => vec![main_pitch.clone(), lower, main_pitch.clone()],
            OrnamentKind::InvertedMordent => vec![main_pitch.clone(), upper, main_pitch.clone()],
            OrnamentKind::Turn => vec![upper, main_pitch.clone(), lower, main_pitch.clone()],
            OrnamentKind::InvertedTurn => {
                vec![lower, main_pitch.clone(), upper, main_pitch.clone()]
            }
        }
    }

    /// Expand this ornament into the actual performed notes for
    /// `main_note`, in `key`'s tonal context: the main note's duration is
    /// divided evenly among the notes `resolve_ornamental_pitches`
    /// produces. A trill repeats its 2-note (main/upper) pattern enough
    /// times to fill the duration at roughly a 16th-note pace (clamped to
    /// a sane range); a turn/mordent plays its fixed pattern exactly
    /// once. Mirrors (a simplified form of) music21's `Ornament.realize`/
    /// `fillListOfRealizedNotes`.
    pub fn realize(&self, main_note: &Note, key: &Key) -> Vec<Note> {
        let pattern = self.resolve_ornamental_pitches(main_note.pitch(), key);
        let total = main_note.duration().quarter_length();

        let note_count = match self.kind {
            OrnamentKind::Trill => {
                let sixteenth = Fraction::new(1, 4);
                let subdivisions = (total / sixteenth).to_integer().max(2);
                (subdivisions as usize).clamp(2, 32)
            }
            _ => pattern.len(),
        };

        let each_length = total / Fraction::from(note_count as i64);
        let each_duration = Duration::from_quarter_length(each_length);

        (0..note_count)
            .map(|i| Note::new(pattern[i % pattern.len()].clone(), each_duration.clone()))
            .collect()
    }

    fn neighbor_pitch(&self, main_pitch: &Pitch, key: &Key, upper: bool) -> Pitch {
        match self.size {
            OrnamentSize::Half => main_pitch.transpose_semitones(if upper { 1 } else { -1 }),
            OrnamentSize::Whole => main_pitch.transpose_semitones(if upper { 2 } else { -2 }),
            OrnamentSize::Diatonic => {
                let mut candidate = diatonic_neighbor(main_pitch, key, upper);
                if let Some(accidental) = self.accidental {
                    candidate.set_accidental(Some(accidental));
                }
                candidate
            }
        }
    }
}

/// The diatonic upper/lower neighbor of `main_pitch` within `key`'s scale,
/// at the correct octave (crossing an octave boundary when the neighbor
/// wraps past the top/bottom of the scale, e.g. B -> C). Falls back to a
/// chromatic half step if `main_pitch` isn't itself one of the scale's 7
/// spelled pitches (e.g. it already carries a foreign accidental).
fn diatonic_neighbor(main_pitch: &Pitch, key: &Key, upper: bool) -> Pitch {
    let scale = Scale::new(key.tonic().clone(), key.mode());
    let Some(degree) = scale.degree_of(main_pitch) else {
        return main_pitch.transpose_semitones(if upper { 1 } else { -1 });
    };

    let neighbor_degree = if upper {
        if degree == 7 { 1 } else { degree + 1 }
    } else if degree == 1 {
        7
    } else {
        degree - 1
    };

    let mut candidate = scale
        .pitch_for_degree(neighbor_degree)
        .expect("degree is always in 1..=7");
    candidate.set_octave(Some(main_pitch.implicit_octave()));

    if upper && candidate.midi() <= main_pitch.midi() {
        candidate = candidate.transpose(&Interval::octave());
    } else if !upper && candidate.midi() >= main_pitch.midi() {
        candidate = candidate.transpose(&Interval::octave().reverse());
    }
    candidate
}

/// A rehearsal mark (e.g. "A", "12", "Coda") — a positional label, not a
/// performance instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct RehearsalMark {
    text: String,
}

impl RehearsalMark {
    /// Create a new rehearsal mark with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Get the mark's text.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// A free-form text expression (e.g. "cantabile", "dolce") attached at a
/// position, distinct from a `RehearsalMark` (a structural label) or a
/// `Dynamics` marking (a specific standardized dynamic level).
#[derive(Debug, Clone, PartialEq)]
pub struct TextExpression {
    text: String,
}

impl TextExpression {
    /// Create a new text expression with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Get the expression's text.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// A sustain-pedal mark, anchored to the range of music it covers via
/// `Spanner` (e.g. "Ped." ... "*").
#[derive(Debug, Clone, PartialEq)]
pub struct PedalMark {
    spanner: Spanner,
}

impl PedalMark {
    /// Create a new pedal mark spanning `start` to `end`.
    pub fn new(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, "pedal"),
        }
    }

    /// The underlying spanner (start/end anchors).
    pub fn spanner(&self) -> &Spanner {
        &self.spanner
    }

    /// Whether `pos` falls within this pedal mark's span.
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        self.spanner.contains(pos)
    }
}

/// An arpeggio marking (roll a chord from bottom to top, or top to
/// bottom), anchored to the chord/notes it covers via `Spanner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArpeggioDirection {
    Up,
    Down,
    /// Non-arpeggiated bracket (play simultaneously; the bracket only
    /// clarifies notation).
    NonArpeggio,
}

/// An arpeggio marking (spans the chord it decorates).
#[derive(Debug, Clone, PartialEq)]
pub struct ArpeggioMark {
    spanner: Spanner,
    direction: ArpeggioDirection,
}

impl ArpeggioMark {
    /// Create a new arpeggio mark spanning `start` to `end`.
    pub fn new(direction: ArpeggioDirection, start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, "arpeggio"),
            direction,
        }
    }

    /// Get the arpeggio direction.
    pub fn direction(&self) -> ArpeggioDirection {
        self.direction
    }

    /// The underlying spanner (start/end anchors).
    pub fn spanner(&self) -> &Spanner {
        &self.spanner
    }

    /// Whether `pos` falls within this arpeggio mark's span.
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        self.spanner.contains(pos)
    }
}

/// An extension line continuing a trill mark across a range of notes,
/// anchored via `Spanner`.
#[derive(Debug, Clone, PartialEq)]
pub struct TrillExtension {
    spanner: Spanner,
}

impl TrillExtension {
    /// Create a new trill extension spanning `start` to `end`.
    pub fn new(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, "trill extension"),
        }
    }

    /// The underlying spanner (start/end anchors).
    pub fn spanner(&self) -> &Spanner {
        &self.spanner
    }

    /// Whether `pos` falls within this trill extension's span.
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        self.spanner.contains(pos)
    }
}

/// A tremolo marking spanning a pair of notes (measured/unmeasured
/// tremolo between two alternating pitches), anchored via `Spanner`.
#[derive(Debug, Clone, PartialEq)]
pub struct TremoloSpanner {
    spanner: Spanner,
    /// Number of tremolo strokes/beams (e.g. 2 for two beams).
    strokes: u8,
}

impl TremoloSpanner {
    /// Create a new tremolo spanner spanning `start` to `end`, with the
    /// given number of strokes/beams.
    pub fn new(start: SpannerAnchor, end: SpannerAnchor, strokes: u8) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, "tremolo"),
            strokes,
        }
    }

    /// Get the number of tremolo strokes/beams.
    pub fn strokes(&self) -> u8 {
        self.strokes
    }

    /// The underlying spanner (start/end anchors).
    pub fn spanner(&self) -> &Spanner {
        &self.spanner
    }

    /// Whether `pos` falls within this tremolo spanner's span.
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        self.spanner.contains(pos)
    }
}

/// A turn's optional delay: whether the ornament's neighbor tones are
/// played immediately on the beat, or delayed to just before the next
/// note (a "delayed turn", common in Romantic-era notation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TurnDelay {
    #[default]
    OnBeat,
    Delayed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Step;
    use crate::notation::KeyMode;

    fn c_major_key() -> Key {
        Key::new(Pitch::from_parts(Step::C, Some(4), None), KeyMode::Major)
    }

    #[test]
    fn test_mordent_resolves_lower_neighbor_diatonically() {
        let key = c_major_key();
        let main = Pitch::from_parts(Step::C, Some(5), None);
        let pitches = Ornament::mordent().resolve_ornamental_pitches(&main, &key);

        assert_eq!(pitches.len(), 3);
        assert_eq!(pitches[0].name(), "C");
        assert_eq!(pitches[1].name(), "B"); // diatonic lower neighbor in C major
        assert_eq!(pitches[1].octave(), Some(4)); // crosses down an octave
        assert_eq!(pitches[2].name(), "C");
    }

    #[test]
    fn test_inverted_mordent_resolves_upper_neighbor() {
        let key = c_major_key();
        let main = Pitch::from_parts(Step::C, Some(4), None);
        let pitches = Ornament::inverted_mordent().resolve_ornamental_pitches(&main, &key);

        assert_eq!(pitches[1].name(), "D");
        assert_eq!(pitches[1].octave(), Some(4));
    }

    #[test]
    fn test_turn_pattern_order() {
        let key = c_major_key();
        let main = Pitch::from_parts(Step::E, Some(4), None);
        let pitches = Ornament::turn().resolve_ornamental_pitches(&main, &key);

        let names: Vec<String> = pitches.iter().map(|p| p.name()).collect();
        assert_eq!(names, vec!["F", "E", "D", "E"]);
    }

    #[test]
    fn test_trill_realize_fills_duration_alternating_main_and_upper() {
        let key = c_major_key();
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::new(pitch, Duration::quarter());
        let realized = Ornament::trill().realize(&note, &key);

        assert!(realized.len() >= 2);
        // Total duration is preserved.
        let total: Fraction = realized.iter().map(|n| n.duration().quarter_length()).sum();
        assert_eq!(total, Fraction::new(1, 1));
        // Alternates main (C) and upper neighbor (D).
        for (i, n) in realized.iter().enumerate() {
            let expected = if i % 2 == 0 { "C" } else { "D" };
            assert_eq!(n.pitch().name(), expected);
        }
    }

    #[test]
    fn test_ornament_size_override_forces_chromatic_step() {
        let key = c_major_key();
        let main = Pitch::from_parts(Step::C, Some(4), None);
        let half_step_mordent = Ornament::mordent().with_size(OrnamentSize::Half);
        let pitches = half_step_mordent.resolve_ornamental_pitches(&main, &key);
        // A half-step below C4 (midi 60) is B3 (midi 59), same as the
        // diatonic case here, but verify explicitly by semitone distance.
        assert_eq!(main.midi() - pitches[1].midi(), 1);
    }

    #[test]
    fn test_non_scale_tone_falls_back_to_chromatic_step() {
        let key = c_major_key();
        // F# is not a diatonic tone of C major.
        let main = Pitch::from_parts(Step::F, Some(4), Some(Accidental::Sharp));
        let pitches = Ornament::mordent().resolve_ornamental_pitches(&main, &key);
        assert_eq!(main.midi() - pitches[1].midi(), 1);
    }

    #[test]
    fn test_rehearsal_mark_and_text_expression() {
        let mark = RehearsalMark::new("A");
        assert_eq!(mark.text(), "A");

        let text = TextExpression::new("dolce");
        assert_eq!(text.text(), "dolce");
    }

    #[test]
    fn test_pedal_arpeggio_trill_extension_and_tremolo_spanners() {
        let start = SpannerAnchor::new(1, Fraction::new(0, 1));
        let end = SpannerAnchor::new(1, Fraction::new(4, 1));

        let pedal = PedalMark::new(start, end);
        assert!(pedal.contains(SpannerAnchor::new(1, Fraction::new(2, 1))));

        let arpeggio = ArpeggioMark::new(ArpeggioDirection::Up, start, end);
        assert_eq!(arpeggio.direction(), ArpeggioDirection::Up);

        let trill_ext = TrillExtension::new(start, end);
        assert!(trill_ext.contains(start));

        let tremolo = TremoloSpanner::new(start, end, 3);
        assert_eq!(tremolo.strokes(), 3);
    }
}
