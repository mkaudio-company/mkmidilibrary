//! CoreMIDI implementation for macOS

use coremidi::{
    Client, Destination, Destinations, InputPort, OutputPort, PacketBuffer, Source, Sources,
    VirtualDestination, VirtualSource,
};
use std::sync::{Arc, Mutex};

use super::port::{Api, MidiPort};
use super::RtMidiError;

/// Get available MIDI input sources
pub fn get_input_ports() -> Vec<MidiPort> {
    Sources
        .into_iter()
        .enumerate()
        .filter_map(|(i, source)| {
            source
                .display_name()
                .map(|name| MidiPort::new(i, name, Api::CoreMidi))
        })
        .collect()
}

/// Get available MIDI output destinations
pub fn get_output_ports() -> Vec<MidiPort> {
    Destinations
        .into_iter()
        .enumerate()
        .filter_map(|(i, dest)| {
            dest.display_name()
                .map(|name| MidiPort::new(i, name, Api::CoreMidi))
        })
        .collect()
}

/// CoreMIDI input handler
pub struct CoreMidiInput {
    client: Client,
    input_port: Option<InputPort>,
    connected_source: Option<Source>,
    virtual_source: Option<VirtualSource>,
    callback_data: Arc<Mutex<CallbackData>>,
}

struct CallbackData {
    callback: Option<Box<dyn FnMut(f64, &[u8]) + Send>>,
    queue: std::collections::VecDeque<(f64, Vec<u8>)>,
    ignore_sysex: bool,
    ignore_timing: bool,
    ignore_active_sensing: bool,
    start_time: std::time::Instant,
}

impl CoreMidiInput {
    /// Create a new CoreMIDI input
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let client = Client::new(client_name)
            .map_err(|e| RtMidiError::DriverError(format!("Failed to create MIDI client: {}", e)))?;

        let callback_data = Arc::new(Mutex::new(CallbackData {
            callback: None,
            queue: std::collections::VecDeque::new(),
            ignore_sysex: false,
            ignore_timing: true,
            ignore_active_sensing: true,
            start_time: std::time::Instant::now(),
        }));

        Ok(Self {
            client,
            input_port: None,
            connected_source: None,
            virtual_source: None,
            callback_data,
        })
    }

    /// Open a MIDI input port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let source = Sources.into_iter().nth(port_index).ok_or_else(|| {
            RtMidiError::InvalidPort(port_index)
        })?;

        let callback_data = Arc::clone(&self.callback_data);

        let input_port = self
            .client
            .input_port(port_name, move |packet_list| {
                let mut data = callback_data.lock().unwrap();
                let elapsed = data.start_time.elapsed().as_secs_f64();

                for packet in packet_list.iter() {
                    let msg = packet.data();
                    if msg.is_empty() {
                        continue;
                    }

                    // Filter message types
                    let status = msg[0];
                    if data.ignore_sysex && status == 0xF0 {
                        continue;
                    }
                    if data.ignore_timing && status == 0xF8 {
                        continue;
                    }
                    if data.ignore_active_sensing && status == 0xFE {
                        continue;
                    }

                    if let Some(ref mut cb) = data.callback {
                        cb(elapsed, msg);
                    } else {
                        data.queue.push_back((elapsed, msg.to_vec()));
                    }
                }
            })
            .map_err(|e| {
                RtMidiError::DriverError(format!("Failed to create input port: {}", e))
            })?;

        input_port.connect_source(&source).map_err(|e| {
            RtMidiError::DriverError(format!("Failed to connect to source: {}", e))
        })?;

        self.input_port = Some(input_port);
        self.connected_source = Some(source);

        // Reset start time when port is opened
        if let Ok(mut data) = self.callback_data.lock() {
            data.start_time = std::time::Instant::now();
        }

        Ok(())
    }

    /// Create a virtual MIDI input port
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        let source = self
            .client
            .virtual_source(port_name)
            .map_err(|e| {
                RtMidiError::DriverError(format!("Failed to create virtual source: {}", e))
            })?;

        self.virtual_source = Some(source);

        if let Ok(mut data) = self.callback_data.lock() {
            data.start_time = std::time::Instant::now();
        }

        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {
        if let (Some(port), Some(source)) = (&self.input_port, &self.connected_source) {
            let _ = port.disconnect_source(source);
        }
        self.input_port = None;
        self.connected_source = None;
        self.virtual_source = None;
    }

    /// Set a callback for incoming messages
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnMut(f64, &[u8]) + Send + 'static,
    {
        if let Ok(mut data) = self.callback_data.lock() {
            data.callback = Some(Box::new(callback));
        }
    }

    /// Cancel the callback
    pub fn cancel_callback(&mut self) {
        if let Ok(mut data) = self.callback_data.lock() {
            data.callback = None;
        }
    }

    /// Get a message from the queue (when not using callback)
    pub fn get_message(&mut self) -> Option<(f64, Vec<u8>)> {
        if let Ok(mut data) = self.callback_data.lock() {
            data.queue.pop_front()
        } else {
            None
        }
    }

    /// Set message type filtering
    pub fn ignore_types(&mut self, sysex: bool, timing: bool, active_sensing: bool) {
        if let Ok(mut data) = self.callback_data.lock() {
            data.ignore_sysex = sysex;
            data.ignore_timing = timing;
            data.ignore_active_sensing = active_sensing;
        }
    }
}

