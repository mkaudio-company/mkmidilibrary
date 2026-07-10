//! Duration and rhythm representation
//!
//! Duration represents the temporal length of musical events,
//! measured in quarter note lengths.

use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

use num::{One, Zero};

use super::{Fraction, ParseError};

/// Duration type (note value)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationType {
    /// Duplex maxima (16 whole notes)
    DuplexMaxima,
    /// Maxima (8 whole notes)
    Maxima,
    /// Longa (4 whole notes)
    Longa,
    /// Breve / Double whole note (2 whole notes)
    Breve,
    /// Whole note / Semibreve
    Whole,
    /// Half note / Minim
    Half,
    /// Quarter note / Crotchet
    Quarter,
    /// Eighth note / Quaver
    Eighth,
    /// 16th note / Semiquaver
    N16th,
    /// 32nd note / Demisemiquaver
    N32nd,
    /// 64th note / Hemidemisemiquaver
    N64th,
    /// 128th note
    N128th,
    /// 256th note
    N256th,
    /// 512th note
    N512th,
    /// 1024th note
    N1024th,
    /// 2048th note
    N2048th,
    /// Zero duration (grace note)
    Zero,
}

/// All duration types with a nonzero length, ordered largest to smallest.
/// Used for parsing/inference and for `Duration`'s tied-component
/// decomposition.
const ALL_DURATION_TYPES: [DurationType; 16] = [
    DurationType::DuplexMaxima,
    DurationType::Maxima,
    DurationType::Longa,
    DurationType::Breve,
    DurationType::Whole,
    DurationType::Half,
    DurationType::Quarter,
    DurationType::Eighth,
    DurationType::N16th,
    DurationType::N32nd,
    DurationType::N64th,
    DurationType::N128th,
    DurationType::N256th,
    DurationType::N512th,
    DurationType::N1024th,
    DurationType::N2048th,
];

impl DurationType {
    /// Get the quarter note length for this duration type (without dots)
    pub fn quarter_length(&self) -> Fraction {
        match self {
            DurationType::DuplexMaxima => Fraction::new(64, 1),
            DurationType::Maxima => Fraction::new(32, 1),
            DurationType::Longa => Fraction::new(16, 1),
            DurationType::Breve => Fraction::new(8, 1),
            DurationType::Whole => Fraction::new(4, 1),
            DurationType::Half => Fraction::new(2, 1),
            DurationType::Quarter => Fraction::one(),
            DurationType::Eighth => Fraction::new(1, 2),
            DurationType::N16th => Fraction::new(1, 4),
            DurationType::N32nd => Fraction::new(1, 8),
            DurationType::N64th => Fraction::new(1, 16),
            DurationType::N128th => Fraction::new(1, 32),
            DurationType::N256th => Fraction::new(1, 64),
            DurationType::N512th => Fraction::new(1, 128),
            DurationType::N1024th => Fraction::new(1, 256),
            DurationType::N2048th => Fraction::new(1, 512),
            DurationType::Zero => Fraction::zero(),
        }
    }

    /// Get duration type from quarter length (returns closest match)
    pub fn from_quarter_length(ql: Fraction) -> Option<DurationType> {
        if ql == Fraction::zero() {
            return Some(DurationType::Zero);
        }

        ALL_DURATION_TYPES
            .into_iter()
            .find(|&t| t.quarter_length() == ql)
    }

    /// Get the standard name
    pub fn name(&self) -> &'static str {
        match self {
            DurationType::DuplexMaxima => "duplex maxima",
            DurationType::Maxima => "maxima",
            DurationType::Longa => "longa",
            DurationType::Breve => "breve",
            DurationType::Whole => "whole",
            DurationType::Half => "half",
            DurationType::Quarter => "quarter",
            DurationType::Eighth => "eighth",
            DurationType::N16th => "16th",
            DurationType::N32nd => "32nd",
            DurationType::N64th => "64th",
            DurationType::N128th => "128th",
            DurationType::N256th => "256th",
            DurationType::N512th => "512th",
            DurationType::N1024th => "1024th",
            DurationType::N2048th => "2048th",
            DurationType::Zero => "zero",
        }
    }
}

