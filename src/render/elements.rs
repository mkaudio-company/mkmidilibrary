//! Score element - the top-level rendering element

use std::any::Any;

use mkgraphic::prelude::*;
use mkgraphic::support::canvas::Canvas;
use num::ToPrimitive;

use super::clef::ClefElement;
use super::config::RenderConfig;
use super::staff::StaffElement;
use super::{STAFF_HEIGHT, STAFF_SPACE};
use crate::notation::Clef;
use crate::stream::Score;

/// A graphical element representing an entire score
pub struct ScoreElement {
    /// The score data
    parts_data: Vec<PartData>,
    /// Rendering configuration (kept for future use)
    #[allow(dead_code)]
    config: RenderConfig,
    /// Total width
    width: f32,
    /// Total height
    height: f32,
}

/// Data for rendering a single part
struct PartData {
    /// Part name (kept for future use - part labels)
    #[allow(dead_code)]
    name: String,
    /// Clef for this part
    clef: Clef,
    /// Measure count
    measure_count: usize,
    /// Notes per measure (simplified representation)
    measures: Vec<MeasureData>,
}

/// Data for a single measure
struct MeasureData {
    /// Note offsets and MIDI values
    notes: Vec<(f64, u8)>,
}

impl ScoreElement {
    /// Create a new score element from a Score
    pub fn new(score: &Score, config: RenderConfig) -> Self {
        let mut parts_data = Vec::new();

        for part in score.parts() {
            // Default to treble clef (Measure doesn't store clef directly)
            let clef = Clef::treble();

            let measures: Vec<MeasureData> = part
                .measures()
                .iter()
                .map(|measure| {
                    let notes: Vec<(f64, u8)> = measure
                        .elements()
                        .iter()
                        .filter_map(|(offset, elem)| {
                            use crate::stream::MusicElement;
                            match elem {
                                MusicElement::Note(n) => {
                                    Some((offset.to_f64().unwrap_or(0.0), n.midi()))
                                }
                                _ => None,
                            }
                        })
                        .collect();
                    MeasureData { notes }
                })
                .collect();

            parts_data.push(PartData {
                name: part.name().unwrap_or("Part").to_string(),
                clef,
                measure_count: part.measures().len(),
                measures,
            });
        }

        // Calculate dimensions
        let num_parts = parts_data.len().max(1);
        let num_measures = parts_data
            .first()
            .map(|p| p.measure_count)
            .unwrap_or(0)
            .max(1);

        let width = config.margin_left
            + config.clef_width
            + config.key_sig_width
            + config.time_sig_width
            + (num_measures as f32 * config.measure_width)
            + config.margin_right;

        let staff_with_spacing = config.staff.height + config.staff_spacing;
        let height = config.margin_top
            + (num_parts as f32 * staff_with_spacing)
            + config.margin_bottom;

        Self {
            parts_data,
            config,
            width,
            height,
        }
    }

