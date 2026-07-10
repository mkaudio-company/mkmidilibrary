//! Dynamic markings

use std::fmt;

use crate::core::Fraction;
use crate::notation::{Spanner, SpannerAnchor};

/// Dynamic level type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DynamicsType {
    /// As quiet as possible
    PPPP,
    /// Very very quiet
    PPP,
    /// Very quiet (pianissimo)
    PP,
    /// Quiet (piano)
    P,
    /// Moderately quiet (mezzo piano)
    MP,
    /// Moderately loud (mezzo forte)
    MF,
    /// Loud (forte)
    F,
    /// Very loud (fortissimo)
    FF,
    /// Very very loud
    FFF,
    /// As loud as possible
    FFFF,
    /// Sforzando (sudden accent)
    SF,
    /// Sforzando forte
    SFZ,
    /// Sforzando piano
    SFP,
    /// Forzando
    FZ,
    /// Rinforzando
    RF,
    /// Rinforzando forte
    RFZ,
    /// Fortepiano (loud, then immediately soft)
    FP,
    /// Niente ("nothing" — silence; used at the start/end of a hairpin to
    /// mean fading from/to inaudibility)
    N,
}

impl DynamicsType {
    /// Get the text representation
    pub fn text(&self) -> &'static str {
        match self {
            DynamicsType::PPPP => "pppp",
            DynamicsType::PPP => "ppp",
            DynamicsType::PP => "pp",
            DynamicsType::P => "p",
            DynamicsType::MP => "mp",
            DynamicsType::MF => "mf",
            DynamicsType::F => "f",
            DynamicsType::FF => "ff",
            DynamicsType::FFF => "fff",
            DynamicsType::FFFF => "ffff",
            DynamicsType::SF => "sf",
            DynamicsType::SFZ => "sfz",
            DynamicsType::SFP => "sfp",
            DynamicsType::FZ => "fz",
            DynamicsType::RF => "rf",
            DynamicsType::RFZ => "rfz",
            DynamicsType::FP => "fp",
            DynamicsType::N => "n",
        }
    }

    /// Get the full name
    pub fn name(&self) -> &'static str {
        match self {
            DynamicsType::PPPP => "pianissississimo",
            DynamicsType::PPP => "pianississimo",
            DynamicsType::PP => "pianissimo",
            DynamicsType::P => "piano",
            DynamicsType::MP => "mezzo piano",
            DynamicsType::MF => "mezzo forte",
            DynamicsType::F => "forte",
            DynamicsType::FF => "fortissimo",
            DynamicsType::FFF => "fortississimo",
            DynamicsType::FFFF => "fortissississimo",
            DynamicsType::SF => "sforzando",
            DynamicsType::SFZ => "sforzato",
            DynamicsType::SFP => "sforzando piano",
            DynamicsType::FZ => "forzando",
            DynamicsType::RF => "rinforzando",
            DynamicsType::RFZ => "rinforzato",
            DynamicsType::FP => "fortepiano",
            DynamicsType::N => "niente",
        }
    }

    /// Get the English-language description (mirrors music21's
    /// English-name dynamic table).
    pub fn english(&self) -> &'static str {
        match self {
            DynamicsType::PPPP => "as soft as possible",
            DynamicsType::PPP => "very very soft",
            DynamicsType::PP => "very soft",
            DynamicsType::P => "soft",
            DynamicsType::MP => "moderately soft",
            DynamicsType::MF => "moderately loud",
            DynamicsType::F => "loud",
            DynamicsType::FF => "very loud",
            DynamicsType::FFF => "very very loud",
            DynamicsType::FFFF => "as loud as possible",
            DynamicsType::SF | DynamicsType::SFZ => "sudden accent",
            DynamicsType::SFP => "sudden accent, then soft",
            DynamicsType::FZ => "forced accent",
            DynamicsType::RF | DynamicsType::RFZ => "reinforced accent",
            DynamicsType::FP => "loud, then immediately soft",
            DynamicsType::N => "nothing (silence)",
        }
    }

    /// Get typical MIDI velocity
    pub fn velocity(&self) -> u8 {
        match self {
            DynamicsType::PPPP => 16,
            DynamicsType::PPP => 24,
            DynamicsType::PP => 36,
            DynamicsType::P => 48,
            DynamicsType::MP => 64,
            DynamicsType::MF => 80,
            DynamicsType::F => 96,
            DynamicsType::FF => 112,
            DynamicsType::FFF => 120,
            DynamicsType::FFFF => 127,
            DynamicsType::SF | DynamicsType::SFZ | DynamicsType::FZ | DynamicsType::RFZ => 112,
            DynamicsType::SFP => 96,
            DynamicsType::RF => 100,
            DynamicsType::FP => 96,
            DynamicsType::N => 0,
        }
    }

    /// Get volume scalar (0.0-1.0)
    pub fn volume(&self) -> f64 {
        self.velocity() as f64 / 127.0
    }

    /// Check if this is an accent/sforzando type
    pub fn is_accent(&self) -> bool {
        matches!(
            self,
            DynamicsType::SF
                | DynamicsType::SFZ
                | DynamicsType::SFP
                | DynamicsType::FZ
                | DynamicsType::RF
                | DynamicsType::RFZ
                | DynamicsType::FP
        )
    }
}