impl std::str::FromStr for DurationType {
    type Err = ParseError;

    /// Parse from string
    fn from_str(s: &str) -> Result<DurationType, ParseError> {
        match s.to_lowercase().as_str() {
            "duplex maxima" | "duplex-maxima" => Ok(DurationType::DuplexMaxima),
            "maxima" => Ok(DurationType::Maxima),
            "longa" | "long" => Ok(DurationType::Longa),
            "breve" | "double whole" | "double-whole" => Ok(DurationType::Breve),
            "whole" | "1" | "semibreve" => Ok(DurationType::Whole),
            "half" | "2" | "minim" => Ok(DurationType::Half),
            "quarter" | "4" | "crotchet" => Ok(DurationType::Quarter),
            "eighth" | "8" | "8th" | "quaver" => Ok(DurationType::Eighth),
            "16th" | "16" | "semiquaver" => Ok(DurationType::N16th),
            "32nd" | "32" | "demisemiquaver" => Ok(DurationType::N32nd),
            "64th" | "64" | "hemidemisemiquaver" => Ok(DurationType::N64th),
            "128th" | "128" => Ok(DurationType::N128th),
            "256th" | "256" => Ok(DurationType::N256th),
            "512th" | "512" => Ok(DurationType::N512th),
            "1024th" | "1024" => Ok(DurationType::N1024th),
            "2048th" | "2048" => Ok(DurationType::N2048th),
            "zero" | "0" | "grace" => Ok(DurationType::Zero),
            _ => Err(ParseError::InvalidDurationType(s.to_string())),
        }
    }
}

impl fmt::Display for DurationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A tuplet modification to duration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tuplet {
    /// Number of notes in the tuplet group
    pub actual: u8,
    /// Number of notes the tuplet replaces
    pub normal: u8,
    /// Duration type of the tuplet notes
    pub duration_type: Option<DurationType>,
}

impl Tuplet {
    /// Create a new tuplet
    pub fn new(actual: u8, normal: u8) -> Self {
        Self {
            actual,
            normal,
            duration_type: None,
        }
    }

    /// Create a triplet (3 in the space of 2)
    pub fn triplet() -> Self {
        Self::new(3, 2)
    }

    /// Create a duplet (2 in the space of 3)
    pub fn duplet() -> Self {
        Self::new(2, 3)
    }

    /// Create a quintuplet (5 in the space of 4)
    pub fn quintuplet() -> Self {
        Self::new(5, 4)
    }

    /// Get the multiplier for this tuplet
    pub fn multiplier(&self) -> Fraction {
        Fraction::new(self.normal as i64, self.actual as i64)
    }

    /// Set the duration type shared by the tuplet's notes.
    pub fn set_duration_type(&mut self, type_: DurationType) {
        self.duration_type = Some(type_);
    }

    /// Set the actual/normal ratio (e.g. `set_ratio(3, 2)` for a triplet).
    pub fn set_ratio(&mut self, actual: u8, normal: u8) {
        self.actual = actual;
        self.normal = normal;
    }

    /// Total quarter-note length this tuplet group occupies — i.e. what the
    /// tuplet's `actual` notes replace: `normal` copies of the base
    /// duration type (e.g. a triplet of eighths, 3-in-the-space-of-2,
    /// occupies the same time as 2 plain eighths, a quarter note).
    pub fn total_tuplet_length(&self) -> Fraction {
        let unit = self
            .duration_type
            .unwrap_or(DurationType::Quarter)
            .quarter_length();
        unit * Fraction::new(self.normal as i64, 1)
    }

    /// The written duration of one "actual" (performed) note in the
    /// tuplet. Note: this crate models a tuplet with a single shared
    /// `duration_type` for both sides, so `duration_actual` and
    /// `duration_normal` currently always agree on type (unlike music21,
    /// which allows cross-type tuplets); only the actual/normal *counts*
    /// differ here.
    pub fn duration_actual(&self) -> Duration {
        Duration::from_type(self.duration_type.unwrap_or(DurationType::Quarter), 0)
    }

