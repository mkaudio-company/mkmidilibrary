//! Stream base types
//!
//! A Stream is an ordered collection of music elements with timing information.

use std::fmt;

use crate::core::{Chord, Duration, Fraction, Note, Rest};

/// A music element that can be stored in a stream
#[derive(Debug, Clone, PartialEq)]
pub enum MusicElement {
    Note(Note),
    Chord(Chord),
    Rest(Rest),
}

impl MusicElement {
    /// Get the duration of this element
    pub fn duration(&self) -> &Duration {
        match self {
            MusicElement::Note(n) => n.duration(),
            MusicElement::Chord(c) => c.duration(),
            MusicElement::Rest(r) => r.duration(),
        }
    }

    /// Get the quarter length of this element
    pub fn quarter_length(&self) -> Fraction {
        match self {
            MusicElement::Note(n) => n.quarter_length(),
            MusicElement::Chord(c) => c.quarter_length(),
            MusicElement::Rest(r) => r.quarter_length(),
        }
    }

    /// Check if this is a note
    pub fn is_note(&self) -> bool {
        matches!(self, MusicElement::Note(_))
    }

    /// Check if this is a chord
    pub fn is_chord(&self) -> bool {
        matches!(self, MusicElement::Chord(_))
    }

    /// Check if this is a rest
    pub fn is_rest(&self) -> bool {
        matches!(self, MusicElement::Rest(_))
    }

    /// Get as note (if this is a note)
    pub fn as_note(&self) -> Option<&Note> {
        match self {
            MusicElement::Note(n) => Some(n),
            _ => None,
        }
    }

    /// Get as chord (if this is a chord)
    pub fn as_chord(&self) -> Option<&Chord> {
        match self {
            MusicElement::Chord(c) => Some(c),
            _ => None,
        }
    }

    /// Get as rest (if this is a rest)
    pub fn as_rest(&self) -> Option<&Rest> {
        match self {
            MusicElement::Rest(r) => Some(r),
            _ => None,
        }
    }

    /// Get mutable note
    pub fn as_note_mut(&mut self) -> Option<&mut Note> {
        match self {
            MusicElement::Note(n) => Some(n),
            _ => None,
        }
    }

    /// Get mutable chord
    pub fn as_chord_mut(&mut self) -> Option<&mut Chord> {
        match self {
            MusicElement::Chord(c) => Some(c),
            _ => None,
        }
    }

    /// Get mutable rest
    pub fn as_rest_mut(&mut self) -> Option<&mut Rest> {
        match self {
            MusicElement::Rest(r) => Some(r),
            _ => None,
        }
    }
}

impl From<Note> for MusicElement {
    fn from(note: Note) -> Self {
        MusicElement::Note(note)
    }
}

impl From<Chord> for MusicElement {
    fn from(chord: Chord) -> Self {
        MusicElement::Chord(chord)
    }
}

impl From<Rest> for MusicElement {
    fn from(rest: Rest) -> Self {
        MusicElement::Rest(rest)
    }
}

impl fmt::Display for MusicElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicElement::Note(n) => write!(f, "{}", n),
            MusicElement::Chord(c) => write!(f, "{}", c),
            MusicElement::Rest(r) => write!(f, "{}", r),
        }
    }
}

/// Trait for elements that can be stored in a stream
pub trait StreamElement: Clone {
    /// Get the offset of this element
    fn offset(&self) -> Fraction;

    /// Set the offset of this element
    fn set_offset(&mut self, offset: Fraction);

    /// Get the duration
    fn duration(&self) -> Fraction;

    /// Get the priority for sorting (lower = earlier at same offset)
    fn priority(&self) -> i32 {
        0
    }
}

/// A stream of music elements
#[derive(Debug, Clone, Default)]
pub struct Stream {
    /// Elements with their offsets
    elements: Vec<(Fraction, MusicElement)>,
    /// Elements stored at the end (barlines, etc.)
    end_elements: Vec<MusicElement>,
    /// Whether to auto-sort on modification
    auto_sort: bool,
    /// Whether elements are currently sorted
    is_sorted: bool,
}