/// Map a 0.0-1.0 volume scalar to the name of the nearest standard
/// dynamic level (`"pppp"` through `"ffff"`), by closest MIDI-velocity
/// match. Mirrors music21's `dynamicStrFromDecimal`.
pub fn dynamic_str_from_decimal(decimal: f64) -> &'static str {
    const LEVELS: [DynamicsType; 10] = [
        DynamicsType::PPPP,
        DynamicsType::PPP,
        DynamicsType::PP,
        DynamicsType::P,
        DynamicsType::MP,
        DynamicsType::MF,
        DynamicsType::F,
        DynamicsType::FF,
        DynamicsType::FFF,
        DynamicsType::FFFF,
    ];
    let clamped = decimal.clamp(0.0, 1.0);
    LEVELS
        .iter()
        .min_by(|a, b| {
            let da = (a.volume() - clamped).abs();
            let db = (b.volume() - clamped).abs();
            da.partial_cmp(&db).unwrap()
        })
        .map(|d| d.text())
        .unwrap_or("mf")
}

impl fmt::Display for DynamicsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// A dynamic marking
#[derive(Debug, Clone, PartialEq)]
pub struct Dynamics {
    /// Dynamic type
    type_: DynamicsType,
    /// Custom velocity override
    velocity_override: Option<u8>,
    /// A custom display string (e.g. "poco f"), overriding `type_`'s own
    /// text — for markings that don't fit the standard vocabulary.
    custom_text: Option<String>,
}

impl Dynamics {
    /// Create a new dynamic marking
    pub fn new(type_: DynamicsType) -> Self {
        Self {
            type_,
            velocity_override: None,
            custom_text: None,
        }
    }

    /// Create a dynamic marking with an arbitrary display string (e.g.
    /// "poco f") and an explicit velocity, for markings that don't fit
    /// the standard vocabulary.
    pub fn custom(text: impl Into<String>, velocity: u8) -> Self {
        Self {
            type_: DynamicsType::MF,
            velocity_override: Some(velocity),
            custom_text: Some(text.into()),
        }
    }

    /// Whether this marking has a custom display string rather than one
    /// of the standard `DynamicsType` labels.
    pub fn is_custom(&self) -> bool {
        self.custom_text.is_some()
    }

    /// This marking's display text: the custom string if set, otherwise
    /// `type_()`'s standard text (e.g. `"mf"`).
    pub fn text(&self) -> String {
        self.custom_text
            .clone()
            .unwrap_or_else(|| self.type_.text().to_string())
    }

    /// Create piano
    pub fn p() -> Self {
        Self::new(DynamicsType::P)
    }

    /// Create mezzo piano
    pub fn mp() -> Self {
        Self::new(DynamicsType::MP)
    }

    /// Create mezzo forte
    pub fn mf() -> Self {
        Self::new(DynamicsType::MF)
    }

    /// Create forte
    pub fn f() -> Self {
        Self::new(DynamicsType::F)
    }

    /// Create fortissimo
    pub fn ff() -> Self {
        Self::new(DynamicsType::FF)
    }

    /// Create pianissimo
    pub fn pp() -> Self {
        Self::new(DynamicsType::PP)
    }

    /// Get the type
    pub fn type_(&self) -> DynamicsType {
        self.type_
    }

    /// Get the velocity
    pub fn velocity(&self) -> u8 {
        self.velocity_override.unwrap_or_else(|| self.type_.velocity())
    }

    /// Set custom velocity
    pub fn set_velocity(&mut self, velocity: u8) {
        self.velocity_override = Some(velocity);
    }

    /// Get volume scalar
    pub fn volume(&self) -> f64 {
        self.velocity() as f64 / 127.0
    }
}

impl Default for Dynamics {
    fn default() -> Self {
        Self::mf()
    }
}