    /// The written duration of one "normal" note being replaced. See
    /// `duration_actual`'s note about this crate's single-type model.
    pub fn duration_normal(&self) -> Duration {
        self.duration_actual()
    }
}

impl Default for Tuplet {
    fn default() -> Self {
        Self::triplet()
    }
}

/// Repairs a group of tied durations sharing a single tuplet so their
/// total written length exactly matches what the tuplet claims to occupy
/// (`Tuplet::total_tuplet_length`), adjusting the last duration to absorb
/// any rounding drift. Mirrors music21's `duration.TupletFixer` at a basic
/// level (single-tuplet, single-group repair rather than full
/// stream-search-and-fix).
pub struct TupletFixer;

impl TupletFixer {
    /// Fix `durations` in place. Returns `true` if a correction was made,
    /// `false` if the group already summed correctly (or was empty).
    pub fn fix(durations: &mut [Duration], tuplet: &Tuplet) -> bool {
        if durations.is_empty() {
            return false;
        }

        let target = tuplet.total_tuplet_length();
        let sum: Fraction = durations.iter().map(|d| d.quarter_length()).sum();
        if sum == target {
            return false;
        }

        let diff = target - sum;
        let last = durations.len() - 1;
        let corrected = durations[last].quarter_length() + diff;
        durations[last] = Duration::from_quarter_length(corrected);
        true
    }
}

/// A single written-note component: a duration type + dot count, with no
/// tuplet scaling. This is the building block `Duration::components`
/// decomposes an arbitrary quarter length into when no single type+dots
/// pair represents it exactly (a "complex"/tied duration).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationTuple {
    /// The written duration type of this component.
    pub type_: DurationType,
    /// Number of dots on this component.
    pub dots: u8,
}

impl DurationTuple {
    /// Create a new duration tuple.
    pub fn new(type_: DurationType, dots: u8) -> Self {
        Self { type_, dots }
    }

    /// The quarter-note length of this component alone (dots applied, no tuplets).
    pub fn quarter_length(&self) -> Fraction {
        Duration::calculate_quarter_length(self.type_, self.dots, &[])
    }
}

/// A musical duration measured in quarter note lengths
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duration {
    /// The quarter note length
    quarter_length: Fraction,
    /// The base duration type (if known)
    type_: Option<DurationType>,
    /// Number of dots
    dots: u8,
    /// Tuplets applied to this duration
    tuplets: Vec<Tuplet>,
    /// Whether type and quarter_length are linked
    linked: bool,
}

impl Duration {
    /// Create a new duration from quarter note length
    pub fn from_quarter_length(ql: impl Into<Fraction>) -> Self {
        let ql = ql.into();
        let (type_, dots) = Self::infer_type_and_dots(ql);

        Self {
            quarter_length: ql,
            type_,
            dots,
            tuplets: Vec::new(),
            linked: true,
        }
    }

    /// Create a duration from type and dots
    pub fn from_type(type_: DurationType, dots: u8) -> Self {
        let quarter_length = Self::calculate_quarter_length(type_, dots, &[]);

        Self {
            quarter_length,
            type_: Some(type_),
            dots,
            tuplets: Vec::new(),
            linked: true,
        }
    }

    /// Create a zero duration (for grace notes)
    pub fn zero() -> Self {
        Self::from_type(DurationType::Zero, 0)
    }

    /// Create a whole note duration
    pub fn whole() -> Self {
        Self::from_type(DurationType::Whole, 0)
    }

    /// Create a half note duration
    pub fn half() -> Self {
        Self::from_type(DurationType::Half, 0)
    }

    /// Create a quarter note duration
    pub fn quarter() -> Self {
        Self::from_type(DurationType::Quarter, 0)
    }

    /// Create an eighth note duration
    pub fn eighth() -> Self {
        Self::from_type(DurationType::Eighth, 0)
    }

    /// Create a 16th note duration
    pub fn sixteenth() -> Self {
        Self::from_type(DurationType::N16th, 0)
    }

    /// Get the quarter note length
    pub fn quarter_length(&self) -> Fraction {
        self.quarter_length
    }

