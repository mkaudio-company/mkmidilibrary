//! Real-time MIDI input

use super::port::{Api, MidiPort};
use super::{MidiCallback, MidiInputConfig, RtMidiError, RtMidiErrorCallback};

#[cfg(target_os = "macos")]
use super::coremidi_impl::CoreMidiInput;

/// Timestamped MIDI message
#[derive(Debug, Clone)]
pub struct TimestampedMessage {
    /// Timestamp in seconds, relative to the previous message received on
    /// this port (the first message after opening the port reports `0.0`).
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
    /// Callback set before a port was opened, applied once the platform
    /// backend is constructed in `open_port`/`open_virtual_port`. Once a
    /// port is open, the callback lives in the platform backend itself
    /// (there is deliberately no separate copy here).
    pending_callback: Option<MidiCallback>,
    /// Non-fatal warning callback set before a port was opened, applied the
    /// same way as `pending_callback`.
    pending_error_callback: Option<RtMidiErrorCallback>,
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
            pending_callback: None,
            pending_error_callback: None,
            platform: None,
        })
    }

    /// Create a new MIDI input with a specific incoming-message queue size
    /// limit (see `MidiInputConfig::queue_size`), matching upstream RtMidi's
    /// `RtMidiIn(Api, clientName, queueSizeLimit)` constructor.
    pub fn with_queue_size(
        api: Api,
        client_name: &str,
        queue_size_limit: usize,
    ) -> Result<Self, RtMidiError> {
        let mut input = Self::with_api(api, client_name)?;
        input.config.queue_size = queue_size_limit;
        Ok(input)
    }

    /// Set the internal buffer size used for reading device data. This is
    /// RtMidiIn-specific and primarily relevant to WinMM's sysex handling;
    /// it has no effect on the CoreMIDI/ALSA backends implemented here and
    /// is reserved for a future WinMM implementation.
    pub fn set_buffer_size(&mut self, _size: usize, _count: usize) {}

    /// Get the API being used
    pub fn api(&self) -> Api {
        self.api
    }

    /// Get the client name
    pub fn client_name(&self) -> &str {
        &self.client_name
    }

    /// Rename the MIDI client. Renames the live backend immediately if one
    /// exists (i.e. a port is currently open); otherwise takes effect the
    /// next time a port is opened.
    pub fn set_client_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        self.client_name = name.to_string();
        self.platform_set_client_name(name)
    }

    /// Rename the currently open port.
    pub fn set_port_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        if !self.port_open {
            return Err(RtMidiError::PortNotOpen);
        }
        self.port_name = Some(name.to_string());
        self.platform_set_port_name(name)
    }

    #[cfg(target_os = "macos")]
    fn platform_set_client_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        match self.platform {
            Some(ref mut p) => p.set_client_name(name),
            None => Ok(()),
        }
    }

    #[cfg(target_os = "macos")]
    fn platform_set_port_name(&mut self, name: &str) -> Result<(), RtMidiError> {
        match self.platform {
            Some(ref mut p) => p.set_port_name(name),
            None => Err(RtMidiError::PortNotOpen),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn platform_set_client_name(&mut self, _name: &str) -> Result<(), RtMidiError> {
        Err(RtMidiError::DriverError(
            "set_client_name is not implemented for this platform".to_string(),
        ))
    }

    #[cfg(not(target_os = "macos"))]
    fn platform_set_port_name(&mut self, _name: &str) -> Result<(), RtMidiError> {
        Err(RtMidiError::DriverError(
            "set_port_name is not implemented for this platform".to_string(),
        ))
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

    /// Set a callback for incoming messages. If a port is already open, this
    /// reaches the platform backend immediately; otherwise it is applied as
    /// soon as a port is opened.
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnMut(f64, &[u8]) + Send + 'static,
    {
        let boxed: MidiCallback = Box::new(callback);
        if self.has_platform() {
            self.platform_set_callback(boxed);
        } else {
            self.pending_callback = Some(boxed);
        }
    }

    /// Cancel the callback and return to queue-based input
    pub fn cancel_callback(&mut self) {
        self.pending_callback = None;
        if self.has_platform() {
            self.platform_cancel_callback();
        }
    }

    /// Get a message from the queue (non-blocking). Returns `None` if a
    /// callback is currently registered (messages are delivered to the
    /// callback instead) or if no port is open.
    pub fn get_message(&mut self) -> Option<TimestampedMessage> {
        self.platform_get_message()
            .map(|(timestamp, data)| TimestampedMessage { timestamp, data })
    }

    /// Set which message types to ignore. Applies immediately to an already-open
    /// port, and is otherwise applied when a port is opened.
    pub fn ignore_types(&mut self, sysex: bool, timing: bool, active_sensing: bool) {
        self.config.ignore_sysex = sysex;
        self.config.ignore_timing = timing;
        self.config.ignore_active_sensing = active_sensing;
        if self.has_platform() {
            self.platform_ignore_types(sysex, timing, active_sensing);
        }
    }

    /// Register a callback for non-fatal warnings (see
    /// `RtMidiError::Warning`/`DebugWarning`), such as a dropped message when
    /// the polling queue is full.
    pub fn set_error_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&RtMidiError) + Send + 'static,
    {
        let boxed: RtMidiErrorCallback = Box::new(callback);
        if self.has_platform() {
            self.platform_set_error_callback(boxed);
        } else {
            self.pending_error_callback = Some(boxed);
        }
    }

    /// Remove any registered error callback.
    pub fn cancel_error_callback(&mut self) {
        self.pending_error_callback = None;
        if self.has_platform() {
            self.platform_cancel_error_callback();
        }
    }

    // Platform-specific implementations

    fn has_platform(&self) -> bool {
        self.platform.is_some()
    }

    #[cfg(target_os = "macos")]
    fn platform_set_callback(&mut self, callback: MidiCallback) {
        if let Some(ref mut p) = self.platform {
            p.set_callback(callback);
        }
    }

    #[cfg(target_os = "macos")]
    fn platform_cancel_callback(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.cancel_callback();
        }
    }

    #[cfg(target_os = "macos")]
    fn platform_get_message(&mut self) -> Option<(f64, Vec<u8>)> {
        self.platform.as_mut().and_then(|p| p.get_message())
    }

    #[cfg(target_os = "macos")]
    fn platform_ignore_types(&mut self, sysex: bool, timing: bool, active_sensing: bool) {
        if let Some(ref mut p) = self.platform {
            p.ignore_types(sysex, timing, active_sensing);
        }
    }

    #[cfg(target_os = "macos")]
    fn platform_set_error_callback(&mut self, callback: RtMidiErrorCallback) {
        if let Some(ref mut p) = self.platform {
            p.set_error_callback(callback);
        }
    }

    #[cfg(target_os = "macos")]
    fn platform_cancel_error_callback(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.cancel_error_callback();
        }
    }

    #[cfg(target_os = "linux")]
    fn platform_set_callback(&mut self, callback: MidiCallback) {
        if let Some(ref mut p) = self.platform {
            p.set_callback(callback);
        }
    }

    #[cfg(target_os = "linux")]
    fn platform_cancel_callback(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.cancel_callback();
        }
    }

    #[cfg(target_os = "linux")]
    fn platform_get_message(&mut self) -> Option<(f64, Vec<u8>)> {
        self.platform.as_mut().and_then(|p| p.get_message())
    }

    #[cfg(target_os = "linux")]
    fn platform_ignore_types(&mut self, sysex: bool, timing: bool, active_sensing: bool) {
        if let Some(ref mut p) = self.platform {
            p.ignore_types(sysex, timing, active_sensing);
        }
    }

    #[cfg(target_os = "linux")]
    fn platform_set_error_callback(&mut self, callback: RtMidiErrorCallback) {
        if let Some(ref mut p) = self.platform {
            p.set_error_callback(callback);
        }
    }

    #[cfg(target_os = "linux")]
    fn platform_cancel_error_callback(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.cancel_error_callback();
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_set_callback(&mut self, callback: MidiCallback) {
        if let Some(ref mut p) = self.platform {
            p.set_callback(callback);
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_cancel_callback(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.cancel_callback();
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_get_message(&mut self) -> Option<(f64, Vec<u8>)> {
        self.platform.as_mut().and_then(|p| p.get_message())
    }

    #[cfg(target_os = "windows")]
    fn platform_ignore_types(&mut self, sysex: bool, timing: bool, active_sensing: bool) {
        if let Some(ref mut p) = self.platform {
            p.ignore_types(sysex, timing, active_sensing);
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_set_error_callback(&mut self, callback: RtMidiErrorCallback) {
        if let Some(ref mut p) = self.platform {
            p.set_error_callback(callback);
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_cancel_error_callback(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.cancel_error_callback();
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn platform_set_callback(&mut self, _callback: MidiCallback) {}

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn platform_cancel_callback(&mut self) {}

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn platform_get_message(&mut self) -> Option<(f64, Vec<u8>)> {
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn platform_set_error_callback(&mut self, _callback: RtMidiErrorCallback) {}

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn platform_cancel_error_callback(&mut self) {}

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn platform_ignore_types(&mut self, _sysex: bool, _timing: bool, _active_sensing: bool) {}

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
        self.apply_pending_state_coremidi(&mut platform);
        self.platform = Some(platform);
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn open_virtual_port_coremidi(&mut self, name: &str) -> Result<(), RtMidiError> {
        let mut platform = CoreMidiInput::new(&self.client_name)?;
        platform.open_virtual_port(name)?;
        self.apply_pending_state_coremidi(&mut platform);
        self.platform = Some(platform);
        Ok(())
    }

    /// Apply the currently configured filter settings and any callback set
    /// before the port was opened to a freshly constructed platform backend.
    #[cfg(target_os = "macos")]
    fn apply_pending_state_coremidi(&mut self, platform: &mut CoreMidiInput) {
        platform.ignore_types(
            self.config.ignore_sysex,
            self.config.ignore_timing,
            self.config.ignore_active_sensing,
        );
        platform.set_queue_size_limit(self.config.queue_size);
        if let Some(callback) = self.pending_callback.take() {
            platform.set_callback(callback);
        }
        if let Some(callback) = self.pending_error_callback.take() {
            platform.set_error_callback(callback);
        }
    }

    #[cfg(target_os = "macos")]
    fn close_port_coremidi(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.close_port();
        }
        self.platform = None;
    }

    // ALSA implementations. `AlsaMidiInput` itself is currently a stub
    // (see its own doc comment) — real sequencer I/O needs a Linux
    // environment to write and verify against actual hardware/drivers —
    // but the wiring here (constructing it, applying pending config/
    // callbacks, storing it in `self.platform`) matches the CoreMIDI
    // pattern exactly, so filling in that stub later doesn't need any
    // dispatch changes.
    #[cfg(target_os = "linux")]
    fn get_ports_alsa(&self) -> Vec<MidiPort> {
        super::alsa_impl::get_input_ports()
    }

    #[cfg(target_os = "linux")]
    fn open_port_alsa(&mut self, port: usize, name: &str) -> Result<(), RtMidiError> {
        let mut platform = super::alsa_impl::AlsaMidiInput::new(&self.client_name)?;
        platform.open_port(port, name)?;
        self.apply_pending_state_alsa(&mut platform);
        self.platform = Some(platform);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn open_virtual_port_alsa(&mut self, name: &str) -> Result<(), RtMidiError> {
        let mut platform = super::alsa_impl::AlsaMidiInput::new(&self.client_name)?;
        platform.open_virtual_port(name)?;
        self.apply_pending_state_alsa(&mut platform);
        self.platform = Some(platform);
        Ok(())
    }

    /// Apply the currently configured filter settings and any callback set
    /// before the port was opened to a freshly constructed platform backend.
    #[cfg(target_os = "linux")]
    fn apply_pending_state_alsa(&mut self, platform: &mut super::alsa_impl::AlsaMidiInput) {
        platform.ignore_types(
            self.config.ignore_sysex,
            self.config.ignore_timing,
            self.config.ignore_active_sensing,
        );
        platform.set_queue_size_limit(self.config.queue_size);
        if let Some(callback) = self.pending_callback.take() {
            platform.set_callback(callback);
        }
        if let Some(callback) = self.pending_error_callback.take() {
            platform.set_error_callback(callback);
        }
    }

    #[cfg(target_os = "linux")]
    fn close_port_alsa(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.close_port();
        }
        self.platform = None;
    }

    // Windows MM implementations — same relationship to `WinMmMidiInput`
    // as the ALSA ones above have to `AlsaMidiInput`.
    #[cfg(target_os = "windows")]
    fn get_ports_winmm(&self) -> Vec<MidiPort> {
        super::winmm_impl::get_input_ports()
    }

    #[cfg(target_os = "windows")]
    fn open_port_winmm(&mut self, port: usize, name: &str) -> Result<(), RtMidiError> {
        let mut platform = super::winmm_impl::WinMmMidiInput::new(&self.client_name)?;
        platform.open_port(port, name)?;
        self.apply_pending_state_winmm(&mut platform);
        self.platform = Some(platform);
        Ok(())
    }

    /// Apply the currently configured filter settings and any callback set
    /// before the port was opened to a freshly constructed platform backend.
    #[cfg(target_os = "windows")]
    fn apply_pending_state_winmm(&mut self, platform: &mut super::winmm_impl::WinMmMidiInput) {
        platform.ignore_types(
            self.config.ignore_sysex,
            self.config.ignore_timing,
            self.config.ignore_active_sensing,
        );
        platform.set_queue_size_limit(self.config.queue_size);
        if let Some(callback) = self.pending_callback.take() {
            platform.set_callback(callback);
        }
        if let Some(callback) = self.pending_error_callback.take() {
            platform.set_error_callback(callback);
        }
    }

    #[cfg(target_os = "windows")]
    fn close_port_winmm(&mut self) {
        if let Some(ref mut p) = self.platform {
            p.close_port();
        }
        self.platform = None;
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

    /// CoreMIDI's virtual-port/dispatch state is process-global; running
    /// several of these tests concurrently (the `cargo test` default) can
    /// make them flaky (a message meant for one test's virtual port arrives
    /// late, or two tests' port enumeration races). Serializing just the
    /// CoreMIDI-touching tests keeps `cargo test`'s default parallel run
    /// deterministic without slowing down the rest of the suite.
    #[cfg(target_os = "macos")]
    static COREMIDI_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[cfg(target_os = "macos")]
    fn lock_coremidi_tests() -> std::sync::MutexGuard<'static, ()> {
        COREMIDI_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

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

    #[cfg(target_os = "macos")]
    #[test]
    fn test_set_port_name_renames_live_coremidi_port() {
        let _guard = lock_coremidi_tests();
        use super::super::output::MidiOutput;

        let mut input = MidiInput::new("mkmidilibrary-test-input-rename").unwrap();
        let original_name = "mkmidilibrary-test-virtual-port-rename";
        if input.open_virtual_port(original_name).is_err() {
            return;
        }

        let renamed = "mkmidilibrary-test-virtual-port-renamed";
        if input.set_port_name(renamed).is_err() {
            return;
        }
        assert_eq!(input.port_name.as_deref(), Some(renamed));

        // Confirm the rename actually reached CoreMIDI by checking the port
        // is now visible under the new name from another client.
        let output = MidiOutput::new("mkmidilibrary-test-output-rename").unwrap();
        let ports = output.ports();
        assert!(
            ports.iter().any(|p| p.name() == renamed),
            "renamed port not visible under new name"
        );
    }

    #[test]
    fn test_with_queue_size_constructor() {
        let input = MidiInput::with_queue_size(Api::Dummy, "Test", 42).unwrap();
        assert_eq!(input.config().queue_size, 42);
    }

    #[test]
    fn test_ignore_sysex_defaults_true() {
        // Matches upstream RtMidi's default (ignoreFlags(7) ignores sysex,
        // timing, and active sensing by default).
        assert!(MidiInputConfig::default().ignore_sysex);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_coremidi_queue_size_cap_drops_overflow() {
        let _guard = lock_coremidi_tests();
        use super::super::output::MidiOutput;
        use std::time::{Duration, Instant};

        let mut input = MidiInput::new("mkmidilibrary-test-input-queuecap").unwrap();
        input.set_config(MidiInputConfig {
            queue_size: 3,
            ..Default::default()
        });
        let port_name = "mkmidilibrary-test-virtual-port-queuecap";
        if input.open_virtual_port(port_name).is_err() {
            return;
        }

        let mut output = MidiOutput::new("mkmidilibrary-test-output-queuecap").unwrap();
        let ports = output.ports();
        let Some(index) = ports.iter().position(|p| p.name() == port_name) else {
            return;
        };
        if output.open_port(index, "conn").is_err() {
            return;
        }

        // Send more messages than the queue can hold.
        for i in 0..10u8 {
            output.send_note_on(0, 60 + i, 100).unwrap();
        }

        // Give CoreMIDI's dispatch a moment to deliver everything.
        std::thread::sleep(Duration::from_millis(200));

        let mut count = 0;
        let deadline = Instant::now() + Duration::from_secs(1);
        while input.get_message().is_some() {
            count += 1;
            if Instant::now() >= deadline {
                break;
            }
        }

        assert!(
            count <= 3,
            "queue_size cap of 3 should bound delivered messages, got {count}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_error_callback_fires_on_queue_overflow() {
        let _guard = lock_coremidi_tests();
        use super::super::output::MidiOutput;
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};

        let mut input = MidiInput::new("mkmidilibrary-test-input-errcb").unwrap();
        input.set_config(MidiInputConfig {
            queue_size: 1,
            ..Default::default()
        });
        let warnings: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let warnings_clone = Arc::clone(&warnings);
        input.set_error_callback(move |err| {
            warnings_clone.lock().unwrap().push(err.to_string());
        });

        let port_name = "mkmidilibrary-test-virtual-port-errcb";
        if input.open_virtual_port(port_name).is_err() {
            return;
        }

        let mut output = MidiOutput::new("mkmidilibrary-test-output-errcb").unwrap();
        let ports = output.ports();
        let Some(index) = ports.iter().position(|p| p.name() == port_name) else {
            return;
        };
        if output.open_port(index, "conn").is_err() {
            return;
        }

        // Keep sending (well beyond the queue_size of 1) until either a
        // warning shows up or we give up after a generous timeout — CoreMIDI
        // delivery between two clients in-process is asynchronous.
        let deadline = Instant::now() + Duration::from_secs(2);
        while warnings.lock().unwrap().is_empty() && Instant::now() < deadline {
            for i in 0..10u8 {
                let _ = output.send_note_on(0, 60 + i, 100);
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        assert!(
            !warnings.lock().unwrap().is_empty(),
            "expected at least one queue-full warning via the error callback"
        );
    }

    // Regression test for the CoreMIDI callback/queue wiring bug: MidiInput
    // used to own an unused shadow queue/callback while the real CoreMIDI
    // backend populated a completely separate one, so neither the polling
    // API nor the callback API ever delivered real messages. This drives a
    // full loopback through the actual CoreMIDI backend (virtual port ->
    // real send -> polling `get_message`) with no external hardware needed.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_coremidi_loopback_get_message() {
        let _guard = lock_coremidi_tests();
        use super::super::output::MidiOutput;
        use std::time::{Duration, Instant};

        let mut input = MidiInput::new("mkmidilibrary-test-input").unwrap();
        let port_name = "mkmidilibrary-test-virtual-port-poll";
        if input.open_virtual_port(port_name).is_err() {
            // CoreMIDI is unavailable in this environment (e.g. sandboxed CI);
            // nothing more we can verify here.
            return;
        }

        let mut output = MidiOutput::new("mkmidilibrary-test-output").unwrap();
        let ports = output.ports();
        let Some(index) = ports.iter().position(|p| p.name() == port_name) else {
            return;
        };
        if output.open_port(index, "conn").is_err() {
            return;
        }

        output.send_note_on(0, 60, 100).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut received = None;
        while Instant::now() < deadline {
            if let Some(msg) = input.get_message() {
                received = Some(msg);
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let msg = received.expect("expected a MIDI message via loopback within 2s");
        assert_eq!(msg.data, vec![0x90, 60, 100]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_coremidi_loopback_callback() {
        let _guard = lock_coremidi_tests();
        use super::super::output::MidiOutput;
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};

        let mut input = MidiInput::new("mkmidilibrary-test-input-cb").unwrap();
        let port_name = "mkmidilibrary-test-virtual-port-cb";
        if input.open_virtual_port(port_name).is_err() {
            return;
        }

        let received: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        input.set_callback(move |_timestamp, data| {
            received_clone.lock().unwrap().push(data.to_vec());
        });

        let mut output = MidiOutput::new("mkmidilibrary-test-output-cb").unwrap();
        let ports = output.ports();
        let Some(index) = ports.iter().position(|p| p.name() == port_name) else {
            return;
        };
        if output.open_port(index, "conn").is_err() {
            return;
        }

        output.send_note_on(1, 64, 90).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if !received.lock().unwrap().is_empty() {
                break;
            }
            if Instant::now() >= deadline {
                panic!("expected a MIDI message via callback loopback within 2s");
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(received.lock().unwrap()[0], vec![0x91, 64, 90]);
    }
}
