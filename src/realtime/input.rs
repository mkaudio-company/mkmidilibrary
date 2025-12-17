//! Real-time MIDI input

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::port::{Api, MidiPort};
use super::{MidiCallback, MidiInputConfig, RtMidiError};

#[cfg(target_os = "macos")]
use super::coremidi_impl::CoreMidiInput;

/// Timestamped MIDI message
#[derive(Debug, Clone)]
pub struct TimestampedMessage {
    /// Timestamp in seconds (relative to port opening)
    pub timestamp: f64,
    /// MIDI message bytes
    pub data: Vec<u8>,
}

/// Real-time MIDI input
pub struct MidiInput {
    /// Client name
    client_name: String,
    /// Current API
    api: Api,
    /// Configuration
    config: MidiInputConfig,
    /// Whether a port is open
    port_open: bool,
    /// Port name (when open)
    port_name: Option<String>,
    /// Message queue (when not using callbacks)
    queue: Arc<Mutex<VecDeque<TimestampedMessage>>>,
    /// Callback (when using callbacks)
    callback: Option<MidiCallback>,
    /// Platform-specific data
    #[cfg(target_os = "macos")]
    platform: Option<PlatformInput>,
    #[cfg(target_os = "linux")]
    platform: Option<PlatformInput>,
    #[cfg(target_os = "windows")]
    platform: Option<PlatformInput>,
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    platform: Option<()>,
}

#[cfg(target_os = "macos")]
type PlatformInput = CoreMidiInput;

#[cfg(target_os = "linux")]
type PlatformInput = super::alsa_impl::AlsaMidiInput;

#[cfg(target_os = "windows")]
type PlatformInput = super::winmm_impl::WinMmMidiInput;