impl Stream {
    /// Create a new empty stream
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            end_elements: Vec::new(),
            auto_sort: true,
            is_sorted: true,
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the stream is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get all elements with their offsets
    pub fn elements(&self) -> &[(Fraction, MusicElement)] {
        &self.elements
    }

    /// Get mutable elements
    pub fn elements_mut(&mut self) -> &mut Vec<(Fraction, MusicElement)> {
        self.is_sorted = false;
        &mut self.elements
    }

    /// Append an element at the end of the stream
    pub fn append(&mut self, element: MusicElement) {
        let offset = self.highest_time();
        self.elements.push((offset, element));
        // Still sorted if appending at end
    }

    /// Insert an element at a specific offset
    pub fn insert(&mut self, offset: Fraction, element: MusicElement) {
        self.elements.push((offset, element));
        self.is_sorted = false;

        if self.auto_sort {
            self.sort();
        }
    }

    /// Remove an element at a specific index
    pub fn remove(&mut self, index: usize) -> Option<(Fraction, MusicElement)> {
        if index < self.elements.len() {
            Some(self.elements.remove(index))
        } else {
            None
        }
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
        self.end_elements.clear();
        self.is_sorted = true;
    }

    /// Sort elements by offset
    pub fn sort(&mut self) {
        if !self.is_sorted {
            self.elements.sort_by(|(a, _), (b, _)| a.cmp(b));
            self.is_sorted = true;
        }
    }

    /// Check if sorted
    pub fn is_sorted(&self) -> bool {
        self.is_sorted
    }

    /// Set auto-sort behavior
    pub fn set_auto_sort(&mut self, auto_sort: bool) {
        self.auto_sort = auto_sort;
    }

    /// Get the highest offset in the stream
    pub fn highest_offset(&self) -> Fraction {
        self.elements
            .iter()
            .map(|(offset, _)| *offset)
            .max()
            .unwrap_or(Fraction::new(0, 1))
    }

    /// Get the highest time (offset + duration) in the stream
    pub fn highest_time(&self) -> Fraction {
        self.elements
            .iter()
            .map(|(offset, elem)| *offset + elem.quarter_length())
            .max()
            .unwrap_or(Fraction::new(0, 1))
    }

    /// Get the lowest offset in the stream
    pub fn lowest_offset(&self) -> Fraction {
        self.elements
            .iter()
            .map(|(offset, _)| *offset)
            .min()
            .unwrap_or(Fraction::new(0, 1))
    }

    /// Get the total duration of the stream
    pub fn duration(&self) -> Fraction {
        self.highest_time() - self.lowest_offset()
    }

    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &(Fraction, MusicElement)> {
        self.elements.iter()
    }

    /// Iterate over just the music elements
    pub fn iter_elements(&self) -> impl Iterator<Item = &MusicElement> {
        self.elements.iter().map(|(_, elem)| elem)
    }

    /// Iterate over notes only
    pub fn notes(&self) -> impl Iterator<Item = &Note> {
        self.elements.iter().filter_map(|(_, elem)| elem.as_note())
    }

    /// Iterate over chords only
    pub fn chords(&self) -> impl Iterator<Item = &Chord> {
        self.elements.iter().filter_map(|(_, elem)| elem.as_chord())
    }

    /// Iterate over rests only
    pub fn rests(&self) -> impl Iterator<Item = &Rest> {
        self.elements.iter().filter_map(|(_, elem)| elem.as_rest())
    }

    /// Get elements at a specific offset
    pub fn elements_at_offset(&self, offset: Fraction) -> impl Iterator<Item = &MusicElement> {
        self.elements
            .iter()
            .filter(move |(o, _)| *o == offset)
            .map(|(_, elem)| elem)
    }

