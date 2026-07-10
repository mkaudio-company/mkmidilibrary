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
    /// Virtual destination: this is the correct CoreMIDI primitive for a
    /// virtual *input* port. Other applications send MIDI data to a virtual
    /// destination via their output port, and CoreMIDI invokes our read
    /// callback here — the same role `input_port`/`connected_source` play
    /// for a real hardware source. (A `VirtualSource` is the opposite
    /// direction: it lets other apps' input ports connect to *us* as a
    /// source, with no callback of our own — that's the right primitive for
    /// `CoreMidiOutput`'s virtual port, not this one.)
    virtual_destination: Option<VirtualDestination>,
    callback_data: Arc<Mutex<CallbackData>>,
}

/// A registered message callback: `(delta_time_seconds, message_bytes)`.
type MessageCallback = Box<dyn FnMut(f64, &[u8]) + Send>;

struct CallbackData {
    callback: Option<MessageCallback>,
    /// Non-fatal warning/debug-warning reporting channel (see
    /// `RtMidiError::Warning`/`DebugWarning`), e.g. for dropped-message
    /// notifications when the polling queue is full.
    error_callback: Option<super::RtMidiErrorCallback>,
    queue: std::collections::VecDeque<(f64, Vec<u8>)>,
    /// Maximum number of messages the polling queue holds. Once full,
    /// incoming messages are silently dropped (not the oldest queued
    /// message) — matching upstream RtMidi's fixed-size `MidiQueue::push`,
    /// which drops the new message and returns `false` when the ring buffer
    /// is full.
    queue_size_limit: usize,
    ignore_sysex: bool,
    ignore_timing: bool,
    ignore_active_sensing: bool,
    /// Timestamp (relative to port opening) of the previously received
    /// message, used to report each message's timestamp as a delta from the
    /// one before it, matching upstream RtMidi semantics.
    start_time: std::time::Instant,
    last_message_time: Option<std::time::Instant>,
}

impl CallbackData {
    fn new() -> Self {
        Self {
            callback: None,
            error_callback: None,
            queue: std::collections::VecDeque::new(),
            queue_size_limit: 100,
            ignore_sysex: true,
            ignore_timing: true,
            ignore_active_sensing: true,
            start_time: std::time::Instant::now(),
            last_message_time: None,
        }
    }

    /// Time in seconds since the previous message (0.0 for the first message
    /// since the port was opened/reset).
    fn next_delta_seconds(&mut self) -> f64 {
        let now = std::time::Instant::now();
        let delta = match self.last_message_time {
            Some(prev) => now.duration_since(prev).as_secs_f64(),
            None => 0.0,
        };
        self.last_message_time = Some(now);
        delta
    }
}

impl CoreMidiInput {
    /// Create a new CoreMIDI input
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let client = Client::new(client_name).map_err(|e| {
            RtMidiError::DriverError(format!("Failed to create MIDI client: {}", e))
        })?;

        let callback_data = Arc::new(Mutex::new(CallbackData::new()));

        Ok(Self {
            client,
            input_port: None,
            connected_source: None,
            virtual_destination: None,
            callback_data,
        })
    }

    /// Build the packet-list handling closure shared by real and virtual
    /// input ports: filters ignored message types, then either invokes the
    /// registered callback or appends to the polling queue.
    fn make_packet_handler(
        callback_data: Arc<Mutex<CallbackData>>,
    ) -> impl FnMut(&coremidi::PacketList) + Send + 'static {
        move |packet_list: &coremidi::PacketList| {
            let mut data = callback_data.lock().unwrap();

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

                let delta = data.next_delta_seconds();
                if let Some(ref mut cb) = data.callback {
                    cb(delta, msg);
                } else if data.queue.len() < data.queue_size_limit {
                    data.queue.push_back((delta, msg.to_vec()));
                } else if let Some(ref mut err_cb) = data.error_callback {
                    // Queue is full: matches upstream RtMidi's
                    // MidiQueue::push behavior of dropping the new message,
                    // but report it as a non-fatal warning if the caller is
                    // listening for one.
                    err_cb(&RtMidiError::Warning(
                        "input queue full, dropping message".to_string(),
                    ));
                }
            }
        }
    }

    /// Open a MIDI input port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let source = Sources
            .into_iter()
            .nth(port_index)
            .ok_or(RtMidiError::InvalidPort(port_index))?;

        let callback_data = Arc::clone(&self.callback_data);
        let input_port = self
            .client
            .input_port(port_name, Self::make_packet_handler(callback_data))
            .map_err(|e| RtMidiError::DriverError(format!("Failed to create input port: {}", e)))?;

        input_port
            .connect_source(&source)
            .map_err(|e| RtMidiError::DriverError(format!("Failed to connect to source: {}", e)))?;

        self.input_port = Some(input_port);
        self.connected_source = Some(source);

        // Reset delta-time tracking when the port is (re-)opened.
        if let Ok(mut data) = self.callback_data.lock() {
            data.start_time = std::time::Instant::now();
            data.last_message_time = None;
        }

        Ok(())
    }

    /// Create a virtual MIDI input port. Other applications can send MIDI to
    /// this port by name; incoming data is routed through the same
    /// filter/callback/queue logic as a real port (see `make_packet_handler`).
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        let callback_data = Arc::clone(&self.callback_data);
        let destination = self
            .client
            .virtual_destination(port_name, Self::make_packet_handler(callback_data))
            .map_err(|e| {
                RtMidiError::DriverError(format!("Failed to create virtual destination: {}", e))
            })?;

        self.virtual_destination = Some(destination);

        if let Ok(mut data) = self.callback_data.lock() {
            data.start_time = std::time::Instant::now();
            data.last_message_time = None;
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
        self.virtual_destination = None;
    }

    /// Rename the underlying CoreMIDI client
    pub fn set_client_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        self.client
            .set_property_string("name", name)
            .map_err(|e| RtMidiError::DriverError(format!("Failed to rename client: {}", e)))
    }

    /// Rename the underlying CoreMIDI port (real input port or virtual
    /// destination, whichever is currently open)
    pub fn set_port_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        if let Some(ref port) = self.input_port {
            return port
                .set_property_string("name", name)
                .map_err(|e| RtMidiError::DriverError(format!("Failed to rename port: {}", e)));
        }
        if let Some(ref dest) = self.virtual_destination {
            return dest
                .set_property_string("name", name)
                .map_err(|e| RtMidiError::DriverError(format!("Failed to rename port: {}", e)));
        }
        Err(RtMidiError::PortNotOpen)
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

    /// Set the maximum number of messages the polling queue holds before
    /// incoming messages are silently dropped.
    pub fn set_queue_size_limit(&mut self, limit: usize) {
        if let Ok(mut data) = self.callback_data.lock() {
            data.queue_size_limit = limit;
        }
    }

    /// Register a callback for non-fatal warnings (see
    /// `RtMidiError::Warning`/`DebugWarning`), such as a dropped message when
    /// the polling queue is full.
    pub fn set_error_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&RtMidiError) + Send + 'static,
    {
        if let Ok(mut data) = self.callback_data.lock() {
            data.error_callback = Some(Box::new(callback));
        }
    }

    /// Remove any registered error callback.
    pub fn cancel_error_callback(&mut self) {
        if let Ok(mut data) = self.callback_data.lock() {
            data.error_callback = None;
        }
    }
}

