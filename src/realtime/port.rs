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
    /// Android AMidi
    AndroidAmidi,
    /// Dummy (no real I/O)
    Dummy,
}

impl Api {
    /// Get the human-readable display name of this API (e.g. "macOS CoreMIDI").
    /// This may change across versions; use `id()` for a stable identifier
    /// suitable for config files.
    pub fn name(&self) -> &'static str {
        match self {
            Api::Unspecified => "Unspecified",
            Api::CoreMidi => "macOS CoreMIDI",
            Api::Alsa => "Linux ALSA",
            Api::Jack => "JACK Audio Connection Kit",
            Api::WindowsMm => "Windows MultiMedia",
            Api::WindowsUwp => "Windows UWP",
            Api::WebMidi => "Web MIDI",
            Api::AndroidAmidi => "Android AMidi",
            Api::Dummy => "Dummy",
        }
    }

    /// Get a short, stable, lower-case machine identifier for this API
    /// (e.g. "core", "alsa"), guaranteed stable across versions — suitable
    /// for serializing an API choice to a config file. Use `name()` for a
    /// human-readable display string instead. The inverse of `from_id`.
    pub fn id(&self) -> &'static str {
        match self {
            Api::Unspecified => "unspecified",
            Api::CoreMidi => "core",
            Api::Alsa => "alsa",
            Api::Jack => "jack",
            Api::WindowsMm => "windows_ms",
            Api::WindowsUwp => "windows_uwp",
            Api::WebMidi => "web_midi",
            Api::AndroidAmidi => "android",
            Api::Dummy => "dummy",
        }
    }

    /// Parse an `Api` from its stable machine identifier (see `id()`).
    /// Returns `None` for an unrecognized identifier.
    pub fn from_id(id: &str) -> Option<Api> {
        match id {
            "unspecified" => Some(Api::Unspecified),
            "core" => Some(Api::CoreMidi),
            "alsa" => Some(Api::Alsa),
            "jack" => Some(Api::Jack),
            "windows_ms" => Some(Api::WindowsMm),
            "windows_uwp" => Some(Api::WindowsUwp),
            "web_midi" => Some(Api::WebMidi),
            "android" => Some(Api::AndroidAmidi),
            "dummy" => Some(Api::Dummy),
            _ => None,
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

    #[test]
    fn test_api_id_roundtrip() {
        for api in [
            Api::Unspecified,
            Api::CoreMidi,
            Api::Alsa,
            Api::Jack,
            Api::WindowsMm,
            Api::WindowsUwp,
            Api::WebMidi,
            Api::AndroidAmidi,
            Api::Dummy,
        ] {
            assert_eq!(Api::from_id(api.id()), Some(api));
        }
        assert_eq!(Api::from_id("not-a-real-api"), None);
    }

    #[test]
    fn test_api_id_stable_and_name_display() {
        // id() is the stable, serializable identifier; name() is a
        // human-readable display string. They must not be conflated.
        assert_eq!(Api::CoreMidi.id(), "core");
        assert_eq!(Api::CoreMidi.name(), "macOS CoreMIDI");
    }
}
