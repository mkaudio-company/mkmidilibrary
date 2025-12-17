//! Measure rendering element

use std::any::Any;

use mkgraphic::prelude::*;
use mkgraphic::support::canvas::Canvas;
use num::{ToPrimitive, Zero};

use super::config::RenderConfig;
use super::note::NoteElement;
use super::staff::{draw_bar_line, draw_double_bar_line, StaffElement};
use super::{midi_to_staff_position, STAFF_SPACE, STAFF_HEIGHT};
use crate::core::Fraction;
use crate::notation::{Clef, KeySignature, TimeSignature};
use crate::stream::{Measure, MusicElement};

/// A graphical element representing a measure
pub struct MeasureElement {
    /// Width of the measure
    width: f32,
    /// X position
    x: f32,
    /// Y position (staff center)
    staff_y: f32,
    /// Clef for this measure
    clef: Clef,
    /// Time signature (if displayed)
    time_signature: Option<TimeSignature>,
    /// Key signature (if displayed)
    key_signature: Option<KeySignature>,
    /// Measure number
    number: u32,
    /// Whether this is the last measure
    is_last: bool,
}

impl MeasureElement {
    /// Create a new measure element
    pub fn new(width: f32, clef: Clef) -> Self {
        Self {
            width,
            x: 0.0,
            staff_y: 0.0,
            clef,
            time_signature: None,
            key_signature: None,
            number: 1,
            is_last: false,
        }
    }

    /// Set position
    pub fn set_position(&mut self, x: f32, staff_y: f32) {
        self.x = x;
        self.staff_y = staff_y;
    }

    /// Set time signature
    pub fn set_time_signature(&mut self, ts: TimeSignature) {
        self.time_signature = Some(ts);
    }

    /// Set key signature
    pub fn set_key_signature(&mut self, ks: KeySignature) {
        self.key_signature = Some(ks);
    }

    /// Set measure number
    pub fn set_number(&mut self, number: u32) {
        self.number = number;
    }

    /// Set whether this is the last measure
    pub fn set_last(&mut self, is_last: bool) {
        self.is_last = is_last;
    }

    /// Draw the measure from a Measure struct
    pub fn draw_measure(
        &self,
        canvas: &mut Canvas,
        measure: &Measure,
        config: &RenderConfig,
    ) {
        let content_start = self.x;
        let content_width = self.width;

        // Calculate total duration of the measure
        let total_duration = measure.duration();

        // Draw each element in the measure
        for (offset, element) in measure.elements() {
            let x = self.offset_to_x(offset, &total_duration, content_start, content_width);

            match element {
                MusicElement::Note(note) => {
                    let midi = note.midi();
                    let position = midi_to_staff_position(midi, &self.clef);
                    let mut note_element = NoteElement::new(note.clone(), position);
                    note_element.set_position(x, self.staff_y);
                    note_element.draw_to_canvas(canvas, config);

                    // Draw ledger lines if needed
                    if config.show_ledger_lines && (position.position > 4 || position.position < -4) {
                        let staff = StaffElement::new(self.width);
                        staff.draw_ledger_lines(
                            canvas,
                            position.position,
                            x,
                            config.note.head_width,
                            &config.colors.staff_lines,
                        );
                    }
                }
                MusicElement::Rest(rest) => {
                    self.draw_rest(canvas, x, rest.duration(), config);
                }
                MusicElement::Chord(chord) => {
                    // Draw each note in the chord
                    for note in chord.notes() {
                        let midi = note.midi();
                        let position = midi_to_staff_position(midi, &self.clef);
                        let mut note_element = NoteElement::new(note.clone(), position);
                        note_element.set_position(x, self.staff_y);
                        note_element.draw_to_canvas(canvas, config);
                    }
                }
            }
        }

        // Draw bar line at the end
        let bar_x = self.x + self.width;
        let top_y = self.staff_y - STAFF_HEIGHT / 2.0;
        let bottom_y = self.staff_y + STAFF_HEIGHT / 2.0;

        if self.is_last {
            draw_double_bar_line(
                canvas,
                bar_x - 6.0,
                top_y,
                bottom_y,
                1.0,
                3.0,
                4.0,
                &config.colors.bar_lines,
            );
        } else {
            draw_bar_line(canvas, bar_x, top_y, bottom_y, 1.0, &config.colors.bar_lines);
        }
    }