    /// Get the quarter length as f64
    pub fn quarter_length_f64(&self) -> f64 {
        *self.quarter_length.numer() as f64 / *self.quarter_length.denom() as f64
    }

    /// Set the quarter note length
    pub fn set_quarter_length(&mut self, ql: impl Into<Fraction>) {
        self.quarter_length = ql.into();
        if self.linked {
            let (type_, dots) = Self::infer_type_and_dots(self.quarter_length);
            self.type_ = type_;
            self.dots = dots;
        }
    }

    /// Get the duration type
    pub fn type_(&self) -> Option<DurationType> {
        self.type_
    }

    /// Set the duration type
    pub fn set_type(&mut self, type_: DurationType) {
        self.type_ = Some(type_);
        if self.linked {
            self.quarter_length = Self::calculate_quarter_length(type_, self.dots, &self.tuplets);
        }
    }

    /// Get the number of dots
    pub fn dots(&self) -> u8 {
        self.dots
    }

    /// Set the number of dots
    pub fn set_dots(&mut self, dots: u8) {
        self.dots = dots;
        if self.linked
            && let Some(type_) = self.type_
        {
            self.quarter_length = Self::calculate_quarter_length(type_, dots, &self.tuplets);
        }
    }

    /// Get the tuplets
    pub fn tuplets(&self) -> &[Tuplet] {
        &self.tuplets
    }

    /// Add a tuplet
    pub fn add_tuplet(&mut self, tuplet: Tuplet) {
        self.tuplets.push(tuplet);
        if self.linked
            && let Some(type_) = self.type_
        {
            self.quarter_length = Self::calculate_quarter_length(type_, self.dots, &self.tuplets);
        }
    }

    /// Clear all tuplets
    pub fn clear_tuplets(&mut self) {
        self.tuplets.clear();
        if self.linked
            && let Some(type_) = self.type_
        {
            self.quarter_length = Self::calculate_quarter_length(type_, self.dots, &self.tuplets);
        }
    }

    /// Check if linked mode is enabled
    pub fn linked(&self) -> bool {
        self.linked
    }

    /// Set linked mode
    pub fn set_linked(&mut self, linked: bool) {
        self.linked = linked;
    }

    /// Scale the duration by a factor
    pub fn augment_or_diminish(&self, scalar: impl Into<Fraction>) -> Duration {
        let new_ql = self.quarter_length * scalar.into();
        Duration::from_quarter_length(new_ql)
    }

    /// Double the duration
    pub fn augment(&self) -> Duration {
        self.augment_or_diminish(Fraction::new(2, 1))
    }

    /// Halve the duration
    pub fn diminish(&self) -> Duration {
        self.augment_or_diminish(Fraction::new(1, 2))
    }

    /// Check if this is a complex duration (requires ties)
    pub fn is_complex(&self) -> bool {
        self.type_.is_none()
    }

    /// Decompose this duration's *written* quarter length (tuplet scaling
    /// excluded, see `quarter_length_no_tuplets`) into a tied sequence of
    /// single (type, dots) components. If a single type+dots exactly
    /// represents the duration, this returns exactly one component
    /// (matching `type_()`/`dots()`). Otherwise (a "complex" duration —
    /// `is_complex()` is true), this returns the greedy largest-first
    /// decomposition ties are conventionally notated with, e.g. 5
    /// sixteenths (quarter length 5/4) decomposes to a dotted eighth tied
    /// to a sixteenth, rather than being an opaque, undecomposable value.
    pub fn components(&self) -> Vec<DurationTuple> {
        if let Some(type_) = self.type_ {
            return vec![DurationTuple::new(type_, self.dots)];
        }

        let mut remaining = self.quarter_length_no_tuplets();
        let mut components = Vec::new();
        let mut guard = 0;
        while remaining > Fraction::zero() && guard < 16 {
            guard += 1;
            let mut best: Option<(DurationType, u8, Fraction)> = None;
            for type_ in ALL_DURATION_TYPES {
                for dots in 0..=4u8 {
                    let ql = Self::calculate_quarter_length(type_, dots, &[]);
                    if ql <= remaining && best.is_none_or(|(_, _, best_ql)| ql > best_ql) {
                        best = Some((type_, dots, ql));
                    }
                }
            }
            match best {
                Some((type_, dots, ql)) if ql > Fraction::zero() => {
                    components.push(DurationTuple::new(type_, dots));
                    remaining -= ql;
                }
                _ => break,
            }
        }
        components
    }