/// CoreMIDI output handler
pub struct CoreMidiOutput {
    client: Client,
    output_port: Option<OutputPort>,
    destination: Option<Destination>,
    virtual_destination: Option<VirtualDestination>,
}

impl CoreMidiOutput {
    /// Create a new CoreMIDI output
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let client = Client::new(client_name)
            .map_err(|e| RtMidiError::DriverError(format!("Failed to create MIDI client: {}", e)))?;

        Ok(Self {
            client,
            output_port: None,
            destination: None,
            virtual_destination: None,
        })
    }

    /// Open a MIDI output port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let destination = Destinations.into_iter().nth(port_index).ok_or_else(|| {
            RtMidiError::InvalidPort(port_index)
        })?;

        let output_port = self
            .client
            .output_port(port_name)
            .map_err(|e| {
                RtMidiError::DriverError(format!("Failed to create output port: {}", e))
            })?;

        self.output_port = Some(output_port);
        self.destination = Some(destination);

        Ok(())
    }

    /// Create a virtual MIDI output port
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        let destination = self
            .client
            .virtual_destination(port_name, |_packet_list| {
                // Virtual destination callback - typically not needed for output
            })
            .map_err(|e| {
                RtMidiError::DriverError(format!("Failed to create virtual destination: {}", e))
            })?;

        self.virtual_destination = Some(destination);

        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {
        self.output_port = None;
        self.destination = None;
        self.virtual_destination = None;
    }

    /// Send a MIDI message
    pub fn send_message(&mut self, message: &[u8]) -> Result<(), RtMidiError> {
        let packet_buffer = PacketBuffer::new(0, message);

        if let (Some(port), Some(dest)) = (&self.output_port, &self.destination) {
            port.send(dest, &packet_buffer).map_err(|e| {
                RtMidiError::DriverError(format!("Failed to send message: {}", e))
            })?;
        } else if self.virtual_destination.is_some() {
            // Virtual destinations in CoreMIDI are meant for receiving, not sending.
            // To send to a virtual destination, we need a different approach.
            // For now, virtual output ports are primarily for other apps to connect to.
            return Err(RtMidiError::DriverError(
                "Virtual output ports cannot send messages directly".to_string(),
            ));
        } else {
            return Err(RtMidiError::PortNotOpen);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ports() {
        // This test just verifies the functions don't panic
        let inputs = get_input_ports();
        let outputs = get_output_ports();
        println!("Found {} input ports and {} output ports", inputs.len(), outputs.len());
    }
}