    /// Convert a time offset to X coordinate
    fn offset_to_x(
        &self,
        offset: &Fraction,
        total_duration: &Fraction,
        content_start: f32,
        content_width: f32,
    ) -> f32 {
        if total_duration.is_zero() {
            return content_start;
        }

        let ratio = (*offset / *total_duration).to_f32().unwrap_or(0.0);
        content_start + ratio * content_width * 0.9 // Leave some space at the end
    }

    /// Draw a rest
    fn draw_rest(
        &self,
        canvas: &mut Canvas,
        x: f32,
        duration: &crate::core::Duration,
        config: &RenderConfig,
    ) {
        use crate::core::DurationType;

        let colors = &config.colors.rests;
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.fill_style(color);
        canvas.line_width(2.0);

        let s = STAFF_SPACE;
        let cy = self.staff_y;

        match duration.type_() {
            Some(DurationType::Whole) => {
                // Whole rest: rectangle hanging from line 4
                let rect = Rect::new(
                    x,
                    cy - s * 0.5 - s * 0.3,
                    x + s * 1.5,
                    cy - s * 0.5,
                );
                canvas.fill_rect(rect);
            }
            Some(DurationType::Half) => {
                // Half rest: rectangle sitting on line 3
                let rect = Rect::new(x, cy, x + s * 1.5, cy + s * 0.3);
                canvas.fill_rect(rect);
            }
            Some(DurationType::Quarter) => {
                // Quarter rest: squiggle shape
                canvas.begin_path();
                canvas.move_to(Point::new(x + s * 0.3, cy - s));
                canvas.line_to(Point::new(x, cy - s * 0.5));
                canvas.line_to(Point::new(x + s * 0.5, cy));
                canvas.line_to(Point::new(x + s * 0.2, cy + s * 0.5));
                canvas.line_to(Point::new(x + s * 0.5, cy + s));
                canvas.stroke();
            }
            Some(DurationType::Eighth) => {
                // Eighth rest: flag with stem
                canvas.begin_path();
                canvas.move_to(Point::new(x + s * 0.3, cy - s * 0.5));
                canvas.line_to(Point::new(x, cy + s * 0.5));
                canvas.stroke();

                // Flag
                canvas.begin_path();
                canvas.add_circle(mkgraphic::support::circle::Circle::new(
                    Point::new(x + s * 0.3, cy - s * 0.3),
                    s * 0.2,
                ));
                canvas.fill();
            }
            Some(DurationType::N16th) | Some(DurationType::N32nd) => {
                // Similar to eighth with more flags
                canvas.begin_path();
                canvas.move_to(Point::new(x + s * 0.3, cy - s * 0.8));
                canvas.line_to(Point::new(x, cy + s * 0.8));
                canvas.stroke();

                // Multiple flags
                let num_flags = match duration.type_() {
                    Some(DurationType::N16th) => 2,
                    Some(DurationType::N32nd) => 3,
                    _ => 1,
                };

                for i in 0..num_flags {
                    let flag_y = cy - s * 0.5 + (i as f32 * s * 0.4);
                    canvas.begin_path();
                    canvas.add_circle(mkgraphic::support::circle::Circle::new(
                        Point::new(x + s * 0.3, flag_y),
                        s * 0.15,
                    ));
                    canvas.fill();
                }
            }
            _ => {
                // Default: draw a simple symbol
                canvas.begin_path();
                canvas.move_to(Point::new(x, cy - s * 0.5));
                canvas.line_to(Point::new(x + s * 0.5, cy + s * 0.5));
                canvas.stroke();
            }
        }

        // Draw dots
        let dots = duration.dots();
        if dots > 0 {
            for i in 0..dots {
                let dot_x = x + s * 2.0 + (i as f32 * s * 0.5);
                canvas.begin_path();
                canvas.add_circle(mkgraphic::support::circle::Circle::new(
                    Point::new(dot_x, cy),
                    s * 0.1,
                ));
                canvas.fill();
            }
        }
    }