impl MidiInput {
    /// Create a new MIDI input
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        Self::with_api(Api::default(), client_name)
    }

    /// Create a new MIDI input with specific API
    pub fn with_api(api: Api, client_name: &str) -> Result<Self, RtMidiError> {
        Ok(Self {
            client_name: client_name.to_string(),
            api,
            config: MidiInputConfig::default(),
            port_open: false,
            port_name: None,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            callback: None,
            platform: None,
        })
    }

    /// Get the API being used
    pub fn api(&self) -> Api {
        self.api
    }

    /// Get the client name
    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    /// Get the configuration
    pub fn config(&self) -> &MidiInputConfig {
        &self.config
    }

    /// Set the configuration (before opening a port)
    pub fn set_config(&mut self, config: MidiInputConfig) {
        self.config = config;
    }

    /// Get available input ports
    pub fn ports(&self) -> Vec<MidiPort> {
        self.get_ports_impl()
    }

    /// Get the number of available ports
    pub fn port_count(&self) -> usize {
        self.ports().len()
    }

    /// Get the name of a specific port
    pub fn port_name(&self, index: usize) -> Option<String> {
        self.ports().get(index).map(|p| p.name().to_string())
    }

    /// Open a MIDI input port
    pub fn open_port(&mut self, port: usize, port_name: &str) -> Result<(), RtMidiError> {
        if self.port_open {
            return Err(RtMidiError::PortAlreadyOpen);
        }

        let ports = self.ports();
        if port >= ports.len() {
            return Err(RtMidiError::InvalidPort(port));
        }

        self.open_port_impl(port, port_name)?;
        self.port_open = true;
        self.port_name = Some(port_name.to_string());
        Ok(())
    }

    /// Create a virtual input port
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        if self.port_open {
            return Err(RtMidiError::PortAlreadyOpen);
        }

        self.open_virtual_port_impl(port_name)?;
        self.port_open = true;
        self.port_name = Some(port_name.to_string());
        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {
        if self.port_open {
            self.close_port_impl();
            self.port_open = false;
            self.port_name = None;
        }
    }

    /// Check if a port is open
    pub fn is_port_open(&self) -> bool {
        self.port_open
    }

    /// Set a callback for incoming messages
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnMut(f64, &[u8]) + Send + 'static,
    {
        self.callback = Some(Box::new(callback));
    }

    /// Cancel the callback and return to queue-based input
    pub fn cancel_callback(&mut self) {
        self.callback = None;
    }

    /// Get a message from the queue (non-blocking)
    pub fn get_message(&mut self) -> Option<TimestampedMessage> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    /// Set which message types to ignore
    pub fn ignore_types(&mut self, sysex: bool, timing: bool, active_sensing: bool) {
        self.config.ignore_sysex = sysex;
        self.config.ignore_timing = timing;
        self.config.ignore_active_sensing = active_sensing;
    }

    // Platform-specific implementations

    fn get_ports_impl(&self) -> Vec<MidiPort> {
        // Default implementation returns empty
        // Platform-specific code would override this
        match self.api {
            Api::Dummy => vec![MidiPort::new(0, "Dummy Input", Api::Dummy)],
            #[cfg(target_os = "macos")]
            Api::CoreMidi => self.get_ports_coremidi(),
            #[cfg(target_os = "linux")]
            Api::Alsa => self.get_ports_alsa(),
            #[cfg(target_os = "windows")]
            Api::WindowsMm => self.get_ports_winmm(),
            _ => vec![],
        }
    }

    fn open_port_impl(&mut self, _port: usize, _port_name: &str) -> Result<(), RtMidiError> {
        match self.api {
            Api::Dummy => Ok(()),
            #[cfg(target_os = "macos")]
            Api::CoreMidi => self.open_port_coremidi(_port, _port_name),
            #[cfg(target_os = "linux")]
            Api::Alsa => self.open_port_alsa(_port, _port_name),
            #[cfg(target_os = "windows")]
            Api::WindowsMm => self.open_port_winmm(_port, _port_name),
            _ => Err(RtMidiError::DriverError("API not available".to_string())),
        }
    }

    fn open_virtual_port_impl(&mut self, _port_name: &str) -> Result<(), RtMidiError> {
        match self.api {
            Api::Dummy => Ok(()),
            #[cfg(target_os = "macos")]
            Api::CoreMidi => self.open_virtual_port_coremidi(_port_name),
            #[cfg(target_os = "linux")]
            Api::Alsa => self.open_virtual_port_alsa(_port_name),
            _ => Err(RtMidiError::VirtualPortError),
        }
    }

    fn close_port_impl(&mut self) {
        match self.api {
            Api::Dummy => {}
            #[cfg(target_os = "macos")]
            Api::CoreMidi => self.close_port_coremidi(),
            #[cfg(target_os = "linux")]
            Api::Alsa => self.close_port_alsa(),
            #[cfg(target_os = "windows")]
            Api::WindowsMm => self.close_port_winmm(),
            _ => {}
        }
    }

    // CoreMIDI implementations
    #[cfg(target_os = "macos")]
    fn get_ports_coremidi(&self) -> Vec<MidiPort> {
        super::coremidi_impl::get_input_ports()
    }

    #[cfg(target_os = "macos")]
    fn open_port_coremidi(&mut self, port: usize, name: &str) -> Result<(), RtMidiError> {
        let mut platform = CoreMidiInput::new(&self.client_name)?;
        platform.open_port(port, name)?;
        self.platform = Some(platform);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn open_virtual_port_coremidi(&mut self, name: &str) -> Result<(), RtMidiError> {
        let mut platform = CoreMidiInput::new(&self.client_name)?;
        platform.open_virtual_port(name)?;
        self.platform = Some(platform);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn close_port_coremidi(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.close_port();
        }
        self.platform = None;
    }

    #[cfg(target_os = "linux")]
    fn get_ports_alsa(&self) -> Vec<MidiPort> {
        // TODO: Implement ALSA port enumeration
        vec![]
    }

    #[cfg(target_os = "linux")]
    fn open_port_alsa(&mut self, _port: usize, _name: &str) -> Result<(), RtMidiError> {
        // TODO: Implement ALSA port opening
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn open_virtual_port_alsa(&mut self, _name: &str) -> Result<(), RtMidiError> {
        // TODO: Implement ALSA virtual port
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn close_port_alsa(&mut self) {
        // TODO: Implement ALSA port closing
    }

    #[cfg(target_os = "windows")]
    fn get_ports_winmm(&self) -> Vec<MidiPort> {
        // TODO: Implement Windows MM port enumeration
        vec![]
    }

    #[cfg(target_os = "windows")]
    fn open_port_winmm(&mut self, _port: usize, _name: &str) -> Result<(), RtMidiError> {
        // TODO: Implement Windows MM port opening
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn close_port_winmm(&mut self) {
        // TODO: Implement Windows MM port closing
    }
}

impl Drop for MidiInput {
    fn drop(&mut self) {
        self.close_port();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_input_creation() {
        let input = MidiInput::new("Test Client");
        assert!(input.is_ok());
    }

    #[test]
    fn test_midi_input_config() {
        let mut input = MidiInput::new("Test").unwrap();
        let config = MidiInputConfig {
            queue_size: 200,
            ignore_timing: false,
            ..Default::default()
        };
        input.set_config(config);
        assert_eq!(input.config().queue_size, 200);
    }
}