    /// Extend this duration's total length by tying on another written
    /// component, mirroring music21's `Duration.addDurationTuple`.
    pub fn add_duration_tuple(&mut self, type_: DurationType, dots: u8) {
        let added = Self::calculate_quarter_length(type_, dots, &[]);
        self.set_quarter_length(self.quarter_length + added);
    }

    /// Combine several tied-together durations into a single `Duration`
    /// representing their total length, simplifying to a single type+dots
    /// if their sum allows it. Mirrors music21's `Duration.consolidate`,
    /// used when a group of tied notes could instead be written as one.
    pub fn consolidate(durations: &[Duration]) -> Duration {
        let total: Fraction = durations.iter().map(|d| d.quarter_length).sum();
        Duration::from_quarter_length(total)
    }

    /// Split this duration's dots into separate tied components (e.g. a
    /// dotted quarter becomes `[quarter, eighth]`). This does not consider
    /// compound-meter beat boundaries (which would require `TimeSignature`
    /// context) — it purely splits the dot structure itself.
    pub fn split_dot_groups(&self) -> Vec<Duration> {
        match self.type_ {
            Some(type_) if self.dots > 0 => {
                let mut pieces = vec![Duration::from_type(type_, 0)];
                let mut piece_len = type_.quarter_length() / 2;
                for _ in 0..self.dots {
                    pieces.push(Duration::from_quarter_length(piece_len));
                    piece_len /= 2;
                }
                pieces
            }
            _ => vec![self.clone()],
        }
    }

    /// Get the component (see `components()`) containing a given offset
    /// within this duration, if any.
    pub fn slice_component_at_position(&self, position: Fraction) -> Option<DurationTuple> {
        let index = self.component_index_at_qtr_position(position)?;
        self.components().get(index).copied()
    }

    /// Get the index into `components()` of the component containing a
    /// given offset within this duration.
    pub fn component_index_at_qtr_position(&self, position: Fraction) -> Option<usize> {
        if position < Fraction::zero() || position >= self.quarter_length {
            return None;
        }
        let mut offset = Fraction::zero();
        for (i, c) in self.components().iter().enumerate() {
            let len = c.quarter_length();
            if position < offset + len {
                return Some(i);
            }
            offset += len;
        }
        None
    }

    /// Get the starting offset (within this duration) of the component at
    /// `index` in `components()`.
    pub fn component_start_time(&self, index: usize) -> Option<Fraction> {
        let comps = self.components();
        if index >= comps.len() {
            return None;
        }
        let mut offset = Fraction::zero();
        for c in &comps[..index] {
            offset += c.quarter_length();
        }
        Some(offset)
    }

    /// The quarter length this duration would have with all tuplet scaling
    /// removed (i.e. the written, un-scaled length).
    pub fn quarter_length_no_tuplets(&self) -> Fraction {
        let mut ql = self.quarter_length;
        for tuplet in &self.tuplets {
            ql /= tuplet.multiplier();
        }
        ql
    }

    /// The combined multiplier of all tuplets applied to this duration.
    pub fn aggregate_tuplet_multiplier(&self) -> Fraction {
        self.tuplets
            .iter()
            .fold(Fraction::one(), |acc, t| acc * t.multiplier())
    }

    /// Get a human-readable description
    pub fn full_name(&self) -> String {
        let mut name = String::new();

        if let Some(type_) = self.type_ {
            // Add dots
            for _ in 0..self.dots {
                name.push_str("Dotted ");
            }

            name.push_str(type_.name());

            // Add tuplets
            for tuplet in &self.tuplets {
                name.push_str(&format!(" ({} in {})", tuplet.actual, tuplet.normal));
            }
        } else {
            name = format!("Complex ({})", self.quarter_length);
        }

        name
    }