impl fmt::Display for Dynamics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl From<DynamicsType> for Dynamics {
    fn from(type_: DynamicsType) -> Self {
        Self::new(type_)
    }
}

/// Dynamic wedge (hairpin) direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DynamicWedgeType {
    /// Crescendo (getting louder)
    Crescendo,
    /// Decrescendo / Diminuendo (getting softer)
    Decrescendo,
}

impl DynamicWedgeType {
    /// Get the text representation
    pub fn text(&self) -> &'static str {
        match self {
            DynamicWedgeType::Crescendo => "cresc.",
            DynamicWedgeType::Decrescendo => "decresc.",
        }
    }
}

impl fmt::Display for DynamicWedgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// A dynamics hairpin (crescendo/decrescendo), anchored to a range of the
/// music via `Spanner` rather than floating free of any position —
/// unlike the marking alone, this lets you ask "does this wedge cover
/// offset X in measure Y" (`contains`) or interpolate an in-between
/// target velocity (`velocity_at`).
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicWedge {
    spanner: Spanner,
    wedge_type: DynamicWedgeType,
    start_dynamic: Option<Dynamics>,
    end_dynamic: Option<Dynamics>,
}

impl DynamicWedge {
    /// Create a new dynamic wedge spanning `start` to `end`.
    pub fn new(wedge_type: DynamicWedgeType, start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, wedge_type.to_string()),
            wedge_type,
            start_dynamic: None,
            end_dynamic: None,
        }
    }

    /// Create a crescendo spanning `start` to `end`.
    pub fn crescendo(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self::new(DynamicWedgeType::Crescendo, start, end)
    }

    /// Create a decrescendo/diminuendo spanning `start` to `end`.
    pub fn diminuendo(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self::new(DynamicWedgeType::Decrescendo, start, end)
    }

    /// Get the wedge direction
    pub fn wedge_type(&self) -> DynamicWedgeType {
        self.wedge_type
    }

    /// The underlying spanner (start/end anchors).
    pub fn spanner(&self) -> &Spanner {
        &self.spanner
    }

    /// Set start dynamic
    pub fn set_start_dynamic(&mut self, dynamic: Dynamics) {
        self.start_dynamic = Some(dynamic);
    }

    /// Set end dynamic
    pub fn set_end_dynamic(&mut self, dynamic: Dynamics) {
        self.end_dynamic = Some(dynamic);
    }

    /// Get start dynamic
    pub fn start_dynamic(&self) -> Option<&Dynamics> {
        self.start_dynamic.as_ref()
    }

    /// Get end dynamic
    pub fn end_dynamic(&self) -> Option<&Dynamics> {
        self.end_dynamic.as_ref()
    }

    /// Whether `pos` falls within this wedge's span.
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        self.spanner.contains(pos)
    }

    /// The target MIDI velocity at `pos`, linearly interpolated between
    /// `start_dynamic` and `end_dynamic`'s velocities. Returns `None` if
    /// `pos` isn't within the wedge's span, if neither endpoint dynamic
    /// is set, or if the wedge spans more than one measure (interpolating
    /// across measures would need real elapsed-time context this
    /// notation-only type doesn't track — only the single-measure case,
    /// by far the common one for a hairpin, is supported).
    pub fn velocity_at(&self, pos: SpannerAnchor) -> Option<u8> {
        if !self.contains(pos) {
            return None;
        }
        match (self.start_dynamic.as_ref(), self.end_dynamic.as_ref()) {
            (Some(s), None) => Some(s.velocity()),
            (None, Some(e)) => Some(e.velocity()),
            (None, None) => None,
            (Some(s), Some(e)) => {
                if !self.spanner.is_single_measure() {
                    return None;
                }
                let start_offset = fraction_to_f64(self.spanner.start().offset);
                let end_offset = fraction_to_f64(self.spanner.end().offset);
                let pos_offset = fraction_to_f64(pos.offset);
                if end_offset <= start_offset {
                    return Some(s.velocity());
                }
                let t = ((pos_offset - start_offset) / (end_offset - start_offset)).clamp(0.0, 1.0);
                Some((s.velocity() as f64 + (e.velocity() as f64 - s.velocity() as f64) * t).round() as u8)
            }
        }
    }
}

impl fmt::Display for DynamicWedge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.wedge_type)
    }
}

