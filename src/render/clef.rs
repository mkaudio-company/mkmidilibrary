//! Clef rendering element

use std::any::Any;

use mkgraphic::prelude::*;
use mkgraphic::support::canvas::Canvas;

use super::{STAFF_SPACE, STAFF_HEIGHT};
use super::config::RenderConfig;
use crate::notation::{Clef, ClefSign};

/// A graphical element representing a clef
pub struct ClefElement {
    /// The clef to render
    clef: Clef,
    /// X position
    x: f32,
    /// Y position (staff center)
    staff_y: f32,
    /// Scale factor
    scale: f32,
}

impl ClefElement {
    /// Create a new clef element
    pub fn new(clef: Clef) -> Self {
        Self {
            clef,
            x: 0.0,
            staff_y: 0.0,
            scale: 1.0,
        }
    }

    /// Set the position
    pub fn set_position(&mut self, x: f32, staff_y: f32) {
        self.x = x;
        self.staff_y = staff_y;
    }

    /// Set the scale
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Get the width of this clef
    pub fn width(&self) -> f32 {
        match self.clef.sign() {
            ClefSign::G => STAFF_SPACE * 3.0 * self.scale,
            ClefSign::F => STAFF_SPACE * 2.5 * self.scale,
            ClefSign::C => STAFF_SPACE * 2.5 * self.scale,
            ClefSign::Percussion => STAFF_SPACE * 2.0 * self.scale,
            ClefSign::Tab => STAFF_SPACE * 2.5 * self.scale,
        }
    }

    /// Draw the clef to a canvas
    pub fn draw_to_canvas(&self, canvas: &mut Canvas, config: &RenderConfig) {
        let colors = &config.colors.clefs;
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);

        canvas.stroke_style(color);
        canvas.fill_style(color);

        match self.clef.sign() {
            ClefSign::G => self.draw_treble_clef(canvas),
            ClefSign::F => self.draw_bass_clef(canvas),
            ClefSign::C => self.draw_c_clef(canvas),
            ClefSign::Percussion => self.draw_percussion_clef(canvas),
            ClefSign::Tab => self.draw_tab_clef(canvas),
        }

