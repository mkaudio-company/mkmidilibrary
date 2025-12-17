//! Real-time MIDI output

use super::port::{Api, MidiPort};
use super::RtMidiError;

#[cfg(target_os = "macos")]
use super::coremidi_impl::CoreMidiOutput;

/// Real-time MIDI output
pub struct MidiOutput {
    /// Client name
    client_name: String,
    /// Current API
    api: Api,
    /// Whether a port is open
    port_open: bool,
    /// Port name (when open)
    port_name: Option<String>,
    /// Platform-specific data
    #[cfg(target_os = "macos")]
    platform: Option<PlatformOutput>,
    #[cfg(target_os = "linux")]
    platform: Option<PlatformOutput>,
    #[cfg(target_os = "windows")]
    platform: Option<PlatformOutput>,
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    platform: Option<()>,
}

#[cfg(target_os = "macos")]
type PlatformOutput = CoreMidiOutput;

#[cfg(target_os = "linux")]
type PlatformOutput = super::alsa_impl::AlsaMidiOutput;

#[cfg(target_os = "windows")]
type PlatformOutput = super::winmm_impl::WinMmMidiOutput;

impl MidiOutput {
    /// Create a new MIDI output
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        Self::with_api(Api::default(), client_name)
    }

    /// Create a new MIDI output with specific API
    pub fn with_api(api: Api, client_name: &str) -> Result<Self, RtMidiError> {
        Ok(Self {
            client_name: client_name.to_string(),
            api,
            port_open: false,
            port_name: None,
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

    /// Get available output ports
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

    /// Open a MIDI output port
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

    /// Create a virtual output port
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

    /// Send a MIDI message
    pub fn send_message(&mut self, message: &[u8]) -> Result<(), RtMidiError> {
        if !self.port_open {
            return Err(RtMidiError::PortNotOpen);
        }

        if message.is_empty() {
            return Err(RtMidiError::InvalidMessage);
        }

        self.send_message_impl(message)
    }

    /// Send a note on message
    pub fn send_note_on(&mut self, channel: u8, key: u8, velocity: u8) -> Result<(), RtMidiError> {
        self.send_message(&[0x90 | (channel & 0x0F), key & 0x7F, velocity & 0x7F])
    }

    /// Send a note off message
    pub fn send_note_off(&mut self, channel: u8, key: u8, velocity: u8) -> Result<(), RtMidiError> {
        self.send_message(&[0x80 | (channel & 0x0F), key & 0x7F, velocity & 0x7F])
    }

    /// Send a control change message
    pub fn send_control_change(
        &mut self,
        channel: u8,
        controller: u8,
        value: u8,
    ) -> Result<(), RtMidiError> {
        self.send_message(&[0xB0 | (channel & 0x0F), controller & 0x7F, value & 0x7F])
    }

    /// Send a program change message
    pub fn send_program_change(&mut self, channel: u8, program: u8) -> Result<(), RtMidiError> {
        self.send_message(&[0xC0 | (channel & 0x0F), program & 0x7F])
    }

    /// Send a pitch bend message
    pub fn send_pitch_bend(&mut self, channel: u8, value: u16) -> Result<(), RtMidiError> {
        let value = value & 0x3FFF;
        self.send_message(&[
            0xE0 | (channel & 0x0F),
            (value & 0x7F) as u8,
            (value >> 7) as u8,
        ])
    }

    /// Send all notes off on a channel
    pub fn send_all_notes_off(&mut self, channel: u8) -> Result<(), RtMidiError> {
        self.send_control_change(channel, 123, 0)
    }

    /// Send all sound off on a channel
    pub fn send_all_sound_off(&mut self, channel: u8) -> Result<(), RtMidiError> {
        self.send_control_change(channel, 120, 0)
    }

    // Platform-specific implementations

    fn get_ports_impl(&self) -> Vec<MidiPort> {
        match self.api {
            Api::Dummy => vec![MidiPort::new(0, "Dummy Output", Api::Dummy)],
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

    fn send_message_impl(&mut self, _message: &[u8]) -> Result<(), RtMidiError> {
        match self.api {
            Api::Dummy => Ok(()),
            #[cfg(target_os = "macos")]
            Api::CoreMidi => self.send_message_coremidi(_message),
            #[cfg(target_os = "linux")]
            Api::Alsa => self.send_message_alsa(_message),
            #[cfg(target_os = "windows")]
            Api::WindowsMm => self.send_message_winmm(_message),
            _ => Err(RtMidiError::DriverError("API not available".to_string())),
        }
    }

    // CoreMIDI implementations
    #[cfg(target_os = "macos")]
    fn get_ports_coremidi(&self) -> Vec<MidiPort> {
        super::coremidi_impl::get_output_ports()
    }

    #[cfg(target_os = "macos")]
    fn open_port_coremidi(&mut self, port: usize, name: &str) -> Result<(), RtMidiError> {
        let mut platform = CoreMidiOutput::new(&self.client_name)?;
        platform.open_port(port, name)?;
        self.platform = Some(platform);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn open_virtual_port_coremidi(&mut self, name: &str) -> Result<(), RtMidiError> {
        let mut platform = CoreMidiOutput::new(&self.client_name)?;
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

    #[cfg(target_os = "macos")]
    fn send_message_coremidi(&mut self, message: &[u8]) -> Result<(), RtMidiError> {
        if let Some(ref mut p) = self.platform {
            p.send_message(message)
        } else {
            Err(RtMidiError::PortNotOpen)
        }
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

    #[cfg(target_os = "linux")]
    fn send_message_alsa(&mut self, _message: &[u8]) -> Result<(), RtMidiError> {
        // TODO: Implement ALSA message sending
        Ok(())
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

    #[cfg(target_os = "windows")]
    fn send_message_winmm(&mut self, _message: &[u8]) -> Result<(), RtMidiError> {
        // TODO: Implement Windows MM message sending
        Ok(())
    }
}

impl Drop for MidiOutput {
    fn drop(&mut self) {
        // Send all notes off before closing
        if self.port_open {
            for channel in 0..16 {
                let _ = self.send_all_notes_off(channel);
            }
        }
        self.close_port();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_output_creation() {
        let output = MidiOutput::new("Test Client");
        assert!(output.is_ok());
    }

    #[test]
    fn test_midi_output_not_open() {
        let mut output = MidiOutput::new("Test").unwrap();
        let result = output.send_message(&[0x90, 60, 100]);
        assert!(matches!(result, Err(RtMidiError::PortNotOpen)));
    }
}
