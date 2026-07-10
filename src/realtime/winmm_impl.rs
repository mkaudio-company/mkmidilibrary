//! Windows Multimedia API implementation

use super::port::MidiPort;
use super::RtMidiError;

/// Get available MIDI input ports
pub fn get_input_ports() -> Vec<MidiPort> {
    // WinMM would use midiInGetNumDevs and midiInGetDevCaps
    vec![]
}

/// Get available MIDI output ports
pub fn get_output_ports() -> Vec<MidiPort> {
    // WinMM would use midiOutGetNumDevs and midiOutGetDevCaps
    vec![]
}

/// Windows MM MIDI input handler
pub struct WinMmMidiInput {
    // HMIDIIN handle would go here
}

impl WinMmMidiInput {
    /// Create a new Windows MM MIDI input
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let _ = client_name;
        Ok(Self {})
    }

    /// Open a MIDI input port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let _ = (port_index, port_name);
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

    /// Set the maximum number of queued messages before incoming messages
    /// are dropped.
    pub fn set_queue_size_limit(&mut self, _limit: usize) {}

    /// Register a callback for non-fatal warnings.
    pub fn set_error_callback<F>(&mut self, _callback: F)
    where
        F: FnMut(&RtMidiError) + Send + 'static,
    {
    }

    /// Remove any registered error callback.
    pub fn cancel_error_callback(&mut self) {}
}

/// Windows MM MIDI output handler
pub struct WinMmMidiOutput {
    // HMIDIOUT handle would go here
}

impl WinMmMidiOutput {
    /// Create a new Windows MM MIDI output
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let _ = client_name;
        Ok(Self {})
    }

    /// Open a MIDI output port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let _ = (port_index, port_name);
        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {}

    /// Send a MIDI message
    pub fn send_message(&mut self, _message: &[u8]) -> Result<(), RtMidiError> {
        Ok(())
    }
}