        // Draw octave marker if present
        let octave_change = self.clef.octave_change();
        if octave_change != 0 {
            self.draw_octave_marker(canvas, octave_change);
        }
    }

    /// Draw treble (G) clef
    fn draw_treble_clef(&self, canvas: &mut Canvas) {
        let s = STAFF_SPACE * self.scale;
        let cx = self.x + s * 1.5;

        // The G clef curls around the G line (line 2)
        // Position relative to staff center
        let _g_line_y = self.staff_y + s * 0.5; // Second line from bottom

        canvas.line_width(2.0 * self.scale);

        // Simplified treble clef shape
        // Main spiral
        canvas.begin_path();

        // Start from bottom, curl up
        let bottom_y = self.staff_y + s * 2.5;
        let top_y = self.staff_y - s * 3.0;

        canvas.move_to(Point::new(cx, bottom_y));

        // Main body curve going up
        canvas.line_to(Point::new(cx - s * 0.8, self.staff_y + s * 1.5));
        canvas.line_to(Point::new(cx - s * 0.5, self.staff_y + s * 0.5));
        canvas.line_to(Point::new(cx + s * 0.3, self.staff_y));
        canvas.line_to(Point::new(cx + s * 0.5, self.staff_y - s * 0.5));
        canvas.line_to(Point::new(cx + s * 0.3, self.staff_y - s * 1.0));
        canvas.line_to(Point::new(cx - s * 0.2, self.staff_y - s * 1.2));
        canvas.line_to(Point::new(cx - s * 0.5, self.staff_y - s * 1.0));

        // Cross through and go up
        canvas.line_to(Point::new(cx, self.staff_y - s * 0.3));
        canvas.line_to(Point::new(cx + s * 0.2, top_y + s));
        canvas.line_to(Point::new(cx, top_y));

        canvas.stroke();

        // Bottom curl
        canvas.begin_path();
        canvas.move_to(Point::new(cx, bottom_y));
        canvas.line_to(Point::new(cx + s * 0.3, bottom_y - s * 0.3));
        canvas.line_to(Point::new(cx + s * 0.2, bottom_y - s * 0.6));
        canvas.line_to(Point::new(cx - s * 0.1, bottom_y - s * 0.4));
        canvas.stroke();
    }

    /// Draw bass (F) clef
    fn draw_bass_clef(&self, canvas: &mut Canvas) {
        let s = STAFF_SPACE * self.scale;
        let cx = self.x + s;

        // F clef centers on the F line (line 4)
        let f_line_y = self.staff_y - s * 0.5;

        canvas.line_width(2.0 * self.scale);

        // Main body (curved shape)
        canvas.begin_path();
        canvas.move_to(Point::new(cx, f_line_y - s * 0.3));
        canvas.line_to(Point::new(cx + s * 0.8, f_line_y - s * 0.5));
        canvas.line_to(Point::new(cx + s, f_line_y));
        canvas.line_to(Point::new(cx + s * 0.8, f_line_y + s * 0.5));
        canvas.line_to(Point::new(cx + s * 0.3, f_line_y + s));
        canvas.line_to(Point::new(cx - s * 0.2, f_line_y + s * 1.2));
        canvas.line_to(Point::new(cx - s * 0.4, f_line_y + s));
        canvas.stroke();

        // Dot at the start
        canvas.begin_path();
        canvas.add_circle(mkgraphic::support::circle::Circle::new(
            Point::new(cx - s * 0.2, f_line_y),
            s * 0.25,
        ));
        canvas.fill();

        // Two dots to the right
        let dot_x = cx + s * 1.3;
        canvas.begin_path();
        canvas.add_circle(mkgraphic::support::circle::Circle::new(
            Point::new(dot_x, f_line_y - s * 0.5),
            s * 0.15,
        ));
        canvas.fill();

        canvas.begin_path();
        canvas.add_circle(mkgraphic::support::circle::Circle::new(
            Point::new(dot_x, f_line_y + s * 0.5),
            s * 0.15,
        ));
        canvas.fill();
    }

    /// Draw C clef (alto/tenor)
    fn draw_c_clef(&self, canvas: &mut Canvas) {
        let s = STAFF_SPACE * self.scale;
        let cx = self.x + s * 0.5;

        // C clef line position based on clef.line()
        let line_offset = (3 - self.clef.line() as i8) as f32;
        let c_line_y = self.staff_y + line_offset * s * 0.5;

        canvas.line_width(2.0 * self.scale);

        // Left vertical bars
        canvas.begin_path();
        canvas.move_to(Point::new(cx, c_line_y - s * 2.0));
        canvas.line_to(Point::new(cx, c_line_y + s * 2.0));
        canvas.stroke();

        canvas.line_width(4.0 * self.scale);
        canvas.begin_path();
        canvas.move_to(Point::new(cx + s * 0.3, c_line_y - s * 2.0));
        canvas.line_to(Point::new(cx + s * 0.3, c_line_y + s * 2.0));
        canvas.stroke();

        // Right curved sections
        canvas.line_width(2.0 * self.scale);

        // Top curve
        canvas.begin_path();
        canvas.move_to(Point::new(cx + s * 0.5, c_line_y - s * 2.0));
        canvas.line_to(Point::new(cx + s * 1.5, c_line_y - s * 1.5));
        canvas.line_to(Point::new(cx + s * 1.8, c_line_y - s * 0.5));
        canvas.line_to(Point::new(cx + s * 1.5, c_line_y));
        canvas.stroke();

        // Bottom curve
        canvas.begin_path();
        canvas.move_to(Point::new(cx + s * 0.5, c_line_y + s * 2.0));
        canvas.line_to(Point::new(cx + s * 1.5, c_line_y + s * 1.5));
        canvas.line_to(Point::new(cx + s * 1.8, c_line_y + s * 0.5));
        canvas.line_to(Point::new(cx + s * 1.5, c_line_y));
        canvas.stroke();
    }

    /// Draw percussion clef
    fn draw_percussion_clef(&self, canvas: &mut Canvas) {
        let s = STAFF_SPACE * self.scale;
        let cx = self.x + s;

        canvas.line_width(4.0 * self.scale);

        // Two thick vertical lines
        canvas.begin_path();
        canvas.move_to(Point::new(cx - s * 0.3, self.staff_y - s));
        canvas.line_to(Point::new(cx - s * 0.3, self.staff_y + s));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(cx + s * 0.3, self.staff_y - s));
        canvas.line_to(Point::new(cx + s * 0.3, self.staff_y + s));
        canvas.stroke();
    }

    /// Draw tab clef
    fn draw_tab_clef(&self, canvas: &mut Canvas) {
        // TAB clef is typically just the letters "TAB" stacked vertically
        // For now, draw a simplified version
        let s = STAFF_SPACE * self.scale;
        let cx = self.x + s;

        canvas.line_width(2.0 * self.scale);

        // T
        let t_y = self.staff_y - s * 1.5;
        canvas.begin_path();
        canvas.move_to(Point::new(cx - s * 0.4, t_y));
        canvas.line_to(Point::new(cx + s * 0.4, t_y));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(cx, t_y));
        canvas.line_to(Point::new(cx, t_y + s * 0.8));
        canvas.stroke();

        // A
        let a_y = self.staff_y;
        canvas.begin_path();
        canvas.move_to(Point::new(cx - s * 0.3, a_y + s * 0.4));
        canvas.line_to(Point::new(cx, a_y - s * 0.4));
        canvas.line_to(Point::new(cx + s * 0.3, a_y + s * 0.4));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(cx - s * 0.15, a_y));
        canvas.line_to(Point::new(cx + s * 0.15, a_y));
        canvas.stroke();

        // B
        let b_y = self.staff_y + s * 1.5;
        canvas.begin_path();
        canvas.move_to(Point::new(cx - s * 0.2, b_y - s * 0.4));
        canvas.line_to(Point::new(cx - s * 0.2, b_y + s * 0.4));
        canvas.stroke();

        canvas.begin_path();
        canvas.move_to(Point::new(cx - s * 0.2, b_y - s * 0.4));
        canvas.line_to(Point::new(cx + s * 0.2, b_y - s * 0.2));
        canvas.line_to(Point::new(cx - s * 0.2, b_y));
        canvas.line_to(Point::new(cx + s * 0.2, b_y + s * 0.2));
        canvas.line_to(Point::new(cx - s * 0.2, b_y + s * 0.4));
        canvas.stroke();
    }

    /// Draw octave marker (8va, 8vb, etc.)
    fn draw_octave_marker(&self, canvas: &mut Canvas, octave_change: i8) {
        let s = STAFF_SPACE * self.scale;
        let marker_y = if octave_change > 0 {
            self.staff_y - STAFF_HEIGHT / 2.0 - s * 1.5
        } else {
            self.staff_y + STAFF_HEIGHT / 2.0 + s * 0.5
        };

        canvas.line_width(1.0);

        // Draw "8" (simplified)
        let eight_x = self.x + self.width() * 0.5;
        canvas.begin_path();
        canvas.add_circle(mkgraphic::support::circle::Circle::new(
            Point::new(eight_x, marker_y - s * 0.15),
            s * 0.2,
        ));
        canvas.stroke();

        canvas.begin_path();
        canvas.add_circle(mkgraphic::support::circle::Circle::new(
            Point::new(eight_x, marker_y + s * 0.15),
            s * 0.25,
        ));
        canvas.stroke();
    }
}

impl Element for ClefElement {
    fn limits(&self, _ctx: &BasicContext) -> ViewLimits {
        ViewLimits::fixed(self.width(), STAFF_HEIGHT)
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

    #[test]
    fn test_clef_element_creation() {
        let clef = Clef::treble();
        let element = ClefElement::new(clef);
        assert!(element.width() > 0.0);
    }

    #[test]
    fn test_clef_widths() {
        let treble = ClefElement::new(Clef::treble());
        let bass = ClefElement::new(Clef::bass());
        let alto = ClefElement::new(Clef::alto());

        assert!(treble.width() > 0.0);
        assert!(bass.width() > 0.0);
        assert!(alto.width() > 0.0);
    }
}
