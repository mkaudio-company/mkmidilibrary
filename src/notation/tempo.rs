//! Tempo representation

use std::fmt;

use crate::core::DurationType;

/// Common tempo indications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TempoIndication {
    /// Very slow (20-40 BPM)
    Grave,
    /// Slow and solemn (40-60 BPM)
    Largo,
    /// Broadly (45-65 BPM)
    Larghetto,
    /// Slow (55-75 BPM)
    Adagio,
    /// At ease (65-80 BPM)
    Andante,
    /// Walking pace (80-100 BPM)
    Andantino,
    /// Moderate (100-120 BPM)
    Moderato,
    /// Lively (120-140 BPM)
    Allegretto,
    /// Fast (140-170 BPM)
    Allegro,
    /// Very fast (170-200 BPM)
    Vivace,
    /// Lively and fast (170-200 BPM)
    Vivacissimo,
    /// Very quick (180-220 BPM)
    Presto,
    /// As fast as possible (200+ BPM)
    Prestissimo,
}

impl TempoIndication {
    /// Get the typical BPM range
    pub fn bpm_range(&self) -> (f64, f64) {
        match self {
            TempoIndication::Grave => (20.0, 40.0),
            TempoIndication::Largo => (40.0, 60.0),
            TempoIndication::Larghetto => (45.0, 65.0),
            TempoIndication::Adagio => (55.0, 75.0),
            TempoIndication::Andante => (65.0, 80.0),
            TempoIndication::Andantino => (80.0, 100.0),
            TempoIndication::Moderato => (100.0, 120.0),
            TempoIndication::Allegretto => (120.0, 140.0),
            TempoIndication::Allegro => (140.0, 170.0),
            TempoIndication::Vivace => (170.0, 200.0),
            TempoIndication::Vivacissimo => (170.0, 200.0),
            TempoIndication::Presto => (180.0, 220.0),
            TempoIndication::Prestissimo => (200.0, 280.0),
        }
    }

    /// Get the typical BPM
    pub fn typical_bpm(&self) -> f64 {
        let (min, max) = self.bpm_range();
        (min + max) / 2.0
    }

    /// Get the Italian name
    pub fn name(&self) -> &'static str {
        match self {
            TempoIndication::Grave => "Grave",
            TempoIndication::Largo => "Largo",
            TempoIndication::Larghetto => "Larghetto",
            TempoIndication::Adagio => "Adagio",
            TempoIndication::Andante => "Andante",
            TempoIndication::Andantino => "Andantino",
            TempoIndication::Moderato => "Moderato",
            TempoIndication::Allegretto => "Allegretto",
            TempoIndication::Allegro => "Allegro",
            TempoIndication::Vivace => "Vivace",
            TempoIndication::Vivacissimo => "Vivacissimo",
            TempoIndication::Presto => "Presto",
            TempoIndication::Prestissimo => "Prestissimo",
        }
    }

    /// Infer tempo indication from BPM
    pub fn from_bpm(bpm: f64) -> Option<TempoIndication> {
        let indications = [
            TempoIndication::Grave,
            TempoIndication::Largo,
            TempoIndication::Adagio,
            TempoIndication::Andante,
            TempoIndication::Moderato,
            TempoIndication::Allegretto,
            TempoIndication::Allegro,
            TempoIndication::Vivace,
            TempoIndication::Presto,
            TempoIndication::Prestissimo,
        ];

        for indication in indications {
            let (min, max) = indication.bpm_range();
            if bpm >= min && bpm <= max {
                return Some(indication);
            }
        }

        None
    }
}

impl fmt::Display for TempoIndication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A metronome marking (beat = BPM)
#[derive(Debug, Clone, PartialEq)]
pub struct MetronomeMark {
    /// The beat unit (quarter, half, etc.)
    beat_unit: DurationType,
    /// Number of dots on the beat unit
    dots: u8,
    /// Beats per minute
    bpm: f64,
}

impl MetronomeMark {
    /// Create a new metronome mark
    pub fn new(beat_unit: DurationType, bpm: f64) -> Self {
        Self {
            beat_unit,
            dots: 0,
            bpm,
        }
    }

    /// Create with dotted beat unit
    pub fn dotted(beat_unit: DurationType, bpm: f64) -> Self {
        Self {
            beat_unit,
            dots: 1,
            bpm,
        }
    }

    /// Get the beat unit
    pub fn beat_unit(&self) -> DurationType {
        self.beat_unit
    }

