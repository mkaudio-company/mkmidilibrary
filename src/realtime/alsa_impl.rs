//! ALSA implementation for Linux

use super::port::{Api, MidiPort};
use super::RtMidiError;

/// Get available MIDI input ports
pub fn get_input_ports() -> Vec<MidiPort> {
    // ALSA implementation would enumerate sequencer clients
    vec![]
}

/// Get available MIDI output ports
pub fn get_output_ports() -> Vec<MidiPort> {
    vec![]
}

/// ALSA MIDI input handler
pub struct AlsaMidiInput {
    // ALSA sequencer handle would go here
}

impl AlsaMidiInput {
    /// Create a new ALSA MIDI input
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let _ = client_name;
        // Would create ALSA sequencer client here
        Ok(Self {})
    }

    /// Open a MIDI input port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let _ = (port_index, port_name);
        Ok(())
    }

    /// Create a virtual MIDI input port
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        let _ = port_name;
        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {}

    /// Set a callback for incoming messages
    pub fn set_callback<F>(&mut self, _callback: F)
    where
        F: FnMut(f64, &[u8]) + Send + 'static,
    {
    }

    /// Cancel the callback
    pub fn cancel_callback(&mut self) {}

    /// Get a message from the queue
    pub fn get_message(&mut self) -> Option<(f64, Vec<u8>)> {
        None
    }

    /// Set message type filtering
    pub fn ignore_types(&mut self, _sysex: bool, _timing: bool, _active_sensing: bool) {}
}

/// ALSA MIDI output handler
pub struct AlsaMidiOutput {
    // ALSA sequencer handle would go here
}

impl AlsaMidiOutput {
    /// Create a new ALSA MIDI output
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let _ = client_name;
        Ok(Self {})
    }

    /// Open a MIDI output port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let _ = (port_index, port_name);
        Ok(())
    }

    /// Create a virtual MIDI output port
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        let _ = port_name;
        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {}

    /// Send a MIDI message
    pub fn send_message(&mut self, _message: &[u8]) -> Result<(), RtMidiError> {
        Ok(())
    }
}