/// CoreMIDI output handler
pub struct CoreMidiOutput {
    client: Client,
    output_port: Option<OutputPort>,
    destination: Option<Destination>,
    /// Virtual source: this is the correct CoreMIDI primitive for a virtual
    /// *output* port. It exposes our client as a MIDI source that other
    /// applications' input ports can connect to; we "send" by injecting data
    /// via `VirtualSource::received`. (A `VirtualDestination` is the
    /// opposite direction — an endpoint other apps send *to*, with a read
    /// callback of ours — which is the right primitive for `CoreMidiInput`'s
    /// virtual port, not this one.)
    virtual_source: Option<VirtualSource>,
}

impl CoreMidiOutput {
    /// Create a new CoreMIDI output
    pub fn new(client_name: &str) -> Result<Self, RtMidiError> {
        let client = Client::new(client_name).map_err(|e| {
            RtMidiError::DriverError(format!("Failed to create MIDI client: {}", e))
        })?;

        Ok(Self {
            client,
            output_port: None,
            destination: None,
            virtual_source: None,
        })
    }

    /// Open a MIDI output port
    pub fn open_port(&mut self, port_index: usize, port_name: &str) -> Result<(), RtMidiError> {
        let destination = Destinations
            .into_iter()
            .nth(port_index)
            .ok_or(RtMidiError::InvalidPort(port_index))?;

        let output_port = self.client.output_port(port_name).map_err(|e| {
            RtMidiError::DriverError(format!("Failed to create output port: {}", e))
        })?;

        self.output_port = Some(output_port);
        self.destination = Some(destination);

        Ok(())
    }

    /// Create a virtual MIDI output port. Other applications can connect an
    /// input port to this source to receive what we send.
    pub fn open_virtual_port(&mut self, port_name: &str) -> Result<(), RtMidiError> {
        let source = self.client.virtual_source(port_name).map_err(|e| {
            RtMidiError::DriverError(format!("Failed to create virtual source: {}", e))
        })?;

        self.virtual_source = Some(source);

        Ok(())
    }

    /// Close the currently open port
    pub fn close_port(&mut self) {
        self.output_port = None;
        self.destination = None;
        self.virtual_source = None;
    }

    /// Rename the underlying CoreMIDI client
    pub fn set_client_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        self.client
            .set_property_string("name", name)
            .map_err(|e| RtMidiError::DriverError(format!("Failed to rename client: {}", e)))
    }

    /// Rename the underlying CoreMIDI port (real output port or virtual
    /// source, whichever is currently open)
    pub fn set_port_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        if let Some(ref port) = self.output_port {
            return port
                .set_property_string("name", name)
                .map_err(|e| RtMidiError::DriverError(format!("Failed to rename port: {}", e)));
        }
        if let Some(ref source) = self.virtual_source {
            return source
                .set_property_string("name", name)
                .map_err(|e| RtMidiError::DriverError(format!("Failed to rename port: {}", e)));
        }
        Err(RtMidiError::PortNotOpen)
    }

    /// Send a MIDI message
    pub fn send_message(&mut self, message: &[u8]) -> Result<(), RtMidiError> {
        let packet_buffer = PacketBuffer::new(0, message);

        if let (Some(port), Some(dest)) = (&self.output_port, &self.destination) {
            port.send(dest, &packet_buffer)
                .map_err(|e| RtMidiError::DriverError(format!("Failed to send message: {}", e)))?;
        } else if let Some(ref source) = self.virtual_source {
            source.received(&packet_buffer).map_err(|e| {
                RtMidiError::DriverError(format!("Failed to send from virtual source: {}", e))
            })?;
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
        println!(
            "Found {} input ports and {} output ports",
            inputs.len(),
            outputs.len()
        );
    }
}
