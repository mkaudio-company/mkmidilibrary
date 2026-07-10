//! Chord representation
//!
//! A Chord represents multiple simultaneous pitches with a shared duration.

use std::cmp::Ordering;
use std::fmt;

use super::{Duration, Fraction, Interval, Note, Pitch, Volume};

/// Chord quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant,
    HalfDiminished,
    FullyDiminished,
    Suspended2,
    Suspended4,
    Power,
    Other,
}

impl ChordQuality {
    /// Get the symbol for this quality
    pub fn symbol(&self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Diminished => "dim",
            ChordQuality::Augmented => "aug",
            ChordQuality::Dominant => "7",
            ChordQuality::HalfDiminished => "m7b5",
            ChordQuality::FullyDiminished => "dim7",
            ChordQuality::Suspended2 => "sus2",
            ChordQuality::Suspended4 => "sus4",
            ChordQuality::Power => "5",
            ChordQuality::Other => "",
        }
    }
}

/// A chord (multiple simultaneous pitches)
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    /// The notes in this chord (each has its own pitch)
    notes: Vec<Note>,
    /// Shared duration for the chord
    duration: Duration,
    /// Offset within the stream
    offset: Fraction,
}

impl Chord {
    /// Create a new chord from notes
    pub fn new(notes: Vec<Note>, duration: Duration) -> Self {
        Self {
            notes,
            duration,
            offset: Fraction::new(0, 1),
        }
    }

    /// Create a chord from pitches
    pub fn from_pitches(pitches: Vec<Pitch>, duration: Duration) -> Self {
        let notes = pitches
            .into_iter()
            .map(|p| Note::new(p, duration.clone()))
            .collect();
        Self {
            notes,
            duration,
            offset: Fraction::new(0, 1),
        }
    }

    /// Create a chord from pitch strings
    pub fn from_pitch_strings(pitches: &[&str], duration: Duration) -> Result<Self, super::ParseError> {
        let parsed: Result<Vec<Pitch>, _> = pitches.iter().map(|s| s.parse()).collect();
        Ok(Self::from_pitches(parsed?, duration))
    }