    /// Calculate quarter length from type, dots, and tuplets
    fn calculate_quarter_length(type_: DurationType, dots: u8, tuplets: &[Tuplet]) -> Fraction {
        let base = type_.quarter_length();

        // Apply dots: each dot adds half of the previous value
        let mut ql = base;
        let mut dot_value = base / 2;
        for _ in 0..dots {
            ql += dot_value;
            dot_value /= 2;
        }

        // Apply tuplets
        for tuplet in tuplets {
            ql *= tuplet.multiplier();
        }

        ql
    }

    /// Infer type and dots from quarter length
    fn infer_type_and_dots(ql: Fraction) -> (Option<DurationType>, u8) {
        if ql == Fraction::zero() {
            return (Some(DurationType::Zero), 0);
        }

        // Try each duration type with 0-4 dots
        for type_ in ALL_DURATION_TYPES {
            for dots in 0..=4 {
                if Self::calculate_quarter_length(type_, dots, &[]) == ql {
                    return (Some(type_), dots);
                }
            }
        }

        (None, 0)
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::quarter()
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length + rhs.quarter_length)
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length - rhs.quarter_length)
    }
}

impl Mul<Fraction> for Duration {
    type Output = Duration;

    fn mul(self, rhs: Fraction) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length * rhs)
    }
}

impl Div<Fraction> for Duration {
    type Output = Duration;

    fn div(self, rhs: Fraction) -> Self::Output {
        Duration::from_quarter_length(self.quarter_length / rhs)
    }
}

impl From<DurationType> for Duration {
    fn from(type_: DurationType) -> Self {
        Duration::from_type(type_, 0)
    }
}

impl FromStr for Duration {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse as a duration type first
        if let Ok(type_) = DurationType::from_str(s) {
            return Ok(Duration::from_type(type_, 0));
        }

        // Try to parse as a fraction (e.g., "1/4" for quarter note)
        if let Some((num, denom)) = s.split_once('/')
            && let (Ok(n), Ok(d)) = (num.trim().parse::<i64>(), denom.trim().parse::<i64>())
        {
            return Ok(Duration::from_quarter_length(Fraction::new(n, d)));
        }

        // Try to parse as a decimal
        if let Ok(f) = s.parse::<f64>() {
            // Convert to fraction (approximate)
            let denom = 256i64;
            let numer = (f * denom as f64).round() as i64;
            return Ok(Duration::from_quarter_length(Fraction::new(numer, denom)));
        }

        Err(ParseError::InvalidDurationType(s.to_string()))
    }
}

/// A grace-note duration: grace notes take zero actual performed quarter
/// length (their notated duration is only for visual/beaming purposes),
/// but are still written with a duration type and a slash flag — slashed
/// (the common default, "acciaccatura"/short grace note) or unslashed
/// ("appoggiatura", which traditionally steals notated time from the note
/// it precedes). See `Note::to_grace`/`Note::to_appoggiatura`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraceDuration {
    /// The written duration type (for display/beaming).
    pub notated_type: DurationType,
    /// Number of dots on the written duration.
    pub notated_dots: u8,
    /// Whether the grace note is slashed (true = acciaccatura/short grace;
    /// false = appoggiatura/long grace).
    pub slash: bool,
}

impl GraceDuration {
    /// Create a slashed (acciaccatura-style) grace duration.
    pub fn new(notated_type: DurationType, notated_dots: u8) -> Self {
        Self {
            notated_type,
            notated_dots,
            slash: true,
        }
    }

    /// Create an unslashed (appoggiatura-style) grace duration.
    pub fn appoggiatura(notated_type: DurationType, notated_dots: u8) -> Self {
        Self {
            notated_type,
            notated_dots,
            slash: false,
        }
    }

