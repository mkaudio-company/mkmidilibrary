//! Articulation markings
//!
//! The actual `ArticulationMark` enum lives in `crate::core::note` (it's
//! attached to `core::note::Note` via `core::note::Articulation`) and is
//! re-exported here for convenient access from the `notation` module path.
//! It used to be duplicated here as a separate, richer-but-unreachable
//! enum; see `crate::core::note::ArticulationMark`'s doc comment for the
//! history.

use crate::notation::{Spanner, SpannerAnchor};

pub use crate::core::ArticulationMark;

/// Placement for articulation marks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ArticulationPlacement {
    #[default]
    Auto,
    Above,
    Below,
}

/// Hammer-on or pull-off direction (fretted-string techniques for
/// sounding a second note without re-plucking/re-bowing it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HammerPullType {
    /// Hammer-on: sound a higher fretted note by pressing the string
    /// down, without picking it.
    HammerOn,
    /// Pull-off: sound a lower fretted (or open) note by releasing/
    /// plucking with the fretting finger, without picking it.
    PullOff,
}

impl HammerPullType {
    /// A short display label.
    pub fn text(&self) -> &'static str {
        match self {
            HammerPullType::HammerOn => "H",
            HammerPullType::PullOff => "P",
        }
    }
}

impl std::fmt::Display for HammerPullType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// A hammer-on or pull-off, anchored to the pair of notes it connects via
/// `Spanner` (unlike a plain articulation mark, this can answer "does
/// this hammer-on/pull-off cover this position").
#[derive(Debug, Clone, PartialEq)]
pub struct HammerPullSpanner {
    spanner: Spanner,
    kind: HammerPullType,
}

impl HammerPullSpanner {
    /// Create a hammer-on spanning `start` to `end`.
    pub fn hammer_on(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, HammerPullType::HammerOn.to_string()),
            kind: HammerPullType::HammerOn,
        }
    }

    /// Create a pull-off spanning `start` to `end`.
    pub fn pull_off(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        Self {
            spanner: Spanner::with_label(start, end, HammerPullType::PullOff.to_string()),
            kind: HammerPullType::PullOff,
        }
    }

    /// Get the direction (hammer-on vs. pull-off).
    pub fn kind(&self) -> HammerPullType {
        self.kind
    }

    /// The underlying spanner (start/end anchors).
    pub fn spanner(&self) -> &Spanner {
        &self.spanner
    }

    /// Whether `pos` falls within this hammer-on/pull-off's span.
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        self.spanner.contains(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Fraction;

    #[test]
    fn test_articulation_mark() {
        let staccato = ArticulationMark::Staccato;
        assert_eq!(staccato.name(), "staccato");
        assert!(staccato.affects_duration());
        assert!(staccato.duration_multiplier() < 1.0);
    }

    #[test]
    fn test_accent_velocity() {
        let accent = ArticulationMark::Accent;
        assert!(accent.velocity_multiplier() > 1.0);
    }

    #[test]
    fn test_fermata() {
        assert!(ArticulationMark::Fermata.is_fermata());
        assert!(ArticulationMark::ShortFermata.is_fermata());
        assert!(!ArticulationMark::Staccato.is_fermata());
    }

    #[test]
    fn test_new_articulation_variants() {
        assert_eq!(ArticulationMark::Stress.name(), "stress");
        assert_eq!(ArticulationMark::Unstress.name(), "unstress");
        assert_eq!(ArticulationMark::StringHarmonic.name(), "string harmonic");
        assert_eq!(
            ArticulationMark::HandbellIndication.name(),
            "handbell indication"
        );
        assert_eq!(ArticulationMark::HarpFingerNails.name(), "harp fingernails");
        assert_eq!(
            ArticulationMark::WoodwindIndication.name(),
            "woodwind indication"
        );
        assert_eq!(ArticulationMark::BrassIndication.name(), "brass indication");
    }

    #[test]
    fn test_fret_and_string_number_markings() {
        let fret = ArticulationMark::Fret(3);
        assert_eq!(fret.fret_number(), Some(3));
        assert_eq!(fret.name(), "fret");
        assert_eq!(ArticulationMark::Accent.fret_number(), None);

        let string_num = ArticulationMark::StringNumber(2);
        assert_eq!(string_num.string_number(), Some(2));
    }

    #[test]
    fn test_additive_volume_shift_alongside_multiplier() {
        use crate::core::{Articulation, Volume};

        // Regression/enhancement: Accent's effect used to be purely
        // multiplicative (velocity_multiplier), which barely moves a
        // quiet base velocity and can overshoot a loud one. The new
        // additive volume_shift is applied on top of (not instead of)
        // the existing multiplier.
        assert_eq!(ArticulationMark::Accent.volume_shift(), 16);
        assert_eq!(ArticulationMark::Staccato.volume_shift(), 0);

        let accent = Articulation::new(ArticulationMark::Accent);
        let quiet = Volume::from_velocity(20);
        let loud = Volume::from_velocity(110);

        let quiet_realized = quiet.get_realized(std::slice::from_ref(&accent));
        let loud_realized = loud.get_realized(std::slice::from_ref(&accent));

        // The additive shift moves the quiet note by a noticeable
        // fraction of its own velocity (more than the ~20% multiplier
        // alone would), and the loud note doesn't blow past 127.
        assert!(quiet_realized.velocity > quiet.velocity + 10);
        assert!(loud_realized.velocity <= 127);
        assert!(loud_realized.velocity > loud.velocity);
    }

    #[test]
    fn test_hammer_on_and_pull_off_spanners() {
        let start = SpannerAnchor::new(1, Fraction::new(0, 1));
        let end = SpannerAnchor::new(1, Fraction::new(1, 1));

        let hammer_on = HammerPullSpanner::hammer_on(start, end);
        assert_eq!(hammer_on.kind(), HammerPullType::HammerOn);
        assert!(hammer_on.contains(start));
        assert!(hammer_on.contains(end));
        assert!(!hammer_on.contains(SpannerAnchor::new(2, Fraction::new(0, 1))));

        let pull_off = HammerPullSpanner::pull_off(start, end);
        assert_eq!(pull_off.kind(), HammerPullType::PullOff);
        assert_eq!(pull_off.kind().text(), "P");
        assert_eq!(hammer_on.kind().text(), "H");
    }
}