    /// Create a major triad
    pub fn major_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create a minor triad
    pub fn minor_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::minor_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create a diminished triad
    pub fn diminished_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::minor_third());
        let fifth = root.transpose(&Interval::tritone());
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create an augmented triad
    pub fn augmented_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::new(4, 8)); // augmented fifth
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create a dominant seventh chord
    pub fn dominant_seventh(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        let seventh = root.transpose(&Interval::minor_seventh());
        Self::from_pitches(vec![root, third, fifth, seventh], Duration::quarter())
    }

    /// Create a major seventh chord
    pub fn major_seventh(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        let seventh = root.transpose(&Interval::major_seventh());
        Self::from_pitches(vec![root, third, fifth, seventh], Duration::quarter())
    }

    /// Create a minor seventh chord
    pub fn minor_seventh(root: Pitch) -> Self {
        let third = root.transpose(&Interval::minor_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        let seventh = root.transpose(&Interval::minor_seventh());
        Self::from_pitches(vec![root, third, fifth, seventh], Duration::quarter())
    }

    /// Get the notes
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// Get mutable notes
    pub fn notes_mut(&mut self) -> &mut Vec<Note> {
        &mut self.notes
    }

    /// Get the pitches
    pub fn pitches(&self) -> Vec<&Pitch> {
        self.notes.iter().map(|n| n.pitch()).collect()
    }

    /// Get pitch classes (0-11)
    pub fn pitch_classes(&self) -> Vec<u8> {
        self.notes.iter().map(|n| n.pitch().pitch_class()).collect()
    }

    /// Get ordered pitch classes (sorted, unique)
    pub fn ordered_pitch_classes(&self) -> Vec<u8> {
        let mut pcs = self.pitch_classes();
        pcs.sort();
        pcs.dedup();
        pcs
    }

    /// Add a note to the chord
    pub fn add(&mut self, note: Note) {
        self.notes.push(note);
    }

    /// Add a pitch to the chord
    pub fn add_pitch(&mut self, pitch: Pitch) {
        self.notes.push(Note::new(pitch, self.duration.clone()));
    }

    /// Remove a note from the chord
    pub fn remove(&mut self, index: usize) -> Option<Note> {
        if index < self.notes.len() {
            Some(self.notes.remove(index))
        } else {
            None
        }
    }

    /// Get the duration
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    /// Set the duration
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration.clone();
        for note in &mut self.notes {
            note.set_duration(duration.clone());
        }
    }

    /// Get the offset
    pub fn offset(&self) -> Fraction {
        self.offset
    }

    /// Set the offset
    pub fn set_offset(&mut self, offset: Fraction) {
        self.offset = offset;
    }

    /// Get the quarter length
    pub fn quarter_length(&self) -> Fraction {
        self.duration.quarter_length()
    }

    /// Get the bass (lowest pitch)
    pub fn bass(&self) -> Option<&Pitch> {
        self.notes.iter().map(|n| n.pitch()).min()
    }

    /// Get the root (may differ from bass in inversions).
    ///
    /// Finds a candidate whose pitch classes, sorted and measured from
    /// that candidate, form a single unbroken chain of thirds (each
    /// consecutive gap is 3 or 4 semitones) — this covers triads, seventh
    /// chords, and (unlike a check that requires an exact perfect fifth)
    /// diminished and augmented chords too, since their "fifth" is a
    /// tritone/augmented fifth rather than a perfect fifth.
    ///
    /// Candidates are tried bass-first (the common root-position case),
    /// then the remaining pitch classes in ascending order, so the result
    /// no longer depends on what order notes were added to the chord (a
    /// first-inversion C major triad entered as E-G-C still returns C).
    /// For intervallically symmetric chords (e.g. a fully-diminished
    /// seventh, or an augmented triad) more than one candidate can pass
    /// this check — genuinely ambiguous in music theory without further
    /// spelling context — and this deterministically returns the first
    /// match in that bass-first-then-ascending order.
    pub fn root(&self) -> Option<Pitch> {
        if self.notes.is_empty() {
            return None;
        }

        let pcs = self.ordered_pitch_classes();
        if pcs.len() < 2 {
            return self.bass().cloned();
        }

        let bass_pc = self.bass().map(|p| p.pitch_class());
        let mut candidates = pcs.clone();
        if let Some(bpc) = bass_pc {
            candidates.sort_by_key(|&pc| if pc == bpc { 0 } else { 1 });
        }

        for &candidate_pc in &candidates {
            let intervals: Vec<u8> = pcs
                .iter()
                .map(|&pc| (pc + 12 - candidate_pc) % 12)
                .collect();
            if Self::is_stacked_thirds_from_root(&intervals) {
                return self
                    .notes
                    .iter()
                    .find(|n| n.pitch().pitch_class() == candidate_pc)
                    .map(|n| n.pitch().clone());
            }
        }

        // No candidate forms a clean stacked-third chain (e.g. a cluster
        // chord); fall back to the bass, matching the previous behavior
        // for genuinely non-tertian chords.
        self.bass().cloned()
    }

    /// Whether `intervals` (semitone offsets from a candidate root,
    /// 0-11, need not be sorted or deduplicated) form a single unbroken
    /// chain of thirds when sorted ascending from the root — i.e. this
    /// candidate could plausibly be the chord's root.
    fn is_stacked_thirds_from_root(intervals: &[u8]) -> bool {
        let mut sorted: Vec<u8> = intervals.iter().copied().filter(|&i| i != 0).collect();
        sorted.sort_unstable();
        sorted.dedup();

        let mut prev = 0u8;
        for iv in sorted {
            let step = (iv + 12 - prev) % 12;
            if step != 3 && step != 4 {
                return false;
            }
            prev = iv;
        }
        true
    }

    /// Get the inversion (0 = root position, 1 = first inversion, etc.)
    pub fn inversion(&self) -> u8 {
        if let (Some(bass), Some(root)) = (self.bass(), self.root()) {
            let bass_pc = bass.pitch_class();
            let root_pc = root.pitch_class();

            if bass_pc == root_pc {
                return 0;
            }

            let pcs = self.ordered_pitch_classes();
            let root_idx = pcs.iter().position(|&pc| pc == root_pc).unwrap_or(0);
            let bass_idx = pcs.iter().position(|&pc| pc == bass_pc).unwrap_or(0);

            ((bass_idx as i32 - root_idx as i32).rem_euclid(pcs.len() as i32)) as u8
        } else {
            0
        }
    }

    /// Determine the chord quality
    pub fn quality(&self) -> ChordQuality {
        if self.notes.len() < 2 {
            return ChordQuality::Other;
        }

        // Regression fix: this used to use `self.notes[0]` (whichever note
        // happened to be added first) as a pseudo-root, so a chord entered
        // in any order other than root-root-third-fifth(-seventh) would be
        // classified from the wrong reference pitch. Use the actually
        // detected root instead.
        let root_pc = match self.root() {
            Some(root) => root.pitch_class(),
            None => return ChordQuality::Other,
        };

        // Get unique pitch classes
        let mut pcs: Vec<u8> = self.pitch_classes();
        pcs.sort();
        pcs.dedup();

        // Calculate intervals from root
        let mut intervals: Vec<u8> = pcs.iter().map(|&pc| (pc + 12 - root_pc) % 12).collect();
        intervals.sort();

        // Check for common chord types
        match intervals.as_slice() {
            // Triads
            [0, 4, 7] => ChordQuality::Major,
            [0, 3, 7] => ChordQuality::Minor,
            [0, 3, 6] => ChordQuality::Diminished,
            [0, 4, 8] => ChordQuality::Augmented,
            [0, 2, 7] => ChordQuality::Suspended2,
            [0, 5, 7] => ChordQuality::Suspended4,
            [0, 7] => ChordQuality::Power,
            // Seventh chords
            [0, 4, 7, 10] => ChordQuality::Dominant,
            [0, 4, 7, 11] => ChordQuality::Major,
            [0, 3, 7, 10] => ChordQuality::Minor,
            [0, 3, 6, 10] => ChordQuality::HalfDiminished,
            [0, 3, 6, 9] => ChordQuality::FullyDiminished,
            _ => ChordQuality::Other,
        }
    }

    /// Check if this is a major triad
    pub fn is_major_triad(&self) -> bool {
        self.quality() == ChordQuality::Major && self.notes.len() == 3
    }

    /// Check if this is a minor triad
    pub fn is_minor_triad(&self) -> bool {
        self.quality() == ChordQuality::Minor && self.notes.len() == 3
    }

    /// Check if this is a diminished triad
    pub fn is_diminished_triad(&self) -> bool {
        self.quality() == ChordQuality::Diminished && self.notes.len() == 3
    }

    /// Check if this is an augmented triad
    pub fn is_augmented_triad(&self) -> bool {
        self.quality() == ChordQuality::Augmented && self.notes.len() == 3
    }

    /// Check if this is a dominant seventh
    pub fn is_dominant_seventh(&self) -> bool {
        self.quality() == ChordQuality::Dominant && self.notes.len() == 4
    }

    /// Get the third of the chord (if present)
    pub fn third(&self) -> Option<&Pitch> {
        self.get_chord_step(3)
    }

    /// Get the fifth of the chord (if present)
    pub fn fifth(&self) -> Option<&Pitch> {
        self.get_chord_step(5)
    }

    /// Get the seventh of the chord (if present)
    pub fn seventh(&self) -> Option<&Pitch> {
        self.get_chord_step(7)
    }

    /// Get a chord step (1 = root, 3 = third, 5 = fifth, etc.)
    pub fn get_chord_step(&self, step: u8) -> Option<&Pitch> {
        let root = self.root()?;
        let root_pc = root.pitch_class();

        // Calculate expected pitch class for the step
        let step_semitones = match step {
            1 => 0,
            2 => 2,
            3 => 3, // could be 3 (minor) or 4 (major)
            4 => 5,
            5 => 7,
            6 => 9,
            7 => 10, // could be 10 (minor) or 11 (major)
            _ => return None,
        };

        // Find pitch closest to expected
        for note in &self.notes {
            let pc = note.pitch().pitch_class();
            let interval = (pc + 12 - root_pc) % 12;

            // Allow for major/minor variants
            if step == 3 && (interval == 3 || interval == 4) {
                return Some(note.pitch());
            }
            if step == 7 && (interval == 10 || interval == 11) {
                return Some(note.pitch());
            }
            if interval == step_semitones {
                return Some(note.pitch());
            }
        }

        None
    }

    /// Transpose the chord
    pub fn transpose(&self, interval: &Interval) -> Chord {
        let notes = self.notes.iter().map(|n| n.transpose(interval)).collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// Transpose by semitones
    pub fn transpose_semitones(&self, semitones: i32) -> Chord {
        let notes = self.notes.iter().map(|n| n.transpose_semitones(semitones)).collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// Get chord symbol (e.g., "Cmaj7", "Dm", "G7")
    pub fn symbol(&self) -> String {
        if let Some(root) = self.root() {
            format!("{}{}", root.name(), self.quality().symbol())
        } else {
            "?".to_string()
        }
    }

    /// Check if the chord contains a specific pitch class
    pub fn contains_pitch_class(&self, pc: u8) -> bool {
        self.pitch_classes().contains(&pc)
    }

    /// Get the interval from bass to root
    pub fn bass_to_root_interval(&self) -> Option<Interval> {
        let bass = self.bass()?;
        let root = self.root()?;

        let semitones = (root.pitch_class() as i32 - bass.pitch_class() as i32).rem_euclid(12);
        Some(Interval::from(semitones))
    }

    // -- Pitch-class-set analysis (previously only reachable as free
    // functions on analysis::chord_analysis::ChordAnalyzer taking a raw
    // &[u8]; now real Chord methods/properties, as in music21). --

    /// The normal order of this chord's pitch classes (the most
    /// left-packed rotation).
    pub fn normal_order_pcs(&self) -> Vec<u8> {
        crate::analysis::ChordAnalyzer::normal_order(&self.ordered_pitch_classes())
    }

    /// The prime form of this chord's pitch-class set (normal order or its
    /// inversion, whichever is more compact, transposed to start at 0).
    pub fn prime_form(&self) -> Vec<u8> {
        crate::analysis::ChordAnalyzer::prime_form(&self.ordered_pitch_classes())
    }

    /// The interval-class vector of this chord's pitch-class set (count of
    /// each interval class 1-6 among all pairs of distinct pitch classes).
    pub fn interval_vector(&self) -> [u8; 6] {
        crate::analysis::ChordAnalyzer::interval_vector(&self.ordered_pitch_classes())
    }

    /// The Forte set-class label (e.g. "3-11" for a major/minor triad,
    /// "4-Z29" for one of the all-interval tetrachords), if this chord's
    /// prime form matches a known trichord or tetrachord (see
    /// `chord_tables` — pentachords and larger aren't covered).
    pub fn forte_class(&self) -> Option<String> {
        let pf = self.prime_form();
        super::chord_tables::forte_class_for_prime_form(&pf).map(|(name, _)| name.to_string())
    }

    /// The numeric part of the Forte class (e.g. 11 for "3-11", 15 for
    /// "4-Z15").
    pub fn forte_class_number(&self) -> Option<u8> {
        let class = self.forte_class()?;
        let (_, rest) = class.split_once('-')?;
        let digits: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse().ok()
    }

    /// The Tn-form Forte class label. This crate's `prime_form` doesn't
    /// track whether the normal order or its inversion was chosen to reach
    /// it, so — unlike music21, which distinguishes the Tn and TnI forms —
    /// this is currently an alias of `forte_class`.
    pub fn forte_class_tn(&self) -> Option<String> {
        self.forte_class()
    }

    /// The TnI-form Forte class label (see `forte_class_tn`'s note on this
    /// crate's simplified single-representative model).
    pub fn forte_class_tn_i(&self) -> Option<String> {
        self.forte_class()
    }

    /// Whether this chord's Forte set-class has a Z-relation partner (a
    /// different prime form sharing the same interval vector). Only
    /// meaningful for the trichords/tetrachords `forte_class` covers.
    pub fn has_z_relation(&self) -> bool {
        let pf = self.prime_form();
        super::chord_tables::forte_class_for_prime_form(&pf)
            .map(|(_, z)| z.is_some())
            .unwrap_or(false)
    }

    /// This chord's Z-relation partner's Forte class label, if any.
    pub fn get_z_relation(&self) -> Option<String> {
        let pf = self.prime_form();
        super::chord_tables::forte_class_for_prime_form(&pf)
            .and_then(|(_, z)| z)
            .map(|s| s.to_string())
    }

    /// Whether this chord and `other` are Z-related: different prime
    /// forms (so not simply transpositions/inversions of each other) that
    /// nonetheless share an identical interval vector.
    pub fn are_z_relations(&self, other: &Chord) -> bool {
        self.prime_form() != other.prime_form() && self.interval_vector() == other.interval_vector()
    }

    /// A human-readable common name for this chord's quality (e.g. "major
    /// triad", "dominant seventh chord"). Falls back to the Forte set-class
    /// label (see `forte_class`) for chords that don't match one of the
    /// familiar tonal qualities `quality()` recognizes, or a generic label
    /// if even that isn't available. This does not attempt music21's full
    /// `chordTables`-based common-name lookup (hundreds of additional
    /// named chord types), which would require a similarly large
    /// hand-verified reference table.
    pub fn common_name(&self) -> String {
        match self.quality() {
            ChordQuality::Major => "major triad".to_string(),
            ChordQuality::Minor => "minor triad".to_string(),
            ChordQuality::Diminished => "diminished triad".to_string(),
            ChordQuality::Augmented => "augmented triad".to_string(),
            ChordQuality::Dominant => "dominant seventh chord".to_string(),
            ChordQuality::HalfDiminished => "half-diminished seventh chord".to_string(),
            ChordQuality::FullyDiminished => "diminished seventh chord".to_string(),
            ChordQuality::Suspended2 => "suspended second chord".to_string(),
            ChordQuality::Suspended4 => "suspended fourth chord".to_string(),
            ChordQuality::Power => "power chord".to_string(),
            ChordQuality::Other => self
                .forte_class()
                .map(|f| format!("set class {f}"))
                .unwrap_or_else(|| "unclassified chord".to_string()),
        }
    }

    /// `common_name()` prefixed with the chord's root pitch name (e.g. "C
    /// major triad").
    pub fn pitched_common_name(&self) -> String {
        match self.root() {
            Some(root) => format!("{} {}", root.name(), self.common_name()),
            None => self.common_name(),
        }
    }

    /// The scale degree (1-7) of each of this chord's pitches within
    /// `key`, in the same order as [`Chord::notes`]/[`Chord::pitches`].
    /// `None` for a pitch that isn't a member of `key`'s scale (a
    /// chromatic/non-diatonic tone).
    pub fn scale_degrees(&self, key: &crate::notation::Key) -> Vec<Option<u8>> {
        let scale = crate::notation::Scale::new(key.tonic().clone(), key.mode());
        self.notes.iter().map(|n| scale.degree_of(n.pitch())).collect()
    }

    /// Whether this chord is a three-distinct-pitch-class chord matching
    /// one of `quality()`'s recognized triad shapes (major/minor/
    /// diminished/augmented/suspended).
    pub fn is_triad(&self) -> bool {
        self.ordered_pitch_classes().len() == 3 && self.quality() != ChordQuality::Other
    }

    /// Whether this chord is a four-distinct-pitch-class chord matching
    /// one of `quality()`'s recognized seventh-chord shapes.
    pub fn is_seventh(&self) -> bool {
        self.ordered_pitch_classes().len() == 4 && self.quality() != ChordQuality::Other
    }

    /// A pragmatic subset of music21's consonance rules: a single pitch or
    /// perfect unison is consonant; a two-pitch-class dyad is consonant if
    /// its interval class isn't a second, tritone, or seventh; a
    /// three-pitch-class chord is consonant only if it's a major or minor
    /// triad (diminished/augmented triads are not); anything larger
    /// (sevenths, clusters) is not. This does not attempt music21's fuller
    /// model of inversions/added-tone consonance.
    pub fn is_consonant(&self) -> bool {
        let pcs = self.ordered_pitch_classes();
        match pcs.len() {
            0 => false,
            1 => true,
            2 => {
                let diff = (pcs[1] as i32 - pcs[0] as i32).rem_euclid(12);
                let interval_class = diff.min(12 - diff);
                matches!(interval_class, 0 | 3 | 4 | 5 | 7 | 8 | 9)
            }
            3 => matches!(self.quality(), ChordQuality::Major | ChordQuality::Minor),
            _ => false,
        }
    }

    /// Whether this chord is just a root and a major third with no fifth
    /// present (e.g. two notes a major third or minor sixth apart).
    pub fn is_incomplete_major_triad(&self) -> bool {
        let pcs = self.ordered_pitch_classes();
        if pcs.len() != 2 {
            return false;
        }
        let diff = (pcs[1] as i32 - pcs[0] as i32).rem_euclid(12);
        diff == 4 || diff == 8
    }

    /// Whether this chord is just a root and a minor third with no fifth
    /// present (e.g. two notes a minor third or major sixth apart).
    pub fn is_incomplete_minor_triad(&self) -> bool {
        let pcs = self.ordered_pitch_classes();
        if pcs.len() != 2 {
            return false;
        }
        let diff = (pcs[1] as i32 - pcs[0] as i32).rem_euclid(12);
        diff == 3 || diff == 9
    }

    /// Find a (bass, augmented-sixth-tone) pair among this chord's
    /// pitches: some note with another a generic sixth above it (letter
    /// distance of a 6th) that's widened to 10 semitones (an augmented
    /// sixth) rather than the usual 8/9 (minor/major sixth) — the
    /// hallmark interval shared by the whole Italian/French/German/Swiss
    /// augmented-sixth family.
    fn augmented_sixth_bass_and_top(&self) -> Option<(Pitch, Pitch)> {
        for a in &self.notes {
            for b in &self.notes {
                let iv = Interval::between(a.pitch(), b.pitch());
                if iv.mod7() == 5 && iv.semitones().rem_euclid(12) == 10 {
                    return Some((a.pitch().clone(), b.pitch().clone()));
                }
            }
        }
        None
    }

    /// Whether any of this chord's pitches sits at the given generic
    /// (letter-distance, 0-indexed) interval and semitone count above
    /// `base` (octave-invariant in both).
    fn has_interval_from(&self, base: &Pitch, generic: i32, semitones: i32) -> bool {
        self.notes.iter().any(|n| {
            let iv = Interval::between(base, n.pitch());
            iv.mod7() == generic.rem_euclid(7) && iv.semitones().rem_euclid(12) == semitones.rem_euclid(12)
        })
    }

    /// Whether this chord contains the hallmark augmented-sixth interval
    /// (a generic sixth widened to 10 semitones) between some pair of its
    /// pitches — true for any member of the Italian/French/German/Swiss
    /// augmented-sixth family.
    pub fn is_augmented_sixth(&self) -> bool {
        self.augmented_sixth_bass_and_top().is_some()
    }

    /// Whether this is an Italian augmented sixth: bass, a major third
    /// above the bass, and the augmented sixth above the bass — no other
    /// pitches (e.g. Ab-C-F# in C major/minor).
    pub fn is_italian_augmented_sixth(&self) -> bool {
        let Some((bass, _)) = self.augmented_sixth_bass_and_top() else {
            return false;
        };
        self.ordered_pitch_classes().len() == 3 && self.has_interval_from(&bass, 2, 4)
    }

    /// Whether this is a French augmented sixth: Italian sixth plus an
    /// augmented fourth above the bass (e.g. Ab-C-D-F# in C major/minor).
    pub fn is_french_augmented_sixth(&self) -> bool {
        let Some((bass, _)) = self.augmented_sixth_bass_and_top() else {
            return false;
        };
        self.ordered_pitch_classes().len() == 4
            && self.has_interval_from(&bass, 2, 4)
            && self.has_interval_from(&bass, 3, 6)
    }

    /// Whether this is a German augmented sixth: Italian sixth plus a
    /// perfect fifth above the bass (e.g. Ab-C-Eb-F# in C major/minor) —
    /// enharmonically identical to a dominant seventh chord.
    pub fn is_german_augmented_sixth(&self) -> bool {
        let Some((bass, _)) = self.augmented_sixth_bass_and_top() else {
            return false;
        };
        self.ordered_pitch_classes().len() == 4
            && self.has_interval_from(&bass, 2, 4)
            && self.has_interval_from(&bass, 4, 7)
    }

    /// Whether this is a Swiss (Alsatian) augmented sixth: Italian sixth
    /// plus a doubly-augmented fourth above the bass — spelled like a
    /// French sixth's fourth but raised an extra semitone (e.g. Ab-C-D#-F#
    /// in C major/minor).
    pub fn is_swiss_augmented_sixth(&self) -> bool {
        let Some((bass, _)) = self.augmented_sixth_bass_and_top() else {
            return false;
        };
        self.ordered_pitch_classes().len() == 4
            && self.has_interval_from(&bass, 2, 4)
            && self.has_interval_from(&bass, 3, 7)
    }

    /// This chord's pitches rearranged into closed position: each pitch
    /// moved into the octave at or immediately above the bass, then
    /// sorted ascending. Mirrors music21's `Chord.closedPosition`.
    pub fn closed_position(&self) -> Chord {
        let Some(bass) = self.bass().cloned() else {
            return self.clone();
        };
        let mut notes: Vec<Note> = self
            .notes
            .iter()
            .map(|n| {
                let mut p = n.pitch().clone();
                while p.midi() < bass.midi() {
                    p = p.transpose(&Interval::octave());
                }
                while p.midi() as i32 >= bass.midi() as i32 + 12 {
                    p = p.transpose(&Interval::octave().reverse());
                }
                let mut note = n.clone();
                note.set_pitch(p);
                note
            })
            .collect();
        notes.sort_by_key(|n| n.pitch().midi());
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// This chord's notes sorted by pitch (ascending), preserving the
    /// pitches' actual octaves (unlike `closed_position`).
    pub fn sort_ascending(&self) -> Chord {
        let mut notes = self.notes.clone();
        notes.sort_by(|a, b| a.pitch().cmp(b.pitch()));
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// A copy of this chord with only the first note kept for each
    /// distinct pitch (identical step, accidental, and octave).
    pub fn remove_redundant_pitches(&self) -> Chord {
        let mut seen: std::collections::HashSet<Pitch> = std::collections::HashSet::new();
        let notes = self
            .notes
            .iter()
            .filter(|n| seen.insert(n.pitch().clone()))
            .cloned()
            .collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// A copy of this chord with only the first note kept for each
    /// distinct pitch class (i.e. octave doublings collapsed).
    pub fn remove_redundant_pitch_classes(&self) -> Chord {
        let mut seen: std::collections::HashSet<u8> = std::collections::HashSet::new();
        let notes = self
            .notes
            .iter()
            .filter(|n| seen.insert(n.pitch().pitch_class()))
            .cloned()
            .collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// A copy of this chord with only the first note kept for each
    /// distinct pitch name (step + accidental, ignoring octave — so e.g.
    /// a C#4 and a C#5 collapse but a C#4 and a Db4 do not).
    pub fn remove_redundant_pitch_names(&self) -> Chord {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let notes = self
            .notes
            .iter()
            .filter(|n| seen.insert(n.pitch().name()))
            .cloned()
            .collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// The realized volume of the first note matching `pitch`, if any
    /// (per-pitch volume/velocity lookup — chord tones can carry
    /// independent dynamics/articulations).
    pub fn volume_for_pitch(&self, pitch: &Pitch) -> Option<Volume> {
        self.notes
            .iter()
            .find(|n| n.pitch() == pitch)
            .map(|n| n.realized_volume())
    }

    /// A label for each note's interval above the bass, in the same order
    /// as [`Chord::notes`] (e.g. `["P1", "M3", "P5"]` for a root-position
    /// major triad). Mirrors music21's `Chord.annotateIntervals`.
    pub fn annotate_intervals(&self) -> Vec<String> {
        let Some(bass) = self.bass().cloned() else {
            return Vec::new();
        };
        self.notes
            .iter()
            .map(|n| Interval::between(&bass, n.pitch()).name())
            .collect()
    }
}

impl Default for Chord {
    fn default() -> Self {
        Self::new(Vec::new(), Duration::quarter())
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pitches: Vec<String> = self.notes.iter().map(|n| n.name()).collect();
        write!(f, "<{}>", pitches.join(" "))
    }
}

impl PartialOrd for Chord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Compare by lowest pitch
        match (self.bass(), other.bass()) {
            (Some(a), Some(b)) => a.partial_cmp(b),
            (Some(_), None) => Some(Ordering::Greater),
            (None, Some(_)) => Some(Ordering::Less),
            (None, None) => Some(Ordering::Equal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Step;

    #[test]
    fn test_chord_major_triad() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);

        assert_eq!(chord.notes.len(), 3);
        assert!(chord.is_major_triad());
        assert_eq!(chord.quality(), ChordQuality::Major);
    }

    #[test]
    fn test_chord_minor_triad() {
        let a = Pitch::from_parts(Step::A, Some(4), None);
        let chord = Chord::minor_triad(a);

        assert!(chord.is_minor_triad());
        assert_eq!(chord.quality(), ChordQuality::Minor);
    }

    #[test]
    fn test_chord_from_strings() {
        let chord = Chord::from_pitch_strings(&["C4", "E4", "G4"], Duration::quarter()).unwrap();
        assert!(chord.is_major_triad());
    }

    #[test]
    fn test_chord_bass_and_root() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c.clone());

        assert_eq!(chord.bass().unwrap().step(), Step::C);
        assert_eq!(chord.root().unwrap().step(), Step::C);
    }

    #[test]
    fn test_chord_dominant_seventh() {
        let g = Pitch::from_parts(Step::G, Some(4), None);
        let chord = Chord::dominant_seventh(g);

        assert_eq!(chord.notes.len(), 4);
        assert!(chord.is_dominant_seventh());
    }

    #[test]
    fn test_chord_root_independent_of_note_entry_order() {
        // Regression test: quality()/root() used to use notes[0] (whichever
        // note was added first) as a pseudo-root, so a first-inversion C
        // major triad entered in E-G-C order would compute intervals from
        // E instead of C and misclassify the chord.
        let e4 = Pitch::from_parts(Step::E, Some(4), None);
        let g4 = Pitch::from_parts(Step::G, Some(4), None);
        let c5 = Pitch::from_parts(Step::C, Some(5), None);
        let chord = Chord::from_pitches(vec![e4, g4, c5], Duration::quarter());

        assert_eq!(chord.root().unwrap().step(), Step::C);
        assert_eq!(chord.quality(), ChordQuality::Major);
        assert!(chord.is_major_triad());
    }

    #[test]
    fn test_chord_diminished_root_in_any_inversion() {
        // Regression test: root() used to require an exact perfect fifth
        // (interval class 7) among stacked thirds to accept a candidate,
        // so it never matched for diminished/augmented chords (whose
        // "fifth" is a tritone/augmented fifth) unless they happened to
        // already be in root position, silently falling back to the bass.
        let eb4 = Pitch::from_parts(Step::E, Some(4), Some(crate::core::Accidental::Flat));
        let gb4 = Pitch::from_parts(Step::G, Some(4), Some(crate::core::Accidental::Flat));
        let c5 = Pitch::from_parts(Step::C, Some(5), None);
        // First inversion of a C diminished triad (C-Eb-Gb), entered
        // starting from the bass (Eb).
        let chord = Chord::from_pitches(vec![eb4, gb4, c5], Duration::quarter());

        assert_eq!(chord.root().unwrap().step(), Step::C);
        assert_eq!(chord.quality(), ChordQuality::Diminished);
        assert!(chord.is_diminished_triad());
    }

    #[test]
    fn test_chord_augmented_root_in_any_inversion() {
        // Augmented triads are intervallically symmetric (major third +
        // major third), so any of the three notes is an equally valid
        // "root" by stacked-thirds alone (this chord is its own first and
        // second inversion, theoretically ambiguous without further
        // spelling context) — but whichever one root() picks, quality()
        // must still correctly identify it as augmented.
        let gs4 = Pitch::from_parts(Step::G, Some(4), Some(crate::core::Accidental::Sharp));
        let c5 = Pitch::from_parts(Step::C, Some(5), None);
        let e5 = Pitch::from_parts(Step::E, Some(5), None);
        let chord = Chord::from_pitches(vec![gs4, c5, e5], Duration::quarter());

        let root_step = chord.root().unwrap().step();
        assert!(matches!(root_step, Step::C | Step::E | Step::G));
        assert_eq!(chord.quality(), ChordQuality::Augmented);
        assert!(chord.is_augmented_triad());
    }

    #[test]
    fn test_chord_transpose() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);
        let transposed = chord.transpose(&Interval::perfect_fifth());

        assert_eq!(transposed.root().unwrap().step(), Step::G);
    }

    #[test]
    fn test_chord_forte_class_major_triad() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);
        assert_eq!(chord.prime_form(), vec![0, 3, 7]);
        assert_eq!(chord.forte_class(), Some("3-11".to_string()));
        assert_eq!(chord.forte_class_number(), Some(11));
        assert!(!chord.has_z_relation());
        assert_eq!(chord.get_z_relation(), None);
    }

    #[test]
    fn test_chord_normal_order_and_interval_vector_match_promoted_methods() {
        // These used to only be reachable as free functions taking a raw
        // &[u8] on analysis::ChordAnalyzer; confirm the promoted Chord
        // methods agree with that underlying implementation.
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::dominant_seventh(c);
        let pcs = chord.ordered_pitch_classes();
        assert_eq!(
            chord.normal_order_pcs(),
            crate::analysis::ChordAnalyzer::normal_order(&pcs)
        );
        assert_eq!(
            chord.interval_vector(),
            crate::analysis::ChordAnalyzer::interval_vector(&pcs)
        );
    }

    #[test]
    fn test_chord_z_related_tetrachords() {
        // 4-Z15 = {0,1,4,6} and 4-Z29 = {0,1,3,7} share an interval vector
        // but are not transpositions/inversions of each other.
        let z15 = Chord::from_pitch_strings(&["C4", "C#4", "E4", "F#4"], Duration::quarter())
            .unwrap();
        let z29 = Chord::from_pitch_strings(&["C4", "C#4", "D#4", "G4"], Duration::quarter())
            .unwrap();

        assert_eq!(z15.forte_class(), Some("4-Z15".to_string()));
        assert_eq!(z29.forte_class(), Some("4-Z29".to_string()));
        assert!(z15.has_z_relation());
        assert_eq!(z15.get_z_relation(), Some("4-Z29".to_string()));
        assert!(z15.are_z_relations(&z29));
    }

    #[test]
    fn test_chord_common_name_and_pitched_common_name() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);
        assert_eq!(chord.common_name(), "major triad");
        assert_eq!(chord.pitched_common_name(), "C major triad");

        let g = Pitch::from_parts(Step::G, Some(4), None);
        let dom7 = Chord::dominant_seventh(g);
        assert_eq!(dom7.common_name(), "dominant seventh chord");
        assert_eq!(dom7.pitched_common_name(), "G dominant seventh chord");
    }

    #[test]
    fn test_chord_forte_class_unsupported_cardinality_is_none() {
        // A 5-note chord isn't covered by the trichord/tetrachord table.
        let chord = Chord::from_pitch_strings(
            &["C4", "D4", "E4", "F#4", "G#4"],
            Duration::quarter(),
        )
        .unwrap();
        assert_eq!(chord.forte_class(), None);
        assert!(!chord.has_z_relation());
    }

    #[test]
    fn test_chord_scale_degrees_in_key() {
        let key = crate::notation::Key::new(
            Pitch::from_parts(Step::C, Some(4), None),
            crate::notation::KeyMode::Major,
        );
        let chord = Chord::from_pitch_strings(&["C4", "E4", "G4"], Duration::quarter()).unwrap();
        assert_eq!(chord.scale_degrees(&key), vec![Some(1), Some(3), Some(5)]);

        // A chromatic tone (not in C major) reports None.
        let chromatic = Chord::from_pitch_strings(&["C4", "D#4"], Duration::quarter()).unwrap();
        assert_eq!(chromatic.scale_degrees(&key), vec![Some(1), None]);
    }

    #[test]
    fn test_chord_is_triad_and_is_seventh() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        assert!(Chord::major_triad(c.clone()).is_triad());
        assert!(!Chord::major_triad(c.clone()).is_seventh());
        assert!(Chord::dominant_seventh(c.clone()).is_seventh());
        assert!(!Chord::dominant_seventh(c).is_triad());

        // A cluster chord (not a recognized triad/seventh shape) is neither.
        let cluster = Chord::from_pitch_strings(&["C4", "C#4", "D4"], Duration::quarter()).unwrap();
        assert!(!cluster.is_triad());
        assert!(!cluster.is_seventh());
    }

    #[test]
    fn test_chord_is_consonant() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        assert!(Chord::major_triad(c.clone()).is_consonant());
        assert!(Chord::minor_triad(c.clone()).is_consonant());
        assert!(!Chord::diminished_triad(c.clone()).is_consonant());
        assert!(!Chord::augmented_triad(c.clone()).is_consonant());
        assert!(!Chord::dominant_seventh(c).is_consonant());

        let fifth_dyad = Chord::from_pitch_strings(&["C4", "G4"], Duration::quarter()).unwrap();
        assert!(fifth_dyad.is_consonant());
        let second_dyad = Chord::from_pitch_strings(&["C4", "D4"], Duration::quarter()).unwrap();
        assert!(!second_dyad.is_consonant());
    }

    #[test]
    fn test_chord_incomplete_triads() {
        let major_third_only =
            Chord::from_pitch_strings(&["C4", "E4"], Duration::quarter()).unwrap();
        assert!(major_third_only.is_incomplete_major_triad());
        assert!(!major_third_only.is_incomplete_minor_triad());

        let minor_third_only =
            Chord::from_pitch_strings(&["C4", "Eb4"], Duration::quarter()).unwrap();
        assert!(minor_third_only.is_incomplete_minor_triad());
        assert!(!minor_third_only.is_incomplete_major_triad());
    }

    #[test]
    fn test_chord_augmented_sixth_family() {
        // All spelled in the "C" tonal context: Ab is the bass (b6),
        // C is the common tone, F# is the augmented-sixth tone.
        let italian =
            Chord::from_pitch_strings(&["Ab3", "C4", "F#4"], Duration::quarter()).unwrap();
        assert!(italian.is_augmented_sixth());
        assert!(italian.is_italian_augmented_sixth());
        assert!(!italian.is_french_augmented_sixth());
        assert!(!italian.is_german_augmented_sixth());
        assert!(!italian.is_swiss_augmented_sixth());

        let french =
            Chord::from_pitch_strings(&["Ab3", "C4", "D4", "F#4"], Duration::quarter()).unwrap();
        assert!(french.is_french_augmented_sixth());
        assert!(!french.is_italian_augmented_sixth());
        assert!(!french.is_german_augmented_sixth());
        assert!(!french.is_swiss_augmented_sixth());

        let german =
            Chord::from_pitch_strings(&["Ab3", "C4", "Eb4", "F#4"], Duration::quarter()).unwrap();
        assert!(german.is_german_augmented_sixth());
        assert!(!german.is_italian_augmented_sixth());
        assert!(!german.is_french_augmented_sixth());
        assert!(!german.is_swiss_augmented_sixth());

        let swiss =
            Chord::from_pitch_strings(&["Ab3", "C4", "D#4", "F#4"], Duration::quarter()).unwrap();
        assert!(swiss.is_swiss_augmented_sixth());
        assert!(!swiss.is_italian_augmented_sixth());
        assert!(!swiss.is_french_augmented_sixth());
        assert!(!swiss.is_german_augmented_sixth());

        // A plain major triad has no augmented sixth interval at all.
        let c = Pitch::from_parts(Step::C, Some(4), None);
        assert!(!Chord::major_triad(c).is_augmented_sixth());
    }

    #[test]
    fn test_chord_closed_position_and_sort_ascending() {
        // Entered as a spread first-inversion triad (E in a low octave,
        // G and C much higher) — closed_position should pack everything
        // within an octave of the bass and sort ascending.
        let spread =
            Chord::from_pitch_strings(&["E3", "C6", "G5"], Duration::quarter()).unwrap();
        let closed = spread.closed_position();
        let names: Vec<String> = closed.pitches().iter().map(|p| p.name()).collect();
        assert_eq!(names, vec!["E", "G", "C"]);
        // Every pitch after closing must be within an octave of the bass.
        let bass_midi = closed.bass().unwrap().midi();
        for p in closed.pitches() {
            assert!(p.midi() >= bass_midi && (p.midi() as i32) < bass_midi as i32 + 12);
        }

        let ascending = spread.sort_ascending();
        let midis: Vec<u8> = ascending.pitches().iter().map(|p| p.midi()).collect();
        let mut sorted_midis = midis.clone();
        sorted_midis.sort();
        assert_eq!(midis, sorted_midis);
    }

    #[test]
    fn test_chord_remove_redundant_variants() {
        let doubled_pitch =
            Chord::from_pitch_strings(&["C4", "E4", "G4", "C4"], Duration::quarter()).unwrap();
        assert_eq!(doubled_pitch.remove_redundant_pitches().notes().len(), 3);

        let doubled_pc =
            Chord::from_pitch_strings(&["C4", "E4", "G4", "C5"], Duration::quarter()).unwrap();
        assert_eq!(doubled_pc.remove_redundant_pitch_classes().notes().len(), 3);
        // But the octave-doubled note is preserved when only exact-pitch
        // (not pitch-class) redundancy is removed.
        assert_eq!(doubled_pc.remove_redundant_pitches().notes().len(), 4);

        let doubled_name =
            Chord::from_pitch_strings(&["C#4", "C#5", "E4"], Duration::quarter()).unwrap();
        assert_eq!(doubled_name.remove_redundant_pitch_names().notes().len(), 2);
    }

    #[test]
    fn test_chord_annotate_intervals() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);
        assert_eq!(chord.annotate_intervals(), vec!["P1", "M3", "P5"]);
    }

    #[test]
    fn test_chord_volume_for_pitch() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c.clone());
        assert!(chord.volume_for_pitch(&c).is_some());
        let unrelated = Pitch::from_parts(Step::B, Some(4), None);
        assert_eq!(chord.volume_for_pitch(&unrelated), None);
    }
}
