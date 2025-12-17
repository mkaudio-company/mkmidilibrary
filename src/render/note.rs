//! Note rendering element

use std::any::Any;

use mkgraphic::prelude::*;
use mkgraphic::support::canvas::Canvas;
use mkgraphic::support::circle::Circle;

use super::config::{NoteConfig, RenderConfig};
use super::{StaffPosition, STAFF_SPACE};
use crate::core::{DurationType, Note};

/// A graphical element representing a musical note
pub struct NoteElement {
    /// The note to render
    note: Note,
    /// Staff position
    position: StaffPosition,
    /// X coordinate
    x: f32,
    /// Y coordinate (staff center)
    staff_y: f32,
    /// Note configuration
    config: NoteConfig,
    /// Whether the note is selected
    selected: bool,
}

impl NoteElement {
    /// Create a new note element
    pub fn new(note: Note, position: StaffPosition) -> Self {
        Self {
            note,
            position,
            x: 0.0,
            staff_y: 0.0,
            config: NoteConfig::default(),
            selected: false,
        }
    }

    /// Set the position
    pub fn set_position(&mut self, x: f32, staff_y: f32) {
        self.x = x;
        self.staff_y = staff_y;
    }

    /// Set selected state
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    /// Get the note Y position
    fn note_y(&self) -> f32 {
        self.staff_y + self.position.to_y(STAFF_SPACE)
    }

    /// Draw the note to a canvas
    pub fn draw_to_canvas(&self, canvas: &mut Canvas, config: &RenderConfig) {
        let y = self.note_y();
        let colors = if self.selected {
            &config.colors.selected
        } else {
            &config.colors.notes
        };

        // Draw accidental if present
        if self.position.accidental != 0 {
            self.draw_accidental(canvas, y, &config.colors.accidentals);
        }

        // Draw notehead
        self.draw_notehead(canvas, y, colors);

        // Draw stem if needed
        if self.needs_stem() {
            self.draw_stem(canvas, y, colors);
        }

        // Draw flags or beams for eighth notes and shorter
        if self.needs_flags() {
            self.draw_flags(canvas, y, colors);
        }

        // Draw dots
        let dots = self.note.duration().dots();
        if dots > 0 {
            self.draw_dots(canvas, y, dots, colors);
        }
    }

    /// Check if the note needs a stem
    fn needs_stem(&self) -> bool {
        match self.note.duration().type_() {
            Some(DurationType::Whole) | Some(DurationType::Breve) | None => false,
            _ => true,
        }
    }

    /// Check if the note needs flags
    fn needs_flags(&self) -> bool {
        matches!(
            self.note.duration().type_(),
            Some(DurationType::Eighth)
                | Some(DurationType::N16th)
                | Some(DurationType::N32nd)
                | Some(DurationType::N64th)
                | Some(DurationType::N128th)
        )
    }

    /// Get the number of flags needed
    fn flag_count(&self) -> u8 {
        match self.note.duration().type_() {
            Some(DurationType::Eighth) => 1,
            Some(DurationType::N16th) => 2,
            Some(DurationType::N32nd) => 3,
            Some(DurationType::N64th) => 4,
            Some(DurationType::N128th) => 5,
            _ => 0,
        }
    }

    /// Draw the notehead
    fn draw_notehead(&self, canvas: &mut Canvas, y: f32, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        let is_filled = self.is_filled_notehead();

        let half_width = self.config.head_width / 2.0;
        let half_height = self.config.head_height / 2.0;

        // Draw elliptical notehead
        canvas.begin_path();

        // Approximate ellipse with bezier curves
        let cx = self.x + half_width;
        let cy = y;

        // Draw filled or hollow notehead
        if is_filled {
            canvas.fill_style(color);
            // Simple circle approximation for filled noteheads
            canvas.add_circle(Circle::new(Point::new(cx, cy), half_height * 0.9));
            canvas.fill();
        } else {
            // Hollow notehead (half note, whole note)
            canvas.stroke_style(color);
            canvas.line_width(1.5);
            canvas.add_circle(Circle::new(Point::new(cx, cy), half_height * 0.9));
            canvas.stroke();
        }
    }

