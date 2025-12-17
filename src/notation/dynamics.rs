//! Dynamic markings

use std::fmt;

/// Dynamic level type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DynamicsType {
    /// As quiet as possible
    PPPP,
    /// Very very quiet
    PPP,
    /// Very quiet (pianissimo)
    PP,
    /// Quiet (piano)
    P,
    /// Moderately quiet (mezzo piano)
    MP,
    /// Moderately loud (mezzo forte)
    MF,
    /// Loud (forte)
    F,
    /// Very loud (fortissimo)
    FF,
    /// Very very loud
    FFF,
    /// As loud as possible
    FFFF,
    /// Sforzando (sudden accent)
    SF,
    /// Sforzando forte
    SFZ,
    /// Sforzando piano
    SFP,
    /// Forzando
    FZ,
    /// Rinforzando
    RF,
    /// Rinforzando forte
    RFZ,
}

impl DynamicsType {
    /// Get the text representation
    pub fn text(&self) -> &'static str {
        match self {
            DynamicsType::PPPP => "pppp",
            DynamicsType::PPP => "ppp",
            DynamicsType::PP => "pp",
            DynamicsType::P => "p",
            DynamicsType::MP => "mp",
            DynamicsType::MF => "mf",
            DynamicsType::F => "f",
            DynamicsType::FF => "ff",
            DynamicsType::FFF => "fff",
            DynamicsType::FFFF => "ffff",
            DynamicsType::SF => "sf",
            DynamicsType::SFZ => "sfz",
            DynamicsType::SFP => "sfp",
            DynamicsType::FZ => "fz",
            DynamicsType::RF => "rf",
            DynamicsType::RFZ => "rfz",
        }
    }

    /// Get the full name
    pub fn name(&self) -> &'static str {
        match self {
            DynamicsType::PPPP => "pianissississimo",
            DynamicsType::PPP => "pianississimo",
            DynamicsType::PP => "pianissimo",
            DynamicsType::P => "piano",
            DynamicsType::MP => "mezzo piano",
            DynamicsType::MF => "mezzo forte",
            DynamicsType::F => "forte",
            DynamicsType::FF => "fortissimo",
            DynamicsType::FFF => "fortississimo",
            DynamicsType::FFFF => "fortissississimo",
            DynamicsType::SF => "sforzando",
            DynamicsType::SFZ => "sforzato",
            DynamicsType::SFP => "sforzando piano",
            DynamicsType::FZ => "forzando",
            DynamicsType::RF => "rinforzando",
            DynamicsType::RFZ => "rinforzato",
        }
    }

    /// Get typical MIDI velocity
    pub fn velocity(&self) -> u8 {
        match self {
            DynamicsType::PPPP => 16,
            DynamicsType::PPP => 24,
            DynamicsType::PP => 36,
            DynamicsType::P => 48,
            DynamicsType::MP => 64,
            DynamicsType::MF => 80,
            DynamicsType::F => 96,
            DynamicsType::FF => 112,
            DynamicsType::FFF => 120,
            DynamicsType::FFFF => 127,
            DynamicsType::SF | DynamicsType::SFZ | DynamicsType::FZ | DynamicsType::RFZ => 112,
            DynamicsType::SFP => 96,
            DynamicsType::RF => 100,
        }
    }

    /// Get volume scalar (0.0-1.0)
    pub fn volume(&self) -> f64 {
        self.velocity() as f64 / 127.0
    }

    /// Check if this is an accent/sforzando type
    pub fn is_accent(&self) -> bool {
        matches!(
            self,
            DynamicsType::SF
                | DynamicsType::SFZ
                | DynamicsType::SFP
                | DynamicsType::FZ
                | DynamicsType::RF
                | DynamicsType::RFZ
        )
    }
}

impl fmt::Display for DynamicsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// A dynamic marking
#[derive(Debug, Clone, PartialEq)]
pub struct Dynamics {
    /// Dynamic type
    type_: DynamicsType,
    /// Custom velocity override
    velocity_override: Option<u8>,
}

impl Dynamics {
    /// Create a new dynamic marking
    pub fn new(type_: DynamicsType) -> Self {
        Self {
            type_,
            velocity_override: None,
        }
    }

    /// Create piano
    pub fn p() -> Self {
        Self::new(DynamicsType::P)
    }

    /// Create mezzo piano
    pub fn mp() -> Self {
        Self::new(DynamicsType::MP)
    }

