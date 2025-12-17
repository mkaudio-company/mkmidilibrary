//! Music analysis tools
//!
//! This module provides tools for analyzing musical content:
//! - Chord identification and analysis
//! - Key detection
//! - Harmonic analysis

mod chord_analysis;

pub use chord_analysis::{ChordAnalyzer, ChordQuality, RomanNumeral};
