//! Score rendering module
//!
//! This module provides graphical rendering of musical scores using mkgraphic.
//!
//! # Components
//!
//! - [`ScoreRenderer`] - Main renderer for full scores
//! - [`StaffElement`] - Element for rendering a single staff
//! - [`NoteElement`] - Element for rendering notes
//! - [`ClefElement`] - Element for rendering clefs

mod staff;
mod note;
mod clef;
mod measure;
mod config;
mod elements;

pub use staff::StaffElement;
pub use note::NoteElement;
pub use clef::ClefElement;
pub use measure::MeasureElement;
pub use config::{RenderConfig, StaffConfig};
pub use elements::{ScoreElement, render_score_to_image};

use mkgraphic::support::canvas::Canvas;

use crate::stream::Score;
use crate::notation::Clef;

/// Staff line spacing constant (in points)
pub const STAFF_SPACE: f32 = 8.0;

/// Default staff height (5 lines = 4 spaces)
pub const STAFF_HEIGHT: f32 = STAFF_SPACE * 4.0;

/// Ledger line extension beyond notehead
pub const LEDGER_EXTENSION: f32 = 4.0;

/// Note positions relative to staff
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StaffPosition {
    /// Line/space position (0 = middle line, positive = up)
    pub position: i8,
    /// Accidental offset in semitones
    pub accidental: i8,
}

impl StaffPosition {
    pub fn new(position: i8, accidental: i8) -> Self {
        Self { position, accidental }
    }

    /// Convert to Y coordinate relative to staff center
    pub fn to_y(&self, staff_space: f32) -> f32 {
        -(self.position as f32) * (staff_space / 2.0)
    }
}

/// Score renderer that creates mkgraphic elements from a Score
pub struct ScoreRenderer {
    config: RenderConfig,
}

impl ScoreRenderer {
    /// Create a new score renderer with default configuration
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
        }
    }

    /// Create a new score renderer with custom configuration
    pub fn with_config(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut RenderConfig {
        &mut self.config
    }

    /// Render a score to a ScoreElement
    pub fn render(&self, score: &Score) -> ScoreElement {
        ScoreElement::new(score, self.config.clone())
    }

    /// Render a score directly to a canvas
    pub fn render_to_canvas(&self, score: &Score, canvas: &mut Canvas) {
        let element = self.render(score);
        element.draw_to_canvas(canvas, &self.config);
    }

    /// Calculate the required canvas size for a score
    pub fn calculate_size(&self, score: &Score) -> (u32, u32) {
        let num_parts = score.parts().len();
        let num_measures = score.parts().first().map(|p| p.measures().len()).unwrap_or(0);

        let width = self.config.margin_left
            + self.config.clef_width
            + self.config.key_sig_width
            + self.config.time_sig_width
            + (num_measures as f32 * self.config.measure_width)
            + self.config.margin_right;

        let staff_with_spacing = self.config.staff.height + self.config.staff_spacing;
        let height = self.config.margin_top
            + (num_parts as f32 * staff_with_spacing)
            + self.config.margin_bottom;

        (width as u32, height as u32)
    }
}

impl Default for ScoreRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert MIDI note number to staff position for a given clef
pub fn midi_to_staff_position(midi: u8, clef: &Clef) -> StaffPosition {
    // Staff position convention:
    // - Position 0 = middle line of staff (line 3)
    // - Position 4 = top line
    // - Position -4 = bottom line
    // - Each line/space increments by 1
    //
    // In treble clef:
    // - G4 (67) on line 2 = position -2
    // - Middle C (60) is on first ledger line below = position -6
    //
    // In bass clef:
    // - F3 (53) on line 4 = position 2

    let reference_midi = clef.reference_pitch() as i8;

    // Convert clef line (1-5 from bottom) to staff position
    // Line 1 = position -4 (bottom)
    // Line 2 = position -2
    // Line 3 = position 0 (middle)
    // Line 4 = position 2
    // Line 5 = position 4 (top)
    let reference_position = (clef.line() as i8 - 3) * 2;

    // Calculate MIDI difference
    let midi_diff = midi as i8 - reference_midi;

    // Convert chromatic interval to diatonic steps
    // This is approximate - accidentals may introduce slight errors
    let octaves = midi_diff / 12;
    let semitones_in_octave = (midi_diff % 12 + 12) % 12; // Handle negative values

    // Map semitones within octave to diatonic steps (0-6)
    // C=0, D=2, E=4, F=5, G=7, A=9, B=11
    let diatonic_step = match semitones_in_octave {
        0 => 0,
        1 | 2 => 1,
        3 | 4 => 2,
        5 => 3,
        6 | 7 => 4,
        8 | 9 => 5,
        10 | 11 => 6,
        _ => 0,
    };

    // Calculate total diatonic steps
    let total_steps = if midi_diff >= 0 {
        octaves * 7 + diatonic_step as i8
    } else {
        octaves * 7 - if diatonic_step > 0 { 7 - diatonic_step as i8 } else { 0 }
    };

    // Position increases upward, so higher pitch = higher position
    let position = reference_position + total_steps;

    // Calculate accidental (simplified)
    let expected_semitones = match diatonic_step {
        0 => 0,
        1 => 2,
        2 => 4,
        3 => 5,
        4 => 7,
        5 => 9,
        6 => 11,
        _ => 0,
    };
    let accidental = (semitones_in_octave - expected_semitones) as i8;

    StaffPosition::new(position, accidental)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staff_position_to_y() {
        let pos = StaffPosition::new(0, 0);
        assert_eq!(pos.to_y(STAFF_SPACE), 0.0);

        let pos = StaffPosition::new(2, 0);
        assert_eq!(pos.to_y(STAFF_SPACE), -STAFF_SPACE);

        let pos = StaffPosition::new(-2, 0);
        assert_eq!(pos.to_y(STAFF_SPACE), STAFF_SPACE);
    }

    #[test]
    fn test_midi_to_staff_position() {
        let treble = Clef::treble();

        // Middle C in treble clef should be below the staff
        let pos = midi_to_staff_position(60, &treble);
        assert!(pos.position < -4); // Below bottom line
    }
}
