//! Staff rendering element

use std::any::Any;

use mkgraphic::prelude::*;
use mkgraphic::support::canvas::Canvas;

use super::config::StaffConfig;

/// A graphical element representing a musical staff
pub struct StaffElement {
    /// Staff configuration
    config: StaffConfig,
    /// Width of the staff
    width: f32,
    /// X position
    x: f32,
    /// Y position (center of staff)
    y: f32,
}

impl StaffElement {
    /// Create a new staff element
    pub fn new(width: f32) -> Self {
        Self {
            config: StaffConfig::default(),
            width,
            x: 0.0,
            y: 0.0,
        }
    }

    /// Create with custom configuration
    pub fn with_config(width: f32, config: StaffConfig) -> Self {
        Self {
            config,
            width,
            x: 0.0,
            y: 0.0,
        }
    }

    /// Set the position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// Get the Y position of a staff line (0 = top line)
    pub fn line_y(&self, line: u8) -> f32 {
        self.y - (self.config.height / 2.0) + (line as f32 * self.config.space)
    }

    /// Get the Y position for a staff position (-4 to 4 for standard 5-line staff)
    pub fn position_y(&self, position: i8) -> f32 {
        self.y - (position as f32 * self.config.space / 2.0)
    }

    /// Draw the staff to a canvas
    pub fn draw_to_canvas(&self, canvas: &mut Canvas, colors: &(f32, f32, f32, f32)) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.line_width(self.config.line_width);

        let top_y = self.y - self.config.height / 2.0;

        for i in 0..self.config.lines {
            let y = top_y + (i as f32 * self.config.space);
            canvas.begin_path();
            canvas.move_to(Point::new(self.x, y));
            canvas.line_to(Point::new(self.x + self.width, y));
            canvas.stroke();
        }
    }

    /// Draw ledger lines for a position outside the staff
    pub fn draw_ledger_lines(
        &self,
        canvas: &mut Canvas,
        position: i8,
        x: f32,
        note_width: f32,
        colors: &(f32, f32, f32, f32),
    ) {
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);
        canvas.stroke_style(color);
        canvas.line_width(self.config.line_width);

        let extension = 4.0;
        let ledger_start = x - extension;
        let ledger_end = x + note_width + extension;

        // Ledger lines above staff (position > 4)
        if position > 4 {
            let num_ledgers = (position - 4 + 1) / 2;
            for i in 1..=num_ledgers {
                let ledger_pos = 4 + (i * 2);
                let y = self.position_y(ledger_pos as i8);
                canvas.begin_path();
                canvas.move_to(Point::new(ledger_start, y));
                canvas.line_to(Point::new(ledger_end, y));
                canvas.stroke();
            }
        }

        // Ledger lines below staff (position < -4)
        if position < -4 {
            let num_ledgers = (-4 - position + 1) / 2;
            for i in 1..=num_ledgers {
                let ledger_pos = -4 - (i * 2);
                let y = self.position_y(ledger_pos as i8);
                canvas.begin_path();
                canvas.move_to(Point::new(ledger_start, y));
                canvas.line_to(Point::new(ledger_end, y));
                canvas.stroke();
            }
        }
    }
}

impl Element for StaffElement {
    fn limits(&self, _ctx: &BasicContext) -> ViewLimits {
        ViewLimits::fixed(self.width, self.config.height)
    }

    fn draw(&self, _ctx: &Context) {
        // Note: In a full implementation, we'd use ctx to access the canvas
        // For now, this is a placeholder - actual drawing happens via draw_to_canvas
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Draw a bar line
pub fn draw_bar_line(
    canvas: &mut Canvas,
    x: f32,
    top_y: f32,
    bottom_y: f32,
    width: f32,
    colors: &(f32, f32, f32, f32),
) {
    let color = Color::new(colors.0, colors.1, colors.2, colors.3);
    canvas.stroke_style(color);
    canvas.line_width(width);

    canvas.begin_path();
    canvas.move_to(Point::new(x, top_y));
    canvas.line_to(Point::new(x, bottom_y));
    canvas.stroke();
}

/// Draw a double bar line
pub fn draw_double_bar_line(
    canvas: &mut Canvas,
    x: f32,
    top_y: f32,
    bottom_y: f32,
    thin_width: f32,
    thick_width: f32,
    spacing: f32,
    colors: &(f32, f32, f32, f32),
) {
    let color = Color::new(colors.0, colors.1, colors.2, colors.3);

    // Thin line
    canvas.stroke_style(color);
    canvas.line_width(thin_width);
    canvas.begin_path();
    canvas.move_to(Point::new(x, top_y));
    canvas.line_to(Point::new(x, bottom_y));
    canvas.stroke();

    // Thick line
    canvas.line_width(thick_width);
    canvas.begin_path();
    canvas.move_to(Point::new(x + spacing, top_y));
    canvas.line_to(Point::new(x + spacing, bottom_y));
    canvas.stroke();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::STAFF_HEIGHT;

    #[test]
    fn test_staff_element_creation() {
        let staff = StaffElement::new(400.0);
        assert_eq!(staff.width, 400.0);
    }

    #[test]
    fn test_staff_line_positions() {
        let mut staff = StaffElement::new(400.0);
        staff.set_position(0.0, 100.0);

        // Top line should be at y - height/2
        let top_y = staff.line_y(0);
        assert!((top_y - (100.0 - STAFF_HEIGHT / 2.0)).abs() < 0.01);
    }
}