    /// Check if the notehead should be filled
    fn is_filled_notehead(&self) -> bool {
        match self.note.duration().type_() {
            Some(DurationType::Whole)
            | Some(DurationType::Breve)
            | Some(DurationType::Half) => false,
            _ => true,
        }
    }

    /// Draw the stem
    fn draw_stem(&self, canvas: &mut Canvas, y: f32, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.line_width(self.config.stem_width);

        // Stem direction: up if below middle line, down if above
        let stem_up = self.position.position <= 0;

        let stem_x = if stem_up {
            self.x + self.config.head_width - self.config.stem_width / 2.0
        } else {
            self.x + self.config.stem_width / 2.0
        };

        let stem_y1 = y;
        let stem_y2 = if stem_up {
            y - self.config.stem_height
        } else {
            y + self.config.stem_height
        };

        canvas.begin_path();
        canvas.move_to(Point::new(stem_x, stem_y1));
        canvas.line_to(Point::new(stem_x, stem_y2));
        canvas.stroke();
    }

    /// Draw flags for eighth notes and shorter
    fn draw_flags(&self, canvas: &mut Canvas, y: f32, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        let num_flags = self.flag_count();
        if num_flags == 0 {
            return;
        }

        canvas.fill_style(color);
        canvas.stroke_style(color);
        canvas.line_width(1.5);

        let stem_up = self.position.position <= 0;
        let stem_x = if stem_up {
            self.x + self.config.head_width - self.config.stem_width / 2.0
        } else {
            self.x + self.config.stem_width / 2.0
        };

        let flag_spacing = STAFF_SPACE * 0.8;

        for i in 0..num_flags {
            let flag_y_start = if stem_up {
                y - self.config.stem_height + (i as f32 * flag_spacing)
            } else {
                y + self.config.stem_height - (i as f32 * flag_spacing)
            };

            // Draw simple flag (curved line)
            canvas.begin_path();
            canvas.move_to(Point::new(stem_x, flag_y_start));

            let flag_direction = if stem_up { 1.0 } else { -1.0 };
            let flag_end_x = stem_x + self.config.flag_width;
            let flag_end_y = flag_y_start + flag_direction * STAFF_SPACE;

            canvas.line_to(Point::new(flag_end_x, flag_end_y));
            canvas.stroke();
        }
    }

    /// Draw dots for dotted notes
    fn draw_dots(&self, canvas: &mut Canvas, y: f32, dots: u8, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.fill_style(color);

        let dot_x_start = self.x + self.config.head_width + self.config.dot_spacing;

        // Adjust y if on a line (move dot to space above)
        let dot_y = if self.position.position % 2 == 0 {
            y - STAFF_SPACE / 4.0
        } else {
            y
        };

        for i in 0..dots {
            let dot_x = dot_x_start + (i as f32 * self.config.dot_spacing * 2.0);
            canvas.begin_path();
            canvas.add_circle(Circle::new(Point::new(dot_x, dot_y), self.config.dot_radius));
            canvas.fill();
        }
    }

    /// Draw accidental
    fn draw_accidental(&self, canvas: &mut Canvas, y: f32, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.fill_style(color);
        canvas.line_width(1.0);

        let acc_x = self.x - self.config.accidental_spacing;

        match self.position.accidental {
            1 => self.draw_sharp(canvas, acc_x, y),
            -1 => self.draw_flat(canvas, acc_x, y),
            2 => self.draw_double_sharp(canvas, acc_x, y),
            -2 => self.draw_double_flat(canvas, acc_x, y),
            0 => self.draw_natural(canvas, acc_x, y),
            _ => {}
        }
    }