fn fraction_to_f64(f: Fraction) -> f64 {
    *f.numer() as f64 / *f.denom() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamics_creation() {
        let dyn_ = Dynamics::f();
        assert_eq!(dyn_.type_(), DynamicsType::F);
        assert!(dyn_.velocity() > 80);
    }

    #[test]
    fn test_dynamics_ordering() {
        assert!(DynamicsType::PP < DynamicsType::P);
        assert!(DynamicsType::P < DynamicsType::MF);
        assert!(DynamicsType::MF < DynamicsType::F);
        assert!(DynamicsType::F < DynamicsType::FF);
    }

    #[test]
    fn test_dynamics_velocity() {
        assert!(Dynamics::pp().velocity() < Dynamics::p().velocity());
        assert!(Dynamics::p().velocity() < Dynamics::mf().velocity());
        assert!(Dynamics::mf().velocity() < Dynamics::f().velocity());
    }

    #[test]
    fn test_dynamic_wedge_creation_and_span() {
        let start = SpannerAnchor::new(1, Fraction::new(0, 1));
        let end = SpannerAnchor::new(1, Fraction::new(4, 1));
        let mut cresc = DynamicWedge::crescendo(start, end);
        cresc.set_start_dynamic(Dynamics::p());
        cresc.set_end_dynamic(Dynamics::f());

        assert_eq!(cresc.wedge_type(), DynamicWedgeType::Crescendo);
        assert!(cresc.start_dynamic().is_some());
        assert!(cresc.end_dynamic().is_some());
        assert!(cresc.contains(SpannerAnchor::new(1, Fraction::new(2, 1))));
        assert!(!cresc.contains(SpannerAnchor::new(2, Fraction::new(0, 1))));
    }

    #[test]
    fn test_dynamic_wedge_velocity_interpolation() {
        // Regression: a Hairpin used to carry no notion of which notes it
        // spans, so there was no way to ask "what's the target velocity
        // partway through this crescendo" — DynamicWedge's Spanner
        // anchoring makes that answerable.
        let start = SpannerAnchor::new(1, Fraction::new(0, 1));
        let end = SpannerAnchor::new(1, Fraction::new(4, 1));
        let mut cresc = DynamicWedge::crescendo(start, end);
        cresc.set_start_dynamic(Dynamics::p());
        cresc.set_end_dynamic(Dynamics::f());

        let at_start = cresc.velocity_at(start).unwrap();
        let at_end = cresc.velocity_at(end).unwrap();
        let at_middle = cresc
            .velocity_at(SpannerAnchor::new(1, Fraction::new(2, 1)))
            .unwrap();

        assert_eq!(at_start, Dynamics::p().velocity());
        assert_eq!(at_end, Dynamics::f().velocity());
        assert!(at_middle > at_start && at_middle < at_end);
    }

    #[test]
    fn test_dynamic_wedge_cross_measure_velocity_at_is_none() {
        let start = SpannerAnchor::new(1, Fraction::new(0, 1));
        let end = SpannerAnchor::new(3, Fraction::new(0, 1));
        let mut cresc = DynamicWedge::crescendo(start, end);
        cresc.set_start_dynamic(Dynamics::p());
        cresc.set_end_dynamic(Dynamics::f());

        assert_eq!(
            cresc.velocity_at(SpannerAnchor::new(2, Fraction::new(0, 1))),
            None
        );
    }

    #[test]
    fn test_dynamics_custom_text() {
        let poco_f = Dynamics::custom("poco f", 90);
        assert!(poco_f.is_custom());
        assert_eq!(poco_f.text(), "poco f");
        assert_eq!(poco_f.velocity(), 90);
        assert_eq!(poco_f.to_string(), "poco f");

        assert!(!Dynamics::f().is_custom());
    }

    #[test]
    fn test_fp_and_niente_markings() {
        assert_eq!(DynamicsType::FP.text(), "fp");
        assert_eq!(DynamicsType::FP.name(), "fortepiano");
        assert!(DynamicsType::FP.is_accent());

        assert_eq!(DynamicsType::N.text(), "n");
        assert_eq!(DynamicsType::N.velocity(), 0);
    }

    #[test]
    fn test_english_names() {
        assert_eq!(DynamicsType::PP.english(), "very soft");
        assert_eq!(DynamicsType::F.english(), "loud");
        assert_eq!(DynamicsType::MP.english(), "moderately soft");
    }

    #[test]
    fn test_dynamic_str_from_decimal() {
        assert_eq!(dynamic_str_from_decimal(0.0), "pppp");
        assert_eq!(dynamic_str_from_decimal(1.0), "ffff");
        assert_eq!(dynamic_str_from_decimal(DynamicsType::MF.volume()), "mf");
    }
}