    /// Create mezzo forte
    pub fn mf() -> Self {
        Self::new(DynamicsType::MF)
    }

    /// Create forte
    pub fn f() -> Self {
        Self::new(DynamicsType::F)
    }

    /// Create fortissimo
    pub fn ff() -> Self {
        Self::new(DynamicsType::FF)
    }

    /// Create pianissimo
    pub fn pp() -> Self {
        Self::new(DynamicsType::PP)
    }

    /// Get the type
    pub fn type_(&self) -> DynamicsType {
        self.type_
    }

    /// Get the velocity
    pub fn velocity(&self) -> u8 {
        self.velocity_override.unwrap_or_else(|| self.type_.velocity())
    }

    /// Set custom velocity
    pub fn set_velocity(&mut self, velocity: u8) {
        self.velocity_override = Some(velocity);
    }

    /// Get volume scalar
    pub fn volume(&self) -> f64 {
        self.velocity() as f64 / 127.0
    }
}

impl Default for Dynamics {
    fn default() -> Self {
        Self::mf()
    }
}

impl fmt::Display for Dynamics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_)
    }
}

impl From<DynamicsType> for Dynamics {
    fn from(type_: DynamicsType) -> Self {
        Self::new(type_)
    }
}

/// Hairpin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HairpinType {
    /// Crescendo (getting louder)
    Crescendo,
    /// Decrescendo / Diminuendo (getting softer)
    Decrescendo,
}

impl HairpinType {
    /// Get the text representation
    pub fn text(&self) -> &'static str {
        match self {
            HairpinType::Crescendo => "cresc.",
            HairpinType::Decrescendo => "decresc.",
        }
    }
}

impl fmt::Display for HairpinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// A hairpin (crescendo/decrescendo)
#[derive(Debug, Clone, PartialEq)]
pub struct Hairpin {
    /// Hairpin type
    type_: HairpinType,
    /// Start dynamic (optional)
    start_dynamic: Option<Dynamics>,
    /// End dynamic (optional)
    end_dynamic: Option<Dynamics>,
}

impl Hairpin {
    /// Create a new hairpin
    pub fn new(type_: HairpinType) -> Self {
        Self {
            type_,
            start_dynamic: None,
            end_dynamic: None,
        }
    }

    /// Create a crescendo
    pub fn crescendo() -> Self {
        Self::new(HairpinType::Crescendo)
    }

    /// Create a decrescendo
    pub fn decrescendo() -> Self {
        Self::new(HairpinType::Decrescendo)
    }

    /// Get the type
    pub fn type_(&self) -> HairpinType {
        self.type_
    }

    /// Set start dynamic
    pub fn set_start(&mut self, dynamic: Dynamics) {
        self.start_dynamic = Some(dynamic);
    }

    /// Set end dynamic
    pub fn set_end(&mut self, dynamic: Dynamics) {
        self.end_dynamic = Some(dynamic);
    }

    /// Get start dynamic
    pub fn start(&self) -> Option<&Dynamics> {
        self.start_dynamic.as_ref()
    }

    /// Get end dynamic
    pub fn end(&self) -> Option<&Dynamics> {
        self.end_dynamic.as_ref()
    }
}

impl fmt::Display for Hairpin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamics_creation() {
        let dyn_ = Dynamics::f();
        assert_eq!(dyn_.type_(), DynamicsType::F);
        assert!(dyn_.velocity() > 80);
    }

    #[test]
    fn test_dynamics_ordering() {
        assert!(DynamicsType::PP < DynamicsType::P);
        assert!(DynamicsType::P < DynamicsType::MF);
        assert!(DynamicsType::MF < DynamicsType::F);
        assert!(DynamicsType::F < DynamicsType::FF);
    }

    #[test]
    fn test_dynamics_velocity() {
        assert!(Dynamics::pp().velocity() < Dynamics::p().velocity());
        assert!(Dynamics::p().velocity() < Dynamics::mf().velocity());
        assert!(Dynamics::mf().velocity() < Dynamics::f().velocity());
    }

    #[test]
    fn test_hairpin() {
        let mut cresc = Hairpin::crescendo();
        cresc.set_start(Dynamics::p());
        cresc.set_end(Dynamics::f());

        assert_eq!(cresc.type_(), HairpinType::Crescendo);
        assert!(cresc.start().is_some());
        assert!(cresc.end().is_some());
    }
}
