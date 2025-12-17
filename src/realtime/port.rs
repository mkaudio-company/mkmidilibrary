//! MIDI port enumeration

use std::fmt;

/// Available MIDI APIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Api {
    /// Unspecified API (let the system choose)
    Unspecified,
    /// macOS CoreMIDI
    CoreMidi,
    /// Linux ALSA
    Alsa,
    /// Linux JACK
    Jack,
    /// Windows Multimedia API
    WindowsMm,
    /// Windows UWP
    WindowsUwp,
    /// Web MIDI
    WebMidi,
    /// Dummy (no real I/O)
    Dummy,
}

impl Api {
    /// Get the name of this API
    pub fn name(&self) -> &'static str {
        match self {
            Api::Unspecified => "unspecified",
            Api::CoreMidi => "CoreMIDI",
            Api::Alsa => "ALSA",
            Api::Jack => "JACK",
            Api::WindowsMm => "Windows MM",
            Api::WindowsUwp => "Windows UWP",
            Api::WebMidi => "Web MIDI",
            Api::Dummy => "Dummy",
        }
    }

    /// Get the default API for the current platform
    pub fn default_for_platform() -> Api {
        #[cfg(target_os = "macos")]
        return Api::CoreMidi;

        #[cfg(target_os = "linux")]
        return Api::Alsa;

        #[cfg(target_os = "windows")]
        return Api::WindowsMm;

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return Api::Dummy;
    }

    /// Get all available APIs on the current platform
    pub fn available() -> Vec<Api> {
        let mut apis = Vec::new();

        #[cfg(target_os = "macos")]
        apis.push(Api::CoreMidi);

        #[cfg(target_os = "linux")]
        {
            apis.push(Api::Alsa);
            // JACK could be detected at runtime
        }

        #[cfg(target_os = "windows")]
        apis.push(Api::WindowsMm);

        if apis.is_empty() {
            apis.push(Api::Dummy);
        }

        apis
    }
}

impl Default for Api {
    fn default() -> Self {
        Api::default_for_platform()
    }
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Information about a MIDI port
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiPort {
    /// Port index
    index: usize,
    /// Port name
    name: String,
    /// API this port belongs to
    api: Api,
}

impl MidiPort {
    /// Create a new port info
    pub fn new(index: usize, name: impl Into<String>, api: Api) -> Self {
        Self {
            index,
            name: name.into(),
            api,
        }
    }

    /// Get the port index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the port name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the API
    pub fn api(&self) -> Api {
        self.api
    }
}

impl fmt::Display for MidiPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.index, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_default() {
        let api = Api::default();
        assert!(Api::available().contains(&api) || api == Api::Dummy);
    }

    #[test]
    fn test_port_creation() {
        let port = MidiPort::new(0, "Test Port", Api::Dummy);
        assert_eq!(port.index(), 0);
        assert_eq!(port.name(), "Test Port");
    }
}
