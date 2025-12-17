//! Rendering configuration

use super::{STAFF_SPACE, STAFF_HEIGHT};

/// Staff rendering configuration
#[derive(Debug, Clone)]
pub struct StaffConfig {
    /// Number of staff lines
    pub lines: u8,
    /// Staff line spacing
    pub space: f32,
    /// Staff height
    pub height: f32,
    /// Line thickness
    pub line_width: f32,
}

impl Default for StaffConfig {
    fn default() -> Self {
        Self {
            lines: 5,
            space: STAFF_SPACE,
            height: STAFF_HEIGHT,
            line_width: 1.0,
        }
    }
}

/// Note rendering configuration
#[derive(Debug, Clone)]
pub struct NoteConfig {
    /// Notehead width
    pub head_width: f32,
    /// Notehead height
    pub head_height: f32,
    /// Stem width
    pub stem_width: f32,
    /// Stem height
    pub stem_height: f32,
    /// Flag width
    pub flag_width: f32,
    /// Beam thickness
    pub beam_thickness: f32,
    /// Ledger line width
    pub ledger_width: f32,
    /// Ledger line extension beyond notehead
    pub ledger_extension: f32,
    /// Accidental spacing from notehead
    pub accidental_spacing: f32,
    /// Dot spacing from notehead
    pub dot_spacing: f32,
    /// Dot radius
    pub dot_radius: f32,
}

impl Default for NoteConfig {
    fn default() -> Self {
        Self {
            head_width: STAFF_SPACE * 1.4,
            head_height: STAFF_SPACE,
            stem_width: 1.2,
            stem_height: STAFF_SPACE * 3.5,
            flag_width: STAFF_SPACE,
            beam_thickness: STAFF_SPACE * 0.5,
            ledger_width: STAFF_SPACE * 2.0,
            ledger_extension: 4.0,
            accidental_spacing: STAFF_SPACE * 0.8,
            dot_spacing: STAFF_SPACE * 0.5,
            dot_radius: STAFF_SPACE * 0.2,
        }
    }
}

/// Color scheme for rendering
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Staff line color
    pub staff_lines: (f32, f32, f32, f32),
    /// Note color
    pub notes: (f32, f32, f32, f32),
    /// Rest color
    pub rests: (f32, f32, f32, f32),
    /// Clef color
    pub clefs: (f32, f32, f32, f32),
    /// Bar line color
    pub bar_lines: (f32, f32, f32, f32),
    /// Background color
    pub background: (f32, f32, f32, f32),
    /// Selected note color
    pub selected: (f32, f32, f32, f32),
    /// Accidental color
    pub accidentals: (f32, f32, f32, f32),
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            staff_lines: (0.0, 0.0, 0.0, 1.0),
            notes: (0.0, 0.0, 0.0, 1.0),
            rests: (0.0, 0.0, 0.0, 1.0),
            clefs: (0.0, 0.0, 0.0, 1.0),
            bar_lines: (0.0, 0.0, 0.0, 1.0),
            background: (1.0, 1.0, 1.0, 1.0),
            selected: (0.2, 0.4, 0.8, 1.0),
            accidentals: (0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// Full rendering configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Staff configuration
    pub staff: StaffConfig,
    /// Note configuration
    pub note: NoteConfig,
    /// Color scheme
    pub colors: ColorScheme,

    /// Spacing between staves (for multi-staff systems)
    pub staff_spacing: f32,
    /// Spacing between systems
    pub system_spacing: f32,

    /// Left margin
    pub margin_left: f32,
    /// Right margin
    pub margin_right: f32,
    /// Top margin
    pub margin_top: f32,
    /// Bottom margin
    pub margin_bottom: f32,

    /// Width reserved for clef
    pub clef_width: f32,
    /// Width reserved for key signature
    pub key_sig_width: f32,
    /// Width reserved for time signature
    pub time_sig_width: f32,
    /// Minimum measure width
    pub measure_width: f32,

    /// Scale factor for the entire rendering
    pub scale: f32,
    /// Whether to show bar numbers
    pub show_bar_numbers: bool,
    /// Whether to show ledger lines
    pub show_ledger_lines: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            staff: StaffConfig::default(),
            note: NoteConfig::default(),
            colors: ColorScheme::default(),
            staff_spacing: STAFF_SPACE * 8.0,
            system_spacing: STAFF_SPACE * 12.0,
            margin_left: 40.0,
            margin_right: 20.0,
            margin_top: 40.0,
            margin_bottom: 40.0,
            clef_width: STAFF_SPACE * 4.0,
            key_sig_width: STAFF_SPACE * 2.0,
            time_sig_width: STAFF_SPACE * 3.0,
            measure_width: STAFF_SPACE * 20.0,
            scale: 1.0,
            show_bar_numbers: true,
            show_ledger_lines: true,
        }
    }
}

impl RenderConfig {
    /// Create a configuration for a small preview
    pub fn preview() -> Self {
        Self {
            scale: 0.5,
            show_bar_numbers: false,
            ..Default::default()
        }
    }

    /// Create a configuration for printing
    pub fn print() -> Self {
        Self {
            scale: 1.5,
            ..Default::default()
        }
    }

    /// Create a configuration with custom scale
    pub fn with_scale(scale: f32) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }
}