    fn draw_sharp(&self, canvas: &mut Canvas, x: f32, y: f32) {
        let h = STAFF_SPACE * 1.5;
        let w = STAFF_SPACE * 0.6;

        // Vertical lines
        canvas.begin_path();
        canvas.move_to(Point::new(x - w / 4.0, y - h / 2.0));
        canvas.line_to(Point::new(x - w / 4.0, y + h / 2.0));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x + w / 4.0, y - h / 2.0));
        canvas.line_to(Point::new(x + w / 4.0, y + h / 2.0));
        canvas.stroke();

        // Horizontal lines (slightly slanted)
        canvas.line_width(2.0);
        canvas.begin_path();
        canvas.move_to(Point::new(x - w / 2.0, y - STAFF_SPACE / 4.0 + 1.0));
        canvas.line_to(Point::new(x + w / 2.0, y - STAFF_SPACE / 4.0 - 1.0));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x - w / 2.0, y + STAFF_SPACE / 4.0 + 1.0));
        canvas.line_to(Point::new(x + w / 2.0, y + STAFF_SPACE / 4.0 - 1.0));
        canvas.stroke();
    }

    fn draw_flat(&self, canvas: &mut Canvas, x: f32, y: f32) {
        let h = STAFF_SPACE * 1.5;

        // Vertical line
        canvas.begin_path();
        canvas.move_to(Point::new(x, y - h / 2.0));
        canvas.line_to(Point::new(x, y + STAFF_SPACE / 3.0));
        canvas.stroke();

        // Curved part (simplified)
        canvas.begin_path();
        canvas.move_to(Point::new(x, y));
        canvas.line_to(Point::new(x + STAFF_SPACE * 0.4, y - STAFF_SPACE / 4.0));
        canvas.line_to(Point::new(x, y + STAFF_SPACE / 3.0));
        canvas.stroke();
    }

    fn draw_natural(&self, canvas: &mut Canvas, x: f32, y: f32) {
        let h = STAFF_SPACE * 1.2;
        let w = STAFF_SPACE * 0.4;

        // Vertical lines
        canvas.begin_path();
        canvas.move_to(Point::new(x - w / 2.0, y - h / 2.0));
        canvas.line_to(Point::new(x - w / 2.0, y + STAFF_SPACE / 4.0));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x + w / 2.0, y - STAFF_SPACE / 4.0));
        canvas.line_to(Point::new(x + w / 2.0, y + h / 2.0));
        canvas.stroke();

        // Horizontal lines
        canvas.line_width(2.0);
        canvas.begin_path();
        canvas.move_to(Point::new(x - w / 2.0, y - STAFF_SPACE / 4.0));
        canvas.line_to(Point::new(x + w / 2.0, y - STAFF_SPACE / 4.0));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x - w / 2.0, y + STAFF_SPACE / 4.0));
        canvas.line_to(Point::new(x + w / 2.0, y + STAFF_SPACE / 4.0));
        canvas.stroke();
    }

    fn draw_double_sharp(&self, canvas: &mut Canvas, x: f32, y: f32) {
        let size = STAFF_SPACE * 0.4;

        // X shape
        canvas.line_width(2.0);
        canvas.begin_path();
        canvas.move_to(Point::new(x - size, y - size));
        canvas.line_to(Point::new(x + size, y + size));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(x + size, y - size));
        canvas.line_to(Point::new(x - size, y + size));
        canvas.stroke();
    }

    fn draw_double_flat(&self, canvas: &mut Canvas, x: f32, y: f32) {
        // Two flats side by side
        self.draw_flat(canvas, x - STAFF_SPACE * 0.3, y);
        self.draw_flat(canvas, x + STAFF_SPACE * 0.3, y);
    }
}

impl Element for NoteElement {
    fn limits(&self, _ctx: &BasicContext) -> ViewLimits {
        ViewLimits::fixed(
            self.config.head_width + self.config.accidental_spacing,
            self.config.stem_height + self.config.head_height,
        )
    }

    fn draw(&self, _ctx: &Context) {
        // Actual drawing happens via draw_to_canvas
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
    use crate::core::{Pitch, Step};

    #[test]
    fn test_note_element_creation() {
        let pitch = Pitch::from_parts(Step::C, Some(4), None);
        let note = Note::quarter(pitch);
        let position = StaffPosition::new(0, 0);
        let element = NoteElement::new(note, position);

        assert!(!element.selected);
    }
}