    /// Draw time signature
    pub fn draw_time_signature(&self, canvas: &mut Canvas, x: f32, config: &RenderConfig) {
        if let Some(ref ts) = self.time_signature {
            let colors = &config.colors.notes;
            let color = Color::new(colors.0, colors.1, colors.2, colors.3);
            canvas.stroke_style(color);
            canvas.fill_style(color);
            canvas.line_width(2.0);

            let s = STAFF_SPACE;
            let num = ts.numerator();
            let den = ts.denominator();

            // Draw numerator (above center)
            self.draw_number(canvas, num, x, self.staff_y - s, s * 1.5);

            // Draw denominator (below center)
            self.draw_number(canvas, den, x, self.staff_y + s, s * 1.5);
        }
    }

    /// Draw a number for time signatures
    fn draw_number(&self, canvas: &mut Canvas, num: u8, x: f32, y: f32, size: f32) {
        // Simplified number drawing
        // In a real implementation, this would use font rendering
        canvas.line_width(size * 0.15);

        match num {
            1 => {
                canvas.begin_path();
                canvas.move_to(Point::new(x + size * 0.3, y - size * 0.4));
                canvas.line_to(Point::new(x + size * 0.5, y - size * 0.5));
                canvas.line_to(Point::new(x + size * 0.5, y + size * 0.5));
                canvas.stroke();
            }
            2 => {
                canvas.begin_path();
                canvas.move_to(Point::new(x + size * 0.2, y - size * 0.3));
                canvas.line_to(Point::new(x + size * 0.5, y - size * 0.5));
                canvas.line_to(Point::new(x + size * 0.8, y - size * 0.3));
                canvas.line_to(Point::new(x + size * 0.2, y + size * 0.4));
                canvas.line_to(Point::new(x + size * 0.8, y + size * 0.5));
                canvas.stroke();
            }
            3 => {
                canvas.begin_path();
                canvas.move_to(Point::new(x + size * 0.2, y - size * 0.4));
                canvas.line_to(Point::new(x + size * 0.8, y - size * 0.4));
                canvas.line_to(Point::new(x + size * 0.5, y));
                canvas.line_to(Point::new(x + size * 0.8, y + size * 0.4));
                canvas.line_to(Point::new(x + size * 0.2, y + size * 0.4));
                canvas.stroke();
            }
            4 => {
                canvas.begin_path();
                canvas.move_to(Point::new(x + size * 0.7, y + size * 0.5));
                canvas.line_to(Point::new(x + size * 0.7, y - size * 0.5));
                canvas.line_to(Point::new(x + size * 0.2, y + size * 0.2));
                canvas.line_to(Point::new(x + size * 0.9, y + size * 0.2));
                canvas.stroke();
            }
            6 => {
                canvas.begin_path();
                canvas.move_to(Point::new(x + size * 0.7, y - size * 0.4));
                canvas.line_to(Point::new(x + size * 0.3, y - size * 0.4));
                canvas.line_to(Point::new(x + size * 0.2, y + size * 0.3));
                canvas.line_to(Point::new(x + size * 0.7, y + size * 0.3));
                canvas.line_to(Point::new(x + size * 0.7, y));
                canvas.line_to(Point::new(x + size * 0.3, y));
                canvas.stroke();
            }
            8 => {
                // Draw two circles
                canvas.begin_path();
                canvas.add_circle(mkgraphic::support::circle::Circle::new(
                    Point::new(x + size * 0.5, y - size * 0.25),
                    size * 0.2,
                ));
                canvas.stroke();
                canvas.begin_path();
                canvas.add_circle(mkgraphic::support::circle::Circle::new(
                    Point::new(x + size * 0.5, y + size * 0.25),
                    size * 0.25,
                ));
                canvas.stroke();
            }
            _ => {
                // Default: just draw the digit outline
                canvas.begin_path();
                canvas.add_circle(mkgraphic::support::circle::Circle::new(
                    Point::new(x + size * 0.5, y),
                    size * 0.3,
                ));
                canvas.stroke();
            }
        }
    }