    /// Get the width
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Draw the score to a canvas
    pub fn draw_to_canvas(&self, canvas: &mut Canvas, config: &RenderConfig) {
        // Clear background
        let bg = &config.colors.background;
        canvas.clear(Color::new(bg.0, bg.1, bg.2, bg.3));

        let staff_with_spacing = config.staff.height + config.staff_spacing;

        // Draw each part
        for (part_idx, part) in self.parts_data.iter().enumerate() {
            let staff_y =
                config.margin_top + STAFF_HEIGHT / 2.0 + (part_idx as f32 * staff_with_spacing);

            // Draw staff lines
            let staff_width = self.width - config.margin_left - config.margin_right;
            let mut staff = StaffElement::new(staff_width);
            staff.set_position(config.margin_left, staff_y);
            staff.draw_to_canvas(canvas, &config.colors.staff_lines);

            // Draw clef
            let mut clef_element = ClefElement::new(part.clef.clone());
            clef_element.set_position(config.margin_left + 5.0, staff_y);
            clef_element.draw_to_canvas(canvas, config);

            // Draw measures
            let measure_start_x =
                config.margin_left + config.clef_width + config.key_sig_width + config.time_sig_width;

            for (measure_idx, measure_data) in part.measures.iter().enumerate() {
                let measure_x = measure_start_x + (measure_idx as f32 * config.measure_width);
                let is_last = measure_idx == part.measure_count - 1;

                // Draw notes in this measure
                for (offset, midi) in &measure_data.notes {
                    let note_x = measure_x + (*offset as f32 * config.measure_width * 0.8);
                    let position = super::midi_to_staff_position(*midi, &part.clef);
                    let note_y = staff_y + position.to_y(STAFF_SPACE);

                    // Draw simple note head
                    self.draw_simple_note(canvas, note_x, note_y, config);

                    // Draw ledger lines if needed
                    if position.position > 4 || position.position < -4 {
                        staff.draw_ledger_lines(
                            canvas,
                            position.position,
                            note_x,
                            config.note.head_width,
                            &config.colors.staff_lines,
                        );
                    }
                }

                // Draw bar line
                let bar_x = measure_x + config.measure_width;
                let top_y = staff_y - STAFF_HEIGHT / 2.0;
                let bottom_y = staff_y + STAFF_HEIGHT / 2.0;

                if is_last {
                    super::staff::draw_double_bar_line(
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
                    super::staff::draw_bar_line(
                        canvas,
                        bar_x,
                        top_y,
                        bottom_y,
                        1.0,
                        &config.colors.bar_lines,
                    );
                }

                // Draw measure number
                if config.show_bar_numbers && measure_idx == 0 {
                    // Would draw measure number here with text rendering
                }
            }

            // Draw initial bar line
            let initial_bar_x = measure_start_x;
            let top_y = staff_y - STAFF_HEIGHT / 2.0;
            let bottom_y = staff_y + STAFF_HEIGHT / 2.0;
            super::staff::draw_bar_line(
                canvas,
                initial_bar_x,
                top_y,
                bottom_y,
                1.0,
                &config.colors.bar_lines,
            );
        }
    }

    /// Draw a simple note (filled oval)
    fn draw_simple_note(&self, canvas: &mut Canvas, x: f32, y: f32, config: &RenderConfig) {
        let colors = &config.colors.notes;
        let color = Color::new(colors.0, colors.1, colors.2, colors.3);

        canvas.fill_style(color);
        canvas.begin_path();
        canvas.add_circle(mkgraphic::support::circle::Circle::new(
            Point::new(x + config.note.head_width / 2.0, y),
            config.note.head_height / 2.0 * 0.9,
        ));
        canvas.fill();

        // Draw stem
        canvas.stroke_style(color);
        canvas.line_width(config.note.stem_width);
        canvas.begin_path();
        let stem_x = x + config.note.head_width;
        canvas.move_to(Point::new(stem_x, y));
        canvas.line_to(Point::new(stem_x, y - config.note.stem_height));
        canvas.stroke();
    }
}

impl Element for ScoreElement {
    fn limits(&self, _ctx: &BasicContext) -> ViewLimits {
        ViewLimits::fixed(self.width, self.height)
    }

    fn draw(&self, _ctx: &Context) {
        // Note: In mkgraphic, drawing typically happens through the Context
        // For now, actual drawing is done via draw_to_canvas
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Helper to create a canvas and render a score
pub fn render_score_to_image(score: &Score, config: &RenderConfig) -> Option<Vec<u8>> {
    let element = ScoreElement::new(score, config.clone());
    let width = (element.width() * config.scale) as u32;
    let height = (element.height() * config.scale) as u32;

    let mut canvas = Canvas::new(width, height)?;

    if config.scale != 1.0 {
        canvas.scale(config.scale, config.scale);
    }

    element.draw_to_canvas(&mut canvas, config);

    // Return PNG data
    Some(canvas.pixmap().encode_png().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_element_creation() {
        let score = Score::new();
        let config = RenderConfig::default();
        let element = ScoreElement::new(&score, config);

        assert!(element.width() > 0.0);
        assert!(element.height() > 0.0);
    }
}