    /// Get the number of dots
    pub fn dots(&self) -> u8 {
        self.dots
    }

    /// Get the BPM
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// Set the BPM
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }
}

impl fmt::Display for MetronomeMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dots = ".".repeat(self.dots as usize);
        write!(f, "{}{} = {:.0}", self.beat_unit, dots, self.bpm)
    }
}

/// A tempo marking
#[derive(Debug, Clone, PartialEq)]
pub struct Tempo {
    /// BPM value
    bpm: f64,
    /// Optional tempo indication
    indication: Option<TempoIndication>,
    /// Optional metronome mark
    metronome: Option<MetronomeMark>,
    /// Optional text description
    text: Option<String>,
}

impl Tempo {
    /// Create a new tempo from BPM
    pub fn new(bpm: f64) -> Self {
        Self {
            bpm,
            indication: TempoIndication::from_bpm(bpm),
            metronome: Some(MetronomeMark::new(DurationType::Quarter, bpm)),
            text: None,
        }
    }

    /// Create from a tempo indication
    pub fn from_indication(indication: TempoIndication) -> Self {
        let bpm = indication.typical_bpm();
        Self {
            bpm,
            indication: Some(indication),
            metronome: Some(MetronomeMark::new(DurationType::Quarter, bpm)),
            text: Some(indication.name().to_string()),
        }
    }

    /// Create with custom text
    pub fn with_text(bpm: f64, text: impl Into<String>) -> Self {
        Self {
            bpm,
            indication: TempoIndication::from_bpm(bpm),
            metronome: Some(MetronomeMark::new(DurationType::Quarter, bpm)),
            text: Some(text.into()),
        }
    }

    /// Get the BPM
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// Set the BPM
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
        self.indication = TempoIndication::from_bpm(bpm);
        if let Some(ref mut metro) = self.metronome {
            metro.set_bpm(bpm);
        }
    }

    /// Get the tempo indication
    pub fn indication(&self) -> Option<TempoIndication> {
        self.indication
    }

    /// Set the tempo indication
    pub fn set_indication(&mut self, indication: TempoIndication) {
        self.indication = Some(indication);
    }

    /// Get the metronome mark
    pub fn metronome(&self) -> Option<&MetronomeMark> {
        self.metronome.as_ref()
    }

    /// Get the text
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Set the text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = Some(text.into());
    }

    /// Get microseconds per quarter note (for MIDI)
    pub fn microseconds_per_quarter(&self) -> u32 {
        (60_000_000.0 / self.bpm).round() as u32
    }

    /// Create from microseconds per quarter note
    pub fn from_microseconds(us: u32) -> Self {
        let bpm = 60_000_000.0 / us as f64;
        Self::new(bpm)
    }

    /// Get seconds per beat
    pub fn seconds_per_beat(&self) -> f64 {
        60.0 / self.bpm
    }
}

impl Default for Tempo {
    fn default() -> Self {
        Self::new(120.0)
    }
}

impl fmt::Display for Tempo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(text) = &self.text {
            write!(f, "{} ({:.0} BPM)", text, self.bpm)
        } else if let Some(metro) = &self.metronome {
            write!(f, "{}", metro)
        } else {
            write!(f, "{:.0} BPM", self.bpm)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_creation() {
        let tempo = Tempo::new(120.0);
        assert_eq!(tempo.bpm(), 120.0);
        assert_eq!(tempo.indication(), Some(TempoIndication::Moderato));
    }

    #[test]
    fn test_tempo_indication() {
        let allegro = TempoIndication::Allegro;
        assert_eq!(allegro.name(), "Allegro");
        let (min, max) = allegro.bpm_range();
        assert!(min < max);
    }

    #[test]
    fn test_tempo_from_indication() {
        let tempo = Tempo::from_indication(TempoIndication::Andante);
        let (min, max) = TempoIndication::Andante.bpm_range();
        assert!(tempo.bpm() >= min && tempo.bpm() <= max);
    }

    #[test]
    fn test_metronome_mark() {
        let mark = MetronomeMark::new(DurationType::Quarter, 120.0);
        assert_eq!(mark.beat_unit(), DurationType::Quarter);
        assert_eq!(mark.bpm(), 120.0);
    }

    #[test]
    fn test_microseconds() {
        let tempo = Tempo::new(120.0);
        assert_eq!(tempo.microseconds_per_quarter(), 500_000);

        let tempo2 = Tempo::from_microseconds(500_000);
        assert!((tempo2.bpm() - 120.0).abs() < 0.01);
    }
}