    /// Draw key signature
    pub fn draw_key_signature(&self, canvas: &mut Canvas, x: f32, config: &RenderConfig) {
        if let Some(ref ks) = self.key_signature {
            let colors = &config.colors.accidentals;
            let sharps = ks.sharps();
            let s = STAFF_SPACE;

            if sharps > 0 {
                // Draw sharps
                // Standard order: F C G D A E B
                let positions: [i8; 7] = [4, 1, 5, 2, -1, 3, 0]; // Staff positions for sharps
                for i in 0..(sharps as usize).min(7) {
                    let pos = positions[i];
                    let sharp_x = x + (i as f32 * s * 0.8);
                    let sharp_y = self.staff_y - (pos as f32 * s / 2.0);
                    self.draw_sharp_symbol(canvas, sharp_x, sharp_y, colors);
                }
            } else if sharps < 0 {
                // Draw flats
                // Standard order: B E A D G C F
                let positions: [i8; 7] = [0, 3, -1, 2, -2, 1, -3]; // Staff positions for flats
                let num_flats = (-sharps) as usize;
                for i in 0..num_flats.min(7) {
                    let pos = positions[i];
                    let flat_x = x + (i as f32 * s * 0.8);
                    let flat_y = self.staff_y - (pos as f32 * s / 2.0);
                    self.draw_flat_symbol(canvas, flat_x, flat_y, colors);
                }
            }
        }
    }

    fn draw_sharp_symbol(&self, canvas: &mut Canvas, x: f32, y: f32, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.line_width(1.0);

        let s = STAFF_SPACE * 0.6;

        // Vertical lines
        canvas.begin_path();
        canvas.move_to(Point::new(x - s * 0.25, y - s));
        canvas.line_to(Point::new(x - s * 0.25, y + s));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x + s * 0.25, y - s));
        canvas.line_to(Point::new(x + s * 0.25, y + s));
        canvas.stroke();

        // Horizontal lines
        canvas.line_width(2.0);
        canvas.begin_path();
        canvas.move_to(Point::new(x - s * 0.5, y - s * 0.3));
        canvas.line_to(Point::new(x + s * 0.5, y - s * 0.4));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x - s * 0.5, y + s * 0.3));
        canvas.line_to(Point::new(x + s * 0.5, y + s * 0.2));
        canvas.stroke();
    }

    fn draw_flat_symbol(&self, canvas: &mut Canvas, x: f32, y: f32, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.line_width(1.5);

        let s = STAFF_SPACE * 0.6;

        // Vertical line
        canvas.begin_path();
        canvas.move_to(Point::new(x, y - s * 1.2));
        canvas.line_to(Point::new(x, y + s * 0.3));
        canvas.stroke();

        // Curved part
        canvas.begin_path();
        canvas.move_to(Point::new(x, y));
        canvas.line_to(Point::new(x + s * 0.5, y - s * 0.3));
        canvas.line_to(Point::new(x, y + s * 0.3));
        canvas.stroke();
    }
}

impl Element for MeasureElement {
    fn limits(&self, _ctx: &BasicContext) -> ViewLimits {
        ViewLimits::fixed(self.width, STAFF_HEIGHT)
    }

    fn draw(&self, _ctx: &Context) {
        // Actual drawing happens via draw_measure
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_element_creation() {
        let measure = MeasureElement::new(200.0, Clef::treble());
        assert_eq!(measure.width, 200.0);
    }
}