    /// Get the notated (display) quarter length, even though a grace note
    /// itself takes no actual performed time during playback.
    pub fn notated_quarter_length(&self) -> Fraction {
        Duration::calculate_quarter_length(self.notated_type, self.notated_dots, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_duration_type_variants() {
        assert_eq!(
            DurationType::DuplexMaxima.quarter_length(),
            Fraction::new(64, 1)
        );
        assert_eq!(DurationType::N512th.quarter_length(), Fraction::new(1, 128));
        assert_eq!(
            DurationType::N1024th.quarter_length(),
            Fraction::new(1, 256)
        );
        assert_eq!(
            DurationType::N2048th.quarter_length(),
            Fraction::new(1, 512)
        );
        assert_eq!(
            DurationType::from_str("512th").unwrap(),
            DurationType::N512th
        );
    }

    #[test]
    fn test_components_simple_duration_is_single_component() {
        let d = Duration::quarter();
        let comps = d.components();
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0], DurationTuple::new(DurationType::Quarter, 0));
    }

    #[test]
    fn test_components_decompose_complex_duration() {
        // Regression test: a quarter length of 5/4 (5 sixteenths) used to
        // be an opaque "complex" duration with no further introspection
        // possible (type_() == None, no way to see the tied breakdown).
        // It should now decompose into a real tied sequence whose
        // components sum back to the original length.
        let d = Duration::from_quarter_length(Fraction::new(5, 4));
        assert!(d.is_complex());

        let comps = d.components();
        assert!(comps.len() > 1, "expected a genuine tied decomposition");

        let total: Fraction = comps.iter().map(|c| c.quarter_length()).sum();
        assert_eq!(total, Fraction::new(5, 4));

        // Greedy largest-first: dotted eighth (3/4 of an eighth... i.e.
        // 3/16 quarter-length... let's just check the largest component is
        // a dotted quarter or dotted eighth per the greedy algorithm.
        assert!(comps[0].quarter_length() >= comps.last().unwrap().quarter_length());
    }

    #[test]
    fn test_component_index_and_start_time() {
        let d = Duration::from_quarter_length(Fraction::new(5, 4));
        let comps = d.components();

        // The very first offset (0) must fall in component 0.
        assert_eq!(d.component_index_at_qtr_position(Fraction::zero()), Some(0));
        assert_eq!(d.component_start_time(0), Some(Fraction::zero()));

        // The last component's start time plus its length must equal the total.
        let last_index = comps.len() - 1;
        let last_start = d.component_start_time(last_index).unwrap();
        assert_eq!(
            last_start + comps[last_index].quarter_length(),
            Fraction::new(5, 4)
        );

        // Out of range position/index return None.
        assert_eq!(d.component_index_at_qtr_position(Fraction::new(5, 4)), None);
        assert_eq!(d.component_start_time(comps.len()), None);
    }

    #[test]
    fn test_add_duration_tuple_and_consolidate() {
        let mut d = Duration::quarter();
        d.add_duration_tuple(DurationType::Eighth, 0);
        assert_eq!(d.quarter_length(), Fraction::new(3, 2));
        assert_eq!(d.type_(), Some(DurationType::Quarter)); // dotted quarter
        assert_eq!(d.dots(), 1);

        let consolidated = Duration::consolidate(&[Duration::quarter(), Duration::eighth()]);
        assert_eq!(consolidated.quarter_length(), Fraction::new(3, 2));
    }

