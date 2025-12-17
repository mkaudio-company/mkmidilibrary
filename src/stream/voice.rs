//! Voice representation
//!
//! A Voice represents a single melodic line within a measure,
//! used when multiple voices share the same staff.

use std::fmt;

use crate::core::Fraction;

use super::base::{MusicElement, Stream};

/// A voice within a measure
#[derive(Debug, Clone)]
pub struct Voice {
    /// Voice ID (typically 1-4)
    id: u8,
    /// The stream of music elements
    stream: Stream,
}

impl Voice {
    /// Create a new voice
    pub fn new(id: u8) -> Self {
        Self {
            id,
            stream: Stream::new(),
        }
    }

    /// Get the voice ID
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Set the voice ID
    pub fn set_id(&mut self, id: u8) {
        self.id = id;
    }

    /// Get the stream
    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    /// Get mutable stream
    pub fn stream_mut(&mut self) -> &mut Stream {
        &mut self.stream
    }

    /// Get elements
    pub fn elements(&self) -> &[(Fraction, MusicElement)] {
        self.stream.elements()
    }

    /// Append an element
    pub fn append(&mut self, element: MusicElement) {
        self.stream.append(element);
    }

    /// Insert an element at an offset
    pub fn insert(&mut self, offset: Fraction, element: MusicElement) {
        self.stream.insert(offset, element);
    }

    /// Get the duration
    pub fn duration(&self) -> Fraction {
        self.stream.duration()
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.stream.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.stream.clear();
    }

    /// Iterate over notes
    pub fn notes(&self) -> impl Iterator<Item = &crate::core::Note> {
        self.stream.notes()
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self::new(1)
    }
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Voice {} ({} elements)", self.id, self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Note, Pitch, Step};

    #[test]
    fn test_voice_creation() {
        let voice = Voice::new(1);
        assert_eq!(voice.id(), 1);
        assert!(voice.is_empty());
    }

    #[test]
    fn test_voice_append() {
        let mut voice = Voice::new(1);
        let note = Note::quarter(Pitch::from_parts(Step::C, Some(4), None));
        voice.append(MusicElement::Note(note));

        assert_eq!(voice.len(), 1);
    }
}