    /// Shift all elements by an offset
    pub fn shift_elements(&mut self, shift: Fraction) {
        for (offset, _) in &mut self.elements {
            *offset = *offset + shift;
        }
    }

    /// Scale all offsets by a factor
    pub fn scale_offsets(&mut self, scale: Fraction) {
        for (offset, _) in &mut self.elements {
            *offset = *offset * scale;
        }
    }

    /// Store an element at the end (for barlines, etc.)
    pub fn store_at_end(&mut self, element: MusicElement) {
        self.end_elements.push(element);
    }

    /// Get end elements
    pub fn end_elements(&self) -> &[MusicElement] {
        &self.end_elements
    }

    /// Get the first element
    pub fn first(&self) -> Option<&MusicElement> {
        self.elements.first().map(|(_, elem)| elem)
    }

    /// Get the last element
    pub fn last(&self) -> Option<&MusicElement> {
        self.elements.last().map(|(_, elem)| elem)
    }

    /// Get element at index
    pub fn get(&self, index: usize) -> Option<&(Fraction, MusicElement)> {
        self.elements.get(index)
    }

    /// Get mutable element at index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut (Fraction, MusicElement)> {
        self.elements.get_mut(index)
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream({} elements)", self.elements.len())
    }
}

impl IntoIterator for Stream {
    type Item = (Fraction, MusicElement);
    type IntoIter = std::vec::IntoIter<(Fraction, MusicElement)>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a Stream {
    type Item = &'a (Fraction, MusicElement);
    type IntoIter = std::slice::Iter<'a, (Fraction, MusicElement)>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl FromIterator<(Fraction, MusicElement)> for Stream {
    fn from_iter<T: IntoIterator<Item = (Fraction, MusicElement)>>(iter: T) -> Self {
        let mut stream = Stream::new();
        for (offset, element) in iter {
            stream.insert(offset, element);
        }
        stream.sort();
        stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Pitch, Step};

    fn make_note() -> Note {
        Note::quarter(Pitch::from_parts(Step::C, Some(4), None))
    }

    #[test]
    fn test_stream_creation() {
        let stream = Stream::new();
        assert!(stream.is_empty());
        assert_eq!(stream.len(), 0);
    }

    #[test]
    fn test_stream_append() {
        let mut stream = Stream::new();
        stream.append(MusicElement::Note(make_note()));
        stream.append(MusicElement::Note(make_note()));

        assert_eq!(stream.len(), 2);
        assert_eq!(stream.elements()[0].0, Fraction::new(0, 1));
        assert_eq!(stream.elements()[1].0, Fraction::new(1, 1)); // After quarter note
    }

    #[test]
    fn test_stream_insert() {
        let mut stream = Stream::new();
        stream.insert(Fraction::new(2, 1), MusicElement::Note(make_note()));
        stream.insert(Fraction::new(0, 1), MusicElement::Note(make_note()));
        stream.insert(Fraction::new(1, 1), MusicElement::Note(make_note()));

        assert!(stream.is_sorted());
        assert_eq!(stream.elements()[0].0, Fraction::new(0, 1));
        assert_eq!(stream.elements()[1].0, Fraction::new(1, 1));
        assert_eq!(stream.elements()[2].0, Fraction::new(2, 1));
    }

    #[test]
    fn test_stream_duration() {
        let mut stream = Stream::new();
        stream.append(MusicElement::Note(make_note()));
        stream.append(MusicElement::Note(make_note()));

        assert_eq!(stream.duration(), Fraction::new(2, 1));
    }

    #[test]
    fn test_stream_shift() {
        let mut stream = Stream::new();
        stream.insert(Fraction::new(0, 1), MusicElement::Note(make_note()));
        stream.insert(Fraction::new(1, 1), MusicElement::Note(make_note()));

        stream.shift_elements(Fraction::new(2, 1));

        assert_eq!(stream.elements()[0].0, Fraction::new(2, 1));
        assert_eq!(stream.elements()[1].0, Fraction::new(3, 1));
    }
}