    #[test]
    fn test_split_dot_groups() {
        let dotted_quarter = Duration::from_type(DurationType::Quarter, 1);
        let pieces = dotted_quarter.split_dot_groups();
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].quarter_length(), Fraction::one());
        assert_eq!(pieces[1].quarter_length(), Fraction::new(1, 2));

        let plain_quarter = Duration::quarter();
        assert_eq!(plain_quarter.split_dot_groups().len(), 1);
    }

    #[test]
    fn test_quarter_length_no_tuplets_and_aggregate_multiplier() {
        let mut d = Duration::quarter();
        d.add_tuplet(Tuplet::triplet());
        assert_eq!(d.quarter_length(), Fraction::new(2, 3));
        assert_eq!(d.quarter_length_no_tuplets(), Fraction::one());
        assert_eq!(d.aggregate_tuplet_multiplier(), Fraction::new(2, 3));
    }

    #[test]
    fn test_tuplet_additions() {
        let mut t = Tuplet::triplet();
        t.set_duration_type(DurationType::Eighth);
        assert_eq!(t.total_tuplet_length(), Fraction::new(1, 1)); // 2 eighths = quarter
        assert_eq!(t.duration_actual().quarter_length(), Fraction::new(1, 2));
        assert_eq!(t.duration_normal().quarter_length(), Fraction::new(1, 2));

        t.set_ratio(5, 4);
        assert_eq!(t.actual, 5);
        assert_eq!(t.normal, 4);
    }

    #[test]
    fn test_tuplet_fixer() {
        let mut tuplet = Tuplet::triplet();
        tuplet.set_duration_type(DurationType::Eighth);
        // Triplet of eighths should sum to a quarter note (1.0); simulate a
        // group that's slightly off due to accumulated rounding.
        let mut durations = vec![
            Duration::from_quarter_length(Fraction::new(1, 3)),
            Duration::from_quarter_length(Fraction::new(1, 3)),
            Duration::from_quarter_length(Fraction::new(1, 4)), // wrong on purpose
        ];
        let fixed = TupletFixer::fix(&mut durations, &tuplet);
        assert!(fixed);
        let total: Fraction = durations.iter().map(|d| d.quarter_length()).sum();
        assert_eq!(total, Fraction::one());

        // Already-correct group should report no fix needed.
        let mut correct = vec![Duration::from_quarter_length(Fraction::one())];
        assert!(!TupletFixer::fix(&mut correct, &Tuplet::new(1, 1)));
    }

    #[test]
    fn test_grace_duration() {
        let grace = GraceDuration::new(DurationType::Eighth, 0);
        assert!(grace.slash);
        assert_eq!(grace.notated_quarter_length(), Fraction::new(1, 2));

        let appoggiatura = GraceDuration::appoggiatura(DurationType::Eighth, 0);
        assert!(!appoggiatura.slash);
    }

    #[test]
    fn test_duration_quarter_length() {
        assert_eq!(Duration::whole().quarter_length(), Fraction::new(4, 1));
        assert_eq!(Duration::half().quarter_length(), Fraction::new(2, 1));
        assert_eq!(Duration::quarter().quarter_length(), Fraction::one());
        assert_eq!(Duration::eighth().quarter_length(), Fraction::new(1, 2));
    }

    #[test]
    fn test_duration_from_quarter_length() {
        let d = Duration::from_quarter_length(Fraction::new(4, 1));
        assert_eq!(d.type_(), Some(DurationType::Whole));
        assert_eq!(d.dots(), 0);

        let d = Duration::from_quarter_length(Fraction::new(3, 2));
        assert_eq!(d.type_(), Some(DurationType::Quarter));
        assert_eq!(d.dots(), 1); // dotted quarter
    }

    #[test]
    fn test_duration_dots() {
        let dotted_half = Duration::from_type(DurationType::Half, 1);
        assert_eq!(dotted_half.quarter_length(), Fraction::new(3, 1));

        let double_dotted_quarter = Duration::from_type(DurationType::Quarter, 2);
        assert_eq!(double_dotted_quarter.quarter_length(), Fraction::new(7, 4));
    }

    #[test]
    fn test_duration_tuplet() {
        let mut d = Duration::quarter();
        d.add_tuplet(Tuplet::triplet());
        assert_eq!(d.quarter_length(), Fraction::new(2, 3));
    }

    #[test]
    fn test_duration_arithmetic() {
        let half = Duration::half();
        let quarter = Duration::quarter();

        let sum = half.clone() + quarter.clone();
        assert_eq!(sum.quarter_length(), Fraction::new(3, 1));

        let diff = half - quarter;
        assert_eq!(diff.quarter_length(), Fraction::one());
    }

    #[test]
    fn test_duration_augment_diminish() {
        let quarter = Duration::quarter();

        let half = quarter.augment();
        assert_eq!(half.quarter_length(), Fraction::new(2, 1));

        let eighth = quarter.diminish();
        assert_eq!(eighth.quarter_length(), Fraction::new(1, 2));
    }
}
