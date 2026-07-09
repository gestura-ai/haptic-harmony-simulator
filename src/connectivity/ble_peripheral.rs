//! BLE peripheral implementation for Haptic Harmony Ring simulation
//! Creates a real BLE peripheral that gestura.app can discover and connect to

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::task::JoinHandle;

use crate::connectivity::native_ble_backend::NativeBleCommand;
#[cfg(feature = "native-ble")]
use crate::connectivity::native_ble_backend::start_native_ble_runtime;
use crate::connectivity::{ConnectionConfig, ConnectionState};
use crate::emulator::{GestureEvent, HapticEvent};
use crate::protocol::{
    AckPayload, AckStatus, BatterySnapshot, DeviceStateSnapshot, HapticCommandPayload,
    ProtocolEnvelope, SHARED_PROTOCOL_VERSION, SemanticGestureEvent, SemanticHapticPattern,
    SimulatorCommand, SimulatorEvent,
};
use crate::transport_adapters::BleProtocolAdapter;
use crate::trust::{DegradedMode, PrivilegedAction, TrustPolicy, TrustState};

/// Haptic Harmony Ring BLE service UUIDs — FINAL joint allocation (user
/// decision 2026-07-02, firmware-minted base). Canonical source:
/// gestura-app SDK `gestura-core-ring::protocol::ring_uuids`; keep in sync.
pub mod ring_uuids {
    /// Main ring service UUID
    pub const HAPTIC_SERVICE_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9bc";
    /// Haptic command characteristic UUID (write, trust-gated)
    pub const HAPTIC_COMMAND_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9bd";
    /// Gesture event characteristic UUID (notify)
    pub const GESTURE_EVENT_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9be";
    /// Battery level characteristic UUID (read + notify)
    pub const BATTERY_LEVEL_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9bf";
    /// OTA update characteristic UUID (write + indicate)
    pub const OTA_UPDATE_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9c0";
    /// State snapshot characteristic UUID (read + notify)
    pub const STATE_SNAPSHOT_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9c1";
    /// Config characteristic UUID (write, encrypted/trust-gated)
    pub const CONFIG_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9c2";
    /// Opt-in raw sensor stream UUID (notify; subscription trust-gated)
    pub const RAW_SENSOR_STREAM_UUID: &str = "e3b742d4-51c9-4f0e-9d26-7a48c1f0b9c3";
}

/// BLE peripheral events
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BlePeripheralEvent {
    /// Client connected
    ClientConnected(String),
    /// Client disconnected
    ClientDisconnected(String),
    /// Haptic command received from client
    HapticCommandReceived(HapticCommand),
    /// Characteristic subscribed
    CharacteristicSubscribed(String),
    /// Characteristic unsubscribed
    CharacteristicUnsubscribed(String),
}

/// Haptic command from gestura.app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticCommand {
    pub pattern: String,
    pub intensity: f32,
    pub duration_ms: u32,
}

impl HapticCommand {
    /// Converts this legacy haptic command into the shared protocol envelope.
    pub fn to_protocol_envelope(&self, sequence: u64) -> ProtocolEnvelope<SimulatorCommand> {
        ProtocolEnvelope::command_now(
            sequence,
            SimulatorCommand::Haptic(HapticCommandPayload::from_legacy_parts(
                &self.pattern,
                self.intensity,
                self.duration_ms,
            )),
        )
    }

    /// Attempts to parse a shared protocol command envelope from BLE bytes.
    pub fn try_from_protocol_bytes(data: &[u8]) -> Result<Self> {
        let envelope: ProtocolEnvelope<SimulatorCommand> = serde_json::from_slice(data)?;

        match envelope.payload {
            SimulatorCommand::Haptic(command) => Ok(Self::from(command)),
        }
    }
}

impl From<HapticCommandPayload> for HapticCommand {
    fn from(value: HapticCommandPayload) -> Self {
        // Preset intensities/durations are placeholder feel values — actual
        // waveform feel is an open cross-lane tuning item.
        match value.pattern {
            SemanticHapticPattern::Confirm => Self {
                pattern: "confirm".to_string(),
                intensity: 1.0,
                duration_ms: 250,
            },
            SemanticHapticPattern::Error => Self {
                pattern: "error".to_string(),
                intensity: 1.0,
                duration_ms: 300,
            },
            SemanticHapticPattern::Tick => Self {
                pattern: "tick".to_string(),
                intensity: 1.0,
                duration_ms: 100,
            },
            SemanticHapticPattern::DoubleTick => Self {
                pattern: "double_tick".to_string(),
                intensity: 1.0,
                duration_ms: 250,
            },
            SemanticHapticPattern::Waveform {
                ref data,
                sample_rate_hz,
                intensity,
            } => Self {
                pattern: "waveform".to_string(),
                intensity,
                duration_ms: crate::protocol::waveform_playback_duration(data, sample_rate_hz)
                    .as_millis() as u32,
            },
            SemanticHapticPattern::Custom {
                intensity,
                duration_ms,
            } => Self {
                pattern: "custom".to_string(),
                intensity,
                duration_ms: duration_ms as u32,
            },
        }
    }
}

/// BLE characteristic data
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleCharacteristic {
    pub uuid: String,
    pub properties: BleCharacteristicProperties,
    pub value: Arc<RwLock<Vec<u8>>>,
    pub subscribers: Arc<RwLock<Vec<String>>>,
    pub live_notifiers: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>,
}

/// BLE characteristic properties
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleCharacteristicProperties {
    pub read: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
}

/// Gesture data for BLE transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleGestureData {
    pub gesture_type: String,
    pub timestamp: u64,
    pub confidence: f32,
    pub data: Vec<u8>,
}

impl BleGestureData {
    /// Builds the BLE gesture payload while embedding the shared semantic protocol event.
    #[allow(dead_code)]
    pub fn from_gesture_event(gesture: &GestureEvent, sequence: u64) -> Result<Self> {
        let semantic_event = SemanticGestureEvent::from_runtime_event(gesture);
        let envelope = ProtocolEnvelope::event(
            sequence,
            semantic_event.timestamp_ms,
            SimulatorEvent::Gesture(semantic_event.clone()),
        );

        Ok(Self {
            gesture_type: format!("{:?}", gesture.gesture_type),
            timestamp: semantic_event.timestamp_ms,
            confidence: semantic_event.confidence,
            data: serde_json::to_vec(&envelope)?,
        })
    }
}

/// Battery level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleBatteryData {
    pub level: u8,
    pub is_charging: bool,
    pub voltage: f32,
    pub temperature: f32,
    pub health: String,
    pub time_remaining: Option<u32>, // minutes
}

impl BleBatteryData {
    /// Converts this BLE-specific battery shape into the shared semantic snapshot.
    #[allow(dead_code)]
    pub fn to_protocol_snapshot(&self) -> BatterySnapshot {
        BatterySnapshot {
            level_percent: self.level,
            is_charging: self.is_charging,
            voltage: self.voltage,
            temperature_celsius: self.temperature,
            health: self.health.clone(),
            time_remaining_minutes: self.time_remaining,
        }
    }
}

/// Battery simulation state
#[derive(Debug, Clone)]
pub struct BatterySimulator {
    pub level: u8,
    pub is_charging: bool,
    pub charge_rate: f32, // % per minute
    pub drain_rate: f32,  // % per minute
    pub last_update: tokio::time::Instant,
    pub temperature: f32,
    pub cycle_count: u32,
}

/// Device info data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleDeviceInfo {
    pub firmware_version: String,
    pub hardware_version: String,
    pub manufacturer: String,
    pub model: String,
}

/// BLE peripheral for Haptic Harmony Ring simulation
#[allow(dead_code)]
pub struct BlePeripheral {
    config: ConnectionConfig,
    state: Arc<RwLock<ConnectionState>>,
    event_tx: broadcast::Sender<BlePeripheralEvent>,
    gesture_rx: Option<mpsc::UnboundedReceiver<GestureEvent>>,
    haptic_tx: Option<mpsc::UnboundedSender<HapticEvent>>,
    battery_level: Arc<RwLock<u8>>,
    firmware_version: String,
    device_name: String,
    connected_clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    // BLE Characteristics
    characteristics: Arc<RwLock<HashMap<String, BleCharacteristic>>>,
    device_info: Arc<RwLock<BleDeviceInfo>>,
    battery_simulator: Arc<RwLock<BatterySimulator>>,
    safety_policy: Arc<RwLock<TrustPolicy>>,
    runtime_command_tx: Arc<RwLock<Option<mpsc::UnboundedSender<NativeBleCommand>>>>,
    runtime_task: Option<JoinHandle<()>>,
}

/// Connected client information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientConnection {
    pub client_id: String,
    pub connected_at: std::time::Instant,
    pub subscribed_characteristics: Vec<String>,
}

#[allow(dead_code)]
impl BlePeripheral {
    /// Create new BLE peripheral
    pub fn new(config: ConnectionConfig) -> Result<Self> {
        let (event_tx, _) = broadcast::channel(100);

        // Initialize device info
        let device_info = BleDeviceInfo {
            firmware_version: "1.0.0-sim".to_string(),
            hardware_version: "1.0.0".to_string(),
            manufacturer: "Gestura AI".to_string(),
            model: "Haptic Harmony Ring Simulator".to_string(),
        };

        // Initialize battery simulator
        let battery_simulator = BatterySimulator {
            level: 85,
            is_charging: false,
            charge_rate: 15.0, // 15% per minute when charging
            drain_rate: 0.5,   // 0.5% per minute normal drain
            last_update: tokio::time::Instant::now(),
            temperature: 25.0, // 25°C
            cycle_count: 42,   // Simulated charge cycles
        };

        // Initialize characteristics
        let mut characteristics = HashMap::new();

        // Gesture Event Characteristic (notify)
        characteristics.insert(
            ring_uuids::GESTURE_EVENT_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::GESTURE_EVENT_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: false,
                    write: false,
                    notify: true,
                    indicate: false,
                },
                value: Arc::new(RwLock::new(Vec::new())),
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        // Haptic Command Characteristic (write)
        characteristics.insert(
            ring_uuids::HAPTIC_COMMAND_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::HAPTIC_COMMAND_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: false,
                    write: true,
                    notify: false,
                    indicate: false,
                },
                value: Arc::new(RwLock::new(Vec::new())),
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        // Battery Level Characteristic (read + notify)
        characteristics.insert(
            ring_uuids::BATTERY_LEVEL_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::BATTERY_LEVEL_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: true,
                    write: false,
                    notify: true,
                    indicate: false,
                },
                value: Arc::new(RwLock::new(vec![85])), // 85% battery
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        // State Snapshot Characteristic (read + notify)
        characteristics.insert(
            ring_uuids::STATE_SNAPSHOT_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::STATE_SNAPSHOT_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: true,
                    write: false,
                    notify: true,
                    indicate: false,
                },
                value: Arc::new(RwLock::new(Vec::new())),
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        // OTA Update Characteristic (write)
        characteristics.insert(
            ring_uuids::OTA_UPDATE_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::OTA_UPDATE_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: false,
                    write: true,
                    notify: false,
                    indicate: true,
                },
                value: Arc::new(RwLock::new(Vec::new())),
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        // Config Characteristic (read+write; both trust-gated — link-layer
        // encryption is enforced by the real ring's BLE stack, the simulator
        // enforces the trust gate at read/write time).
        // READ support: readable-C2, RATIFIED 2026-07-08 — hosts
        // read-modify-write instead of clobbering config with defaults.
        // The simulator gates at Enrolled (reference-stricter); the wire
        // contract's device guarantee is Bonded (see PROTOCOL.md).
        // Value seeded with the default config
        // [sensitivity, raw-stream, gesture-mask, HID=on].
        characteristics.insert(
            ring_uuids::CONFIG_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::CONFIG_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: true,
                    write: true,
                    notify: false,
                    indicate: false,
                },
                value: Arc::new(RwLock::new(vec![0x80, 0x00, 0xFF, 0x01])),
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        // Raw Sensor Stream Characteristic (notify; opt-in via config AND
        // subscription trust-gated at Bonded or better — raw IMU data is the
        // most privacy-sensitive channel on the device)
        characteristics.insert(
            ring_uuids::RAW_SENSOR_STREAM_UUID.to_string(),
            BleCharacteristic {
                uuid: ring_uuids::RAW_SENSOR_STREAM_UUID.to_string(),
                properties: BleCharacteristicProperties {
                    read: false,
                    write: false,
                    notify: true,
                    indicate: false,
                },
                value: Arc::new(RwLock::new(Vec::new())),
                subscribers: Arc::new(RwLock::new(Vec::new())),
                live_notifiers: Arc::new(RwLock::new(HashMap::new())),
            },
        );

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            event_tx,
            gesture_rx: None,
            haptic_tx: None,
            battery_level: Arc::new(RwLock::new(85)), // Start with 85% battery
            firmware_version: "1.0.0-sim".to_string(),
            device_name: "Haptic Harmony Ring Simulator".to_string(),
            connected_clients: Arc::new(RwLock::new(HashMap::new())),
            characteristics: Arc::new(RwLock::new(characteristics)),
            device_info: Arc::new(RwLock::new(device_info)),
            battery_simulator: Arc::new(RwLock::new(battery_simulator)),
            safety_policy: Arc::new(RwLock::new(TrustPolicy::default())),
            runtime_command_tx: Arc::new(RwLock::new(None)),
            runtime_task: None,
        })
    }

    /// Set gesture event receiver
    pub fn set_gesture_receiver(&mut self, rx: mpsc::UnboundedReceiver<GestureEvent>) {
        self.gesture_rx = Some(rx);
    }

    /// Set haptic event sender
    #[allow(dead_code)]
    pub fn set_haptic_sender(&mut self, tx: mpsc::UnboundedSender<HapticEvent>) {
        self.haptic_tx = Some(tx);
    }

    /// Get event receiver for peripheral events
    pub fn subscribe_events(&self) -> broadcast::Receiver<BlePeripheralEvent> {
        self.event_tx.subscribe()
    }

    /// Start advertising as BLE peripheral
    pub async fn start_advertising(&mut self) -> Result<()> {
        tracing::info!(
            "Starting BLE peripheral advertising as '{}'",
            self.device_name
        );

        // Update state to advertising
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connecting;
        }

        // Start the advertising process
        self.setup_ble_services().await?;
        self.update_battery_characteristic().await?;
        self.start_advertising_loop().await?;

        // Start battery simulation
        self.start_battery_simulation().await?;

        Ok(())
    }

    /// Stop advertising and disconnect all clients
    pub async fn stop_advertising(&mut self) -> Result<()> {
        tracing::info!("Stopping BLE peripheral advertising");

        if let Some(command_tx) = self.runtime_command_tx.write().await.take() {
            let _ = command_tx.send(NativeBleCommand::Stop);
        }
        if let Some(runtime_task) = self.runtime_task.take() {
            let _ = runtime_task.await;
        }

        // Disconnect all clients
        {
            let mut clients = self.connected_clients.write().await;
            for (client_id, _) in clients.drain() {
                let _ = self
                    .event_tx
                    .send(BlePeripheralEvent::ClientDisconnected(client_id));
            }
        }

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Disconnected;
        }

        Ok(())
    }

    /// Send gesture notification to connected clients
    pub async fn notify_gesture(&self, gesture: &GestureEvent) -> Result<()> {
        let semantic_event = SemanticGestureEvent::from_runtime_event(gesture);
        let envelope = ProtocolEnvelope::event(
            0,
            semantic_event.timestamp_ms,
            SimulatorEvent::Gesture(semantic_event),
        );
        let frame = BleProtocolAdapter.project_event(&envelope)?;
        self.notify_characteristic(frame.characteristic_uuid, frame.payload)
            .await?;

        tracing::info!("📡 Gesture notification sent: {:?}", gesture.gesture_type);
        Ok(())
    }

    /// Get current battery level
    pub async fn get_battery_level(&self) -> u8 {
        *self.battery_level.read().await
    }

    /// Set battery level
    pub async fn set_battery_level(&self, level: u8) -> Result<()> {
        let clamped_level = level.min(100);

        // Update internal battery level
        {
            let mut battery = self.battery_level.write().await;
            *battery = clamped_level;
        }

        // Update battery simulator
        {
            let mut sim = self.battery_simulator.write().await;
            sim.level = clamped_level;
            sim.last_update = tokio::time::Instant::now();
        }

        // Create and send battery data
        self.update_battery_characteristic().await?;

        tracing::info!("🔋 Battery level manually set to {}%", clamped_level);
        Ok(())
    }

    /// Update battery characteristic with current simulation data
    async fn update_battery_characteristic(&self) -> Result<()> {
        let sim = self.battery_simulator.read().await;

        // Calculate voltage based on battery level (realistic Li-ion curve)
        let voltage = self.calculate_battery_voltage(sim.level);

        // Determine health based on cycle count and level
        let health = if sim.cycle_count > 500 {
            "Poor".to_string()
        } else if sim.cycle_count > 300 {
            "Fair".to_string()
        } else {
            "Good".to_string()
        };

        // Calculate time remaining
        let time_remaining = if sim.is_charging && sim.level < 100 {
            Some(((100 - sim.level) as f32 / sim.charge_rate) as u32)
        } else if !sim.is_charging && sim.level > 0 {
            Some((sim.level as f32 / sim.drain_rate) as u32)
        } else {
            None
        };

        let battery_data = BleBatteryData {
            level: sim.level,
            is_charging: sim.is_charging,
            voltage,
            temperature: sim.temperature,
            health,
            time_remaining,
        };

        drop(sim);

        {
            let mut policy = self.safety_policy.write().await;
            policy.sync_low_battery(battery_data.level);
        }

        let envelope = ProtocolEnvelope::event(
            0,
            crate::protocol::current_protocol_timestamp_ms(),
            SimulatorEvent::Battery(battery_data.to_protocol_snapshot()),
        );
        let frame = BleProtocolAdapter.project_event(&envelope)?;
        self.notify_characteristic(frame.characteristic_uuid, frame.payload)
            .await?;
        self.notify_protocol_state_snapshot().await?;

        Ok(())
    }

    /// Calculate realistic battery voltage based on level
    fn calculate_battery_voltage(&self, level: u8) -> f32 {
        // Realistic Li-ion voltage curve
        match level {
            0..=5 => 3.2 + (level as f32 * 0.1),
            6..=20 => 3.7 + ((level - 5) as f32 * 0.01),
            21..=80 => 3.85 + ((level - 20) as f32 * 0.005),
            81..=95 => 4.15 + ((level - 80) as f32 * 0.002),
            96..=100 => 4.18 + ((level - 95) as f32 * 0.004),
            _ => 4.2,
        }
    }

    /// Start battery simulation (drain/charge over time)
    pub async fn start_battery_simulation(&self) -> Result<()> {
        let battery_simulator = Arc::clone(&self.battery_simulator);
        let battery_level = Arc::clone(&self.battery_level);
        let characteristics = Arc::clone(&self.characteristics);
        let runtime_command_tx = Arc::clone(&self.runtime_command_tx);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Update every 30 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::update_battery_simulation_static(
                    &battery_simulator,
                    &battery_level,
                    &characteristics,
                    &runtime_command_tx,
                )
                .await
                {
                    tracing::error!("Battery simulation error: {}", e);
                }
            }
        });

        tracing::info!("🔋 Battery simulation started");
        Ok(())
    }

    /// Update battery simulation (called periodically)
    #[allow(dead_code)]
    async fn update_battery_simulation(&self) -> Result<()> {
        Self::update_battery_simulation_static(
            &self.battery_simulator,
            &self.battery_level,
            &self.characteristics,
            &self.runtime_command_tx,
        )
        .await
    }

    /// Static method for battery simulation update
    async fn update_battery_simulation_static(
        battery_simulator: &Arc<RwLock<BatterySimulator>>,
        battery_level: &Arc<RwLock<u8>>,
        characteristics: &Arc<RwLock<HashMap<String, BleCharacteristic>>>,
        runtime_command_tx: &Arc<RwLock<Option<mpsc::UnboundedSender<NativeBleCommand>>>>,
    ) -> Result<()> {
        let (old_level, new_level, is_charging, voltage, temperature, health, time_remaining) = {
            let mut sim = battery_simulator.write().await;
            let now = tokio::time::Instant::now();
            let elapsed = now.duration_since(sim.last_update).as_secs_f32() / 60.0; // minutes

            if elapsed < 0.5 {
                return Ok(()); // Don't update too frequently
            }

            let old_level = sim.level;

            if sim.is_charging {
                // Charging logic
                let charge_amount = sim.charge_rate * elapsed;
                sim.level = (sim.level as f32 + charge_amount).min(100.0) as u8;

                // Stop charging when full
                if sim.level >= 100 {
                    sim.is_charging = false;
                    tracing::info!("🔋 Battery fully charged!");
                }
            } else {
                // Draining logic
                let drain_amount = sim.drain_rate * elapsed;
                sim.level = (sim.level as f32 - drain_amount).max(0.0) as u8;

                // Low battery warnings
                if sim.level <= 20 && old_level > 20 {
                    tracing::warn!("🔋 Low battery warning: {}%", sim.level);
                } else if sim.level <= 5 && old_level > 5 {
                    tracing::error!("🔋 Critical battery: {}%", sim.level);
                }
            }

            sim.last_update = now;

            // Calculate battery data
            let voltage = Self::calculate_battery_voltage_static(sim.level);
            let health = if sim.cycle_count > 500 {
                "Poor".to_string()
            } else if sim.cycle_count > 300 {
                "Fair".to_string()
            } else {
                "Good".to_string()
            };

            let time_remaining = if sim.is_charging && sim.level < 100 {
                Some(((100 - sim.level) as f32 / sim.charge_rate) as u32)
            } else if !sim.is_charging && sim.level > 0 {
                Some((sim.level as f32 / sim.drain_rate) as u32)
            } else {
                None
            };

            (
                old_level,
                sim.level,
                sim.is_charging,
                voltage,
                sim.temperature,
                health,
                time_remaining,
            )
        };

        // Update internal battery level
        {
            let mut battery = battery_level.write().await;
            *battery = new_level;
        }

        // Only notify if level changed
        if new_level != old_level {
            let battery_data = BleBatteryData {
                level: new_level,
                is_charging,
                voltage,
                temperature,
                health,
                time_remaining,
            };

            // Update BLE characteristic
            let battery_bytes = serde_json::to_vec(&battery_data)?;
            Self::notify_characteristic_static(
                characteristics,
                ring_uuids::BATTERY_LEVEL_UUID,
                battery_bytes.clone(),
            )
            .await?;
            send_native_runtime_update(
                runtime_command_tx,
                ring_uuids::BATTERY_LEVEL_UUID,
                battery_bytes,
            )
            .await;

            tracing::debug!("🔋 Battery simulation: {}% → {}%", old_level, new_level);
        }

        Ok(())
    }

    /// Static method for calculating battery voltage
    fn calculate_battery_voltage_static(level: u8) -> f32 {
        // Realistic Li-ion voltage curve
        match level {
            0..=5 => 3.2 + (level as f32 * 0.1),
            6..=20 => 3.7 + ((level - 5) as f32 * 0.01),
            21..=80 => 3.85 + ((level - 20) as f32 * 0.005),
            81..=95 => 4.15 + ((level - 80) as f32 * 0.002),
            96..=100 => 4.18 + ((level - 95) as f32 * 0.004),
            _ => 4.2,
        }
    }

    /// Static method for notifying characteristics
    async fn notify_characteristic_static(
        characteristics: &Arc<RwLock<HashMap<String, BleCharacteristic>>>,
        uuid: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        let characteristics_guard = characteristics.read().await;
        if let Some(characteristic) = characteristics_guard.get(uuid) {
            if characteristic.properties.notify || characteristic.properties.indicate {
                emit_notification_payload(characteristic, uuid, data).await
            } else {
                anyhow::bail!("Characteristic {} does not support notifications", uuid);
            }
        } else {
            anyhow::bail!("Characteristic {} not found", uuid);
        }
    }

    /// Toggle charging state
    pub async fn toggle_charging(&self) -> Result<()> {
        let mut sim = self.battery_simulator.write().await;
        sim.is_charging = !sim.is_charging;

        let status = if sim.is_charging {
            "started"
        } else {
            "stopped"
        };
        tracing::info!("🔌 Charging {}", status);

        drop(sim);
        self.update_battery_characteristic().await?;
        Ok(())
    }

    /// Get detailed battery status
    pub async fn get_battery_status(&self) -> BleBatteryData {
        let sim = self.battery_simulator.read().await;

        let voltage = self.calculate_battery_voltage(sim.level);
        let health = if sim.cycle_count > 500 {
            "Poor".to_string()
        } else if sim.cycle_count > 300 {
            "Fair".to_string()
        } else {
            "Good".to_string()
        };

        let time_remaining = if sim.is_charging && sim.level < 100 {
            Some(((100 - sim.level) as f32 / sim.charge_rate) as u32)
        } else if !sim.is_charging && sim.level > 0 {
            Some((sim.level as f32 / sim.drain_rate) as u32)
        } else {
            None
        };

        BleBatteryData {
            level: sim.level,
            is_charging: sim.is_charging,
            voltage,
            temperature: sim.temperature,
            health,
            time_remaining,
        }
    }

    /// Returns the current trust and safety policy.
    pub async fn get_safety_policy(&self) -> TrustPolicy {
        self.safety_policy.read().await.clone()
    }

    /// Transition the simulated device to a new trust state.
    pub async fn transition_trust_state(&self, trust_state: TrustState) -> Result<()> {
        {
            let mut policy = self.safety_policy.write().await;
            policy.transition_to(trust_state);
        }
        self.notify_protocol_state_snapshot().await
    }

    /// Revoke the simulated device trust.
    pub async fn revoke_trust(&self, reason: impl Into<String>) -> Result<()> {
        {
            let mut policy = self.safety_policy.write().await;
            policy.revoke(reason);
        }
        self.notify_protocol_state_snapshot().await
    }

    /// Enable or disable a degraded mode.
    pub async fn set_degraded_mode(&self, mode: DegradedMode, enabled: bool) -> Result<()> {
        {
            let mut policy = self.safety_policy.write().await;
            policy.set_degraded_mode(mode, enabled);
        }
        self.notify_protocol_state_snapshot().await
    }

    /// Update firmware compatibility status.
    pub async fn set_firmware_compatible(&self, compatible: bool) -> Result<()> {
        {
            let mut policy = self.safety_policy.write().await;
            policy.set_firmware_compatible(compatible);
        }
        self.notify_protocol_state_snapshot().await
    }

    /// Build the current shared protocol state snapshot.
    pub async fn get_protocol_state_snapshot(&self) -> DeviceStateSnapshot {
        let battery = self.get_battery_status().await.to_protocol_snapshot();
        let policy = self.safety_policy.read().await.clone();
        let privileged_actions_enabled = policy
            .evaluate(PrivilegedAction::ExecuteProtocolCommand)
            .allowed;

        DeviceStateSnapshot {
            battery,
            trust_state: policy.trust_state,
            degraded_modes: policy.degraded_modes,
            firmware_version: self.firmware_version.clone(),
            protocol_version: SHARED_PROTOCOL_VERSION.to_string(),
            revocation_reason: policy.revocation_reason,
            privileged_actions_enabled,
        }
    }

    /// Notify subscribers with the current shared protocol state snapshot.
    pub async fn notify_protocol_state_snapshot(&self) -> Result<()> {
        let snapshot = self.get_protocol_state_snapshot().await;
        let envelope = ProtocolEnvelope::event(
            0,
            crate::protocol::current_protocol_timestamp_ms(),
            SimulatorEvent::StateSnapshot(snapshot),
        );
        let frame = BleProtocolAdapter.project_event(&envelope)?;
        self.notify_characteristic(frame.characteristic_uuid, frame.payload)
            .await
    }

    /// Get firmware version
    pub fn get_firmware_version(&self) -> &str {
        &self.firmware_version
    }

    /// Get device name
    pub fn get_device_name(&self) -> &str {
        &self.device_name
    }

    /// Get connection state
    pub async fn get_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Read characteristic value
    pub async fn read_characteristic(&self, uuid: &str) -> Result<Vec<u8>> {
        // Config reads are trust-gated like config writes (readable-C2,
        // ratified 2026-07-08) — config contents reveal device posture.
        // Simulator gates at Enrolled (reference-stricter than the bonded
        // device guarantee).
        if uuid == ring_uuids::CONFIG_UUID {
            ensure_privileged_action_allowed_shared(
                &self.safety_policy,
                PrivilegedAction::ExecuteProtocolCommand,
            )
            .await?;
        }

        let characteristics = self.characteristics.read().await;
        if let Some(characteristic) = characteristics.get(uuid) {
            if characteristic.properties.read {
                let value = characteristic.value.read().await;
                tracing::debug!("📖 BLE Read characteristic {}: {} bytes", uuid, value.len());
                Ok(value.clone())
            } else {
                anyhow::bail!("Characteristic {} is not readable", uuid);
            }
        } else {
            anyhow::bail!("Characteristic {} not found", uuid);
        }
    }

    /// Write characteristic value
    pub async fn write_characteristic(&self, uuid: &str, data: Vec<u8>) -> Result<()> {
        // Config writes are trust-gated BEFORE the value is stored — with
        // readable C2, a denied write must not persist state a later read
        // would expose.
        if uuid == ring_uuids::CONFIG_UUID {
            ensure_privileged_action_allowed_shared(
                &self.safety_policy,
                PrivilegedAction::ExecuteProtocolCommand,
            )
            .await?;
        }

        let characteristics = self.characteristics.read().await;
        if let Some(characteristic) = characteristics.get(uuid) {
            if characteristic.properties.write {
                {
                    let mut value = characteristic.value.write().await;
                    store_characteristic_write(&mut value, uuid, &data);
                }
                tracing::info!("✍️ BLE Write characteristic {}: {} bytes", uuid, data.len());

                // Handle specific characteristic writes
                match uuid {
                    ring_uuids::HAPTIC_COMMAND_UUID => {
                        let sequence = haptic_command_sequence(&data);
                        let result = handle_haptic_command_shared(
                            data,
                            &self.safety_policy,
                            self.haptic_tx.as_ref(),
                            &self.event_tx,
                        )
                        .await;
                        // v0.3.0: acknowledge the command (ok or denied) so
                        // policy denials are visible to the host.
                        drop(characteristics);
                        if let Some(payload) = build_haptic_ack_payload(sequence, &result)
                            && let Err(notify_error) = self
                                .notify_characteristic(ring_uuids::STATE_SNAPSHOT_UUID, payload)
                                .await
                        {
                            tracing::debug!("Failed to notify command ack: {notify_error}");
                        }
                        result.map(|_| ())?;
                    }
                    ring_uuids::OTA_UPDATE_UUID => {
                        handle_ota_update_shared(data).await?;
                    }
                    ring_uuids::CONFIG_UUID => {
                        // Trust gate already enforced at the top of this fn,
                        // before the value was stored.
                        log_config_write(&data);
                    }
                    _ => {
                        tracing::debug!("Unhandled characteristic write: {}", uuid);
                    }
                }

                Ok(())
            } else {
                anyhow::bail!("Characteristic {} is not writable", uuid);
            }
        } else {
            anyhow::bail!("Characteristic {} not found", uuid);
        }
    }

    /// Subscribe to characteristic notifications
    pub async fn subscribe_characteristic(&self, uuid: &str, client_id: String) -> Result<()> {
        // Raw sensor stream subscriptions are trust-gated (Bonded or better),
        // mirroring the real ring's enforcement.
        if uuid == ring_uuids::RAW_SENSOR_STREAM_UUID {
            ensure_privileged_action_allowed_shared(
                &self.safety_policy,
                PrivilegedAction::SensitiveDiagnostics,
            )
            .await?;
        }

        let characteristics = self.characteristics.read().await;
        if let Some(characteristic) = characteristics.get(uuid) {
            if characteristic.properties.notify || characteristic.properties.indicate {
                let mut subscribers = characteristic.subscribers.write().await;
                if !subscribers.contains(&client_id) {
                    subscribers.push(client_id.clone());
                    tracing::info!(
                        "🔔 Client {} subscribed to characteristic {}",
                        client_id,
                        uuid
                    );
                }
                Ok(())
            } else {
                anyhow::bail!("Characteristic {} does not support notifications", uuid);
            }
        } else {
            anyhow::bail!("Characteristic {} not found", uuid);
        }
    }

    /// Unsubscribe from characteristic notifications
    pub async fn unsubscribe_characteristic(&self, uuid: &str, client_id: &str) -> Result<()> {
        let characteristics = self.characteristics.read().await;
        if let Some(characteristic) = characteristics.get(uuid) {
            let mut subscribers = characteristic.subscribers.write().await;
            subscribers.retain(|id| id != client_id);
            tracing::info!(
                "🔕 Client {} unsubscribed from characteristic {}",
                client_id,
                uuid
            );
            Ok(())
        } else {
            anyhow::bail!("Characteristic {} not found", uuid);
        }
    }

    /// Send notification to subscribers
    pub async fn notify_characteristic(&self, uuid: &str, data: Vec<u8>) -> Result<()> {
        let characteristics = self.characteristics.read().await;
        if let Some(characteristic) = characteristics.get(uuid) {
            if characteristic.properties.notify || characteristic.properties.indicate {
                emit_notification_payload(characteristic, uuid, data.clone()).await?;
                drop(characteristics);
                send_native_runtime_update(&self.runtime_command_tx, uuid, data).await;
                Ok(())
            } else {
                anyhow::bail!("Characteristic {} does not support notifications", uuid);
            }
        } else {
            anyhow::bail!("Characteristic {} not found", uuid);
        }
    }

    /// Setup BLE services and characteristics
    async fn setup_ble_services(&self) -> Result<()> {
        tracing::info!("Setting up BLE services with UUIDs:");
        tracing::info!("  Haptic Service: {}", ring_uuids::HAPTIC_SERVICE_UUID);
        tracing::info!("  Gesture Events: {}", ring_uuids::GESTURE_EVENT_UUID);
        tracing::info!("  Haptic Commands: {}", ring_uuids::HAPTIC_COMMAND_UUID);
        tracing::info!("  Battery Level: {}", ring_uuids::BATTERY_LEVEL_UUID);
        tracing::info!("  OTA Update: {}", ring_uuids::OTA_UPDATE_UUID);
        Ok(())
    }

    /// Start the advertising loop
    async fn start_advertising_loop(&mut self) -> Result<()> {
        self.start_gesture_handler().await?;

        #[cfg(feature = "native-ble")]
        match self.try_start_native_advertising().await {
            Ok(()) => return Ok(()),
            Err(error) => tracing::warn!(
                %error,
                "Failed to start native BLE peripheral, falling back to simulated advertising"
            ),
        }

        self.start_mock_advertising_loop().await
    }

    async fn start_mock_advertising_loop(&self) -> Result<()> {
        tracing::info!("BLE peripheral is now discoverable by gestura.app (simulated mode)");

        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
        }

        self.start_connection_handler().await
    }

    /// Start gesture event handler
    async fn start_gesture_handler(&mut self) -> Result<()> {
        if let Some(mut gesture_rx) = self.gesture_rx.take() {
            let peripheral = self.clone_for_task();

            tokio::spawn(async move {
                while let Some(gesture) = gesture_rx.recv().await {
                    if let Err(e) = peripheral.notify_gesture(&gesture).await {
                        tracing::error!("Failed to notify gesture: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Start connection handler
    async fn start_connection_handler(&self) -> Result<()> {
        // Simulate periodic client connections for testing
        let peripheral = self.clone_for_task();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                // Simulate gestura.app discovery and connection
                if let Err(e) = peripheral.simulate_client_connection().await {
                    tracing::error!("Failed to simulate client connection: {}", e);
                }
            }
        });

        Ok(())
    }

    #[cfg(feature = "native-ble")]
    async fn try_start_native_advertising(&mut self) -> Result<()> {
        let runtime =
            start_native_ble_runtime(self.clone_for_task(), self.device_name.clone()).await?;
        *self.runtime_command_tx.write().await = Some(runtime.command_tx);
        self.runtime_task = Some(runtime.task);
        tracing::info!(
            "BLE peripheral is now discoverable by gestura.app via native OS BLE advertising"
        );
        Ok(())
    }

    /// Simulate a client connection (for testing)
    #[allow(dead_code)]
    async fn simulate_client_connection(&self) -> Result<()> {
        let client_id = format!("gestura-app-{}", uuid::Uuid::new_v4());

        let client = ClientConnection {
            client_id: client_id.clone(),
            connected_at: std::time::Instant::now(),
            subscribed_characteristics: vec![
                ring_uuids::GESTURE_EVENT_UUID.to_string(),
                ring_uuids::BATTERY_LEVEL_UUID.to_string(),
                ring_uuids::STATE_SNAPSHOT_UUID.to_string(),
            ],
        };

        {
            let mut clients = self.connected_clients.write().await;
            clients.insert(client_id.clone(), client);
        }

        let _ = self
            .event_tx
            .send(BlePeripheralEvent::ClientConnected(client_id));
        tracing::info!("Simulated client connection from gestura.app");

        Ok(())
    }

    /// Handle haptic command from gestura.app (acknowledged, v0.3.0).
    async fn handle_haptic_command(&self, data: Vec<u8>) -> Result<()> {
        let sequence = haptic_command_sequence(&data);
        let result = handle_haptic_command_shared(
            data,
            &self.safety_policy,
            self.haptic_tx.as_ref(),
            &self.event_tx,
        )
        .await;
        if let Some(payload) = build_haptic_ack_payload(sequence, &result)
            && let Err(notify_error) = self
                .notify_characteristic(ring_uuids::STATE_SNAPSHOT_UUID, payload)
                .await
        {
            tracing::debug!("Failed to notify command ack: {notify_error}");
        }
        result.map(|_| ())
    }

    /// Ensure a privileged action is allowed under the current trust policy.
    async fn ensure_privileged_action_allowed(&self, action: PrivilegedAction) -> Result<()> {
        ensure_privileged_action_allowed_shared(&self.safety_policy, action).await
    }

    /// Process a parsed haptic command from any adapter payload.
    async fn process_haptic_command(&self, command: HapticCommand) -> Result<()> {
        process_haptic_command_shared(command, self.haptic_tx.as_ref()).await
    }

    /// Display haptic command in user-friendly format
    async fn display_haptic_command(&self, command: &HapticCommand) {
        display_haptic_command_shared(command).await;
    }

    /// Display raw haptic data
    async fn display_raw_haptic_data(&self, data: &[u8]) {
        display_raw_haptic_data_shared(data).await;
    }

    /// Create visual intensity bar
    fn create_intensity_bar(&self, intensity: f32) -> String {
        create_intensity_bar(intensity)
    }

    /// Simulate haptic feedback visually
    async fn simulate_haptic_feedback(&self, command: &HapticCommand) {
        simulate_haptic_feedback_shared(command).await;
    }

    /// Handle OTA update command
    async fn handle_ota_update(&self, data: Vec<u8>) -> Result<()> {
        handle_ota_update_shared(data).await
    }

    /// Simulate gesture notification
    #[allow(dead_code)]
    async fn simulate_gesture_notification(
        &self,
        client_id: &str,
        gesture: &GestureEvent,
    ) -> Result<()> {
        tracing::info!(
            "📡 BLE Notification → {}: Gesture {:?}",
            client_id,
            gesture.gesture_type
        );
        Ok(())
    }

    /// Simulate receiving a haptic command (for testing)
    pub async fn simulate_haptic_command(
        &self,
        pattern: &str,
        intensity: f32,
        duration_ms: u32,
    ) -> Result<()> {
        let command = HapticCommand {
            pattern: pattern.to_string(),
            intensity,
            duration_ms,
        };

        let frame = BleProtocolAdapter.project_command(&command.to_protocol_envelope(0))?;
        self.write_characteristic(frame.characteristic_uuid, frame.payload)
            .await?;

        Ok(())
    }

    /// Clone for async tasks
    fn clone_for_task(&self) -> BlePeripheralTask {
        BlePeripheralTask {
            state: self.state.clone(),
            event_tx: self.event_tx.clone(),
            connected_clients: self.connected_clients.clone(),
            battery_level: self.battery_level.clone(),
            characteristics: self.characteristics.clone(),
            safety_policy: self.safety_policy.clone(),
            haptic_tx: self.haptic_tx.clone(),
            runtime_command_tx: self.runtime_command_tx.clone(),
        }
    }
}

/// Helper struct for async tasks
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct BlePeripheralTask {
    state: Arc<RwLock<ConnectionState>>,
    event_tx: broadcast::Sender<BlePeripheralEvent>,
    connected_clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    battery_level: Arc<RwLock<u8>>,
    characteristics: Arc<RwLock<HashMap<String, BleCharacteristic>>>,
    safety_policy: Arc<RwLock<TrustPolicy>>,
    haptic_tx: Option<mpsc::UnboundedSender<HapticEvent>>,
    runtime_command_tx: Arc<RwLock<Option<mpsc::UnboundedSender<NativeBleCommand>>>>,
}

#[cfg_attr(not(feature = "native-ble"), allow(dead_code))]
impl BlePeripheralTask {
    pub(crate) async fn set_connection_state(&self, state: ConnectionState) {
        *self.state.write().await = state;
    }

    async fn notify_gesture(&self, gesture: &GestureEvent) -> Result<()> {
        let semantic_event = SemanticGestureEvent::from_runtime_event(gesture);
        let envelope = ProtocolEnvelope::event(
            0,
            semantic_event.timestamp_ms,
            SimulatorEvent::Gesture(semantic_event),
        );
        let frame = BleProtocolAdapter.project_event(&envelope)?;
        self.notify_characteristic(frame.characteristic_uuid, frame.payload)
            .await
    }

    async fn simulate_client_connection(&self) -> Result<()> {
        let client_id = format!("gestura-app-{}", uuid::Uuid::new_v4());

        let client = ClientConnection {
            client_id: client_id.clone(),
            connected_at: std::time::Instant::now(),
            subscribed_characteristics: vec![
                ring_uuids::GESTURE_EVENT_UUID.to_string(),
                ring_uuids::BATTERY_LEVEL_UUID.to_string(),
                ring_uuids::STATE_SNAPSHOT_UUID.to_string(),
            ],
        };

        self.connected_clients
            .write()
            .await
            .insert(client_id.clone(), client);
        let _ = self
            .event_tx
            .send(BlePeripheralEvent::ClientConnected(client_id));
        Ok(())
    }

    pub(crate) async fn read_characteristic(&self, uuid: &str) -> Result<Vec<u8>> {
        // Config reads are trust-gated like config writes (readable-C2,
        // ratified 2026-07-08).
        if uuid == ring_uuids::CONFIG_UUID {
            ensure_privileged_action_allowed_shared(
                &self.safety_policy,
                PrivilegedAction::ExecuteProtocolCommand,
            )
            .await?;
        }

        let characteristics = self.characteristics.read().await;
        let characteristic = characteristics
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))?;
        Ok(characteristic.value.read().await.clone())
    }

    pub(crate) async fn write_characteristic(&self, uuid: &str, data: Vec<u8>) -> Result<()> {
        // Config writes are trust-gated BEFORE the value is stored (see the
        // public write_characteristic for rationale).
        if uuid == ring_uuids::CONFIG_UUID {
            ensure_privileged_action_allowed_shared(
                &self.safety_policy,
                PrivilegedAction::ExecuteProtocolCommand,
            )
            .await?;
        }

        let characteristics = self.characteristics.read().await;
        let characteristic = characteristics
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))?;
        if !characteristic.properties.write {
            anyhow::bail!("Characteristic {} is not writable", uuid);
        }

        {
            let mut value = characteristic.value.write().await;
            store_characteristic_write(&mut value, uuid, &data);
        }

        drop(characteristics);
        match uuid {
            ring_uuids::HAPTIC_COMMAND_UUID => {
                let sequence = haptic_command_sequence(&data);
                let result = handle_haptic_command_shared(
                    data,
                    &self.safety_policy,
                    self.haptic_tx.as_ref(),
                    &self.event_tx,
                )
                .await;
                // v0.3.0: acknowledge the command (ok or denied).
                if let Some(payload) = build_haptic_ack_payload(sequence, &result)
                    && let Err(notify_error) = self
                        .notify_characteristic(ring_uuids::STATE_SNAPSHOT_UUID, payload)
                        .await
                {
                    tracing::debug!("Failed to notify command ack: {notify_error}");
                }
                result.map(|_| ())?;
            }
            ring_uuids::OTA_UPDATE_UUID => {
                handle_ota_update_shared(data).await?;
            }
            ring_uuids::CONFIG_UUID => {
                // Trust gate already enforced at the top of this fn.
                log_config_write(&data);
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) async fn notify_characteristic(&self, uuid: &str, data: Vec<u8>) -> Result<()> {
        let characteristics = self.characteristics.read().await;
        let characteristic = characteristics
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))?;
        emit_notification_payload(characteristic, uuid, data.clone()).await?;
        drop(characteristics);
        send_native_runtime_update(&self.runtime_command_tx, uuid, data).await;
        Ok(())
    }

    pub(crate) async fn apply_subscription_update(
        &self,
        client_id: &str,
        uuid: &str,
        subscribed: bool,
    ) -> Result<()> {
        // Raw sensor stream subscriptions are trust-gated (Bonded or better),
        // mirroring the real ring's enforcement.
        if subscribed && uuid == ring_uuids::RAW_SENSOR_STREAM_UUID {
            ensure_privileged_action_allowed_shared(
                &self.safety_policy,
                PrivilegedAction::SensitiveDiagnostics,
            )
            .await?;
        }

        let characteristics = self.characteristics.read().await;
        let characteristic = characteristics
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))?
            .clone();
        drop(characteristics);

        let mut emit_connected = false;
        let mut emit_disconnected = false;
        {
            let mut clients = self.connected_clients.write().await;
            if subscribed {
                let entry = clients.entry(client_id.to_string()).or_insert_with(|| {
                    emit_connected = true;
                    ClientConnection {
                        client_id: client_id.to_string(),
                        connected_at: std::time::Instant::now(),
                        subscribed_characteristics: Vec::new(),
                    }
                });
                if !entry.subscribed_characteristics.contains(&uuid.to_string()) {
                    entry.subscribed_characteristics.push(uuid.to_string());
                }
            } else if let Some(client) = clients.get_mut(client_id) {
                client
                    .subscribed_characteristics
                    .retain(|existing_uuid| existing_uuid != uuid);
                if client.subscribed_characteristics.is_empty() {
                    clients.remove(client_id);
                    emit_disconnected = true;
                }
            }
        }

        if subscribed {
            let mut subscribers = characteristic.subscribers.write().await;
            if !subscribers.contains(&client_id.to_string()) {
                subscribers.push(client_id.to_string());
            }
        } else {
            characteristic
                .subscribers
                .write()
                .await
                .retain(|subscriber_id| subscriber_id != client_id);
        }

        if emit_connected {
            let _ = self
                .event_tx
                .send(BlePeripheralEvent::ClientConnected(client_id.to_string()));
        }
        let _ = self.event_tx.send(if subscribed {
            BlePeripheralEvent::CharacteristicSubscribed(uuid.to_string())
        } else {
            BlePeripheralEvent::CharacteristicUnsubscribed(uuid.to_string())
        });
        if emit_disconnected {
            let _ = self.event_tx.send(BlePeripheralEvent::ClientDisconnected(
                client_id.to_string(),
            ));
        }

        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    async fn register_live_notifier(
        &self,
        uuid: &str,
        tx: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<(String, Vec<u8>)> {
        let client_id = format!("ble-client-{}", uuid::Uuid::new_v4());
        let characteristics = self.characteristics.read().await;
        let characteristic = characteristics
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))?;

        if !(characteristic.properties.notify || characteristic.properties.indicate) {
            anyhow::bail!("Characteristic {} does not support notifications", uuid);
        }

        let current_value = characteristic.value.read().await.clone();
        characteristic
            .live_notifiers
            .write()
            .await
            .insert(client_id.clone(), tx);
        {
            let mut subscribers = characteristic.subscribers.write().await;
            if !subscribers.contains(&client_id) {
                subscribers.push(client_id.clone());
            }
        }

        self.connected_clients.write().await.insert(
            client_id.clone(),
            ClientConnection {
                client_id: client_id.clone(),
                connected_at: std::time::Instant::now(),
                subscribed_characteristics: vec![uuid.to_string()],
            },
        );

        let _ = self
            .event_tx
            .send(BlePeripheralEvent::ClientConnected(client_id.clone()));
        let _ = self
            .event_tx
            .send(BlePeripheralEvent::CharacteristicSubscribed(
                uuid.to_string(),
            ));

        Ok((client_id, current_value))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    async fn unregister_live_notifier(&self, uuid: &str, client_id: &str) -> Result<()> {
        let characteristics = self.characteristics.read().await;
        let characteristic = characteristics
            .get(uuid)
            .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))?;

        characteristic
            .live_notifiers
            .write()
            .await
            .remove(client_id);
        characteristic
            .subscribers
            .write()
            .await
            .retain(|subscriber_id| subscriber_id != client_id);
        self.connected_clients.write().await.remove(client_id);

        let _ = self
            .event_tx
            .send(BlePeripheralEvent::CharacteristicUnsubscribed(
                uuid.to_string(),
            ));
        let _ = self.event_tx.send(BlePeripheralEvent::ClientDisconnected(
            client_id.to_string(),
        ));

        Ok(())
    }
}

async fn emit_notification_payload(
    characteristic: &BleCharacteristic,
    uuid: &str,
    data: Vec<u8>,
) -> Result<()> {
    let subscribers = characteristic.subscribers.read().await.clone();
    for client_id in &subscribers {
        tracing::info!(
            "📡 BLE Notification → {}: {} ({} bytes)",
            client_id,
            uuid,
            data.len()
        );
    }

    let live_notifiers = characteristic
        .live_notifiers
        .read()
        .await
        .iter()
        .map(|(client_id, tx)| (client_id.clone(), tx.clone()))
        .collect::<Vec<_>>();
    let mut stale_clients = Vec::new();
    for (client_id, tx) in live_notifiers {
        if tx.send(data.clone()).is_err() {
            stale_clients.push(client_id);
        }
    }
    if !stale_clients.is_empty() {
        let mut notifiers = characteristic.live_notifiers.write().await;
        for client_id in stale_clients {
            notifiers.remove(&client_id);
        }
    }

    *characteristic.value.write().await = data;
    Ok(())
}

async fn send_native_runtime_update(
    runtime_command_tx: &Arc<RwLock<Option<mpsc::UnboundedSender<NativeBleCommand>>>>,
    uuid: &str,
    data: Vec<u8>,
) {
    let command_tx = runtime_command_tx.read().await.clone();
    if let Some(command_tx) = command_tx {
        let _ = command_tx.send(NativeBleCommand::UpdateCharacteristic {
            uuid: uuid.to_string(),
            value: data,
        });
    }
}

/// Decodes and logs a Config characteristic write.
/// Layout: [0]=sensitivity [1]=raw-stream opt-in [2]=gesture mask
/// [3]=HID projection enable (optional, backward-compatible — the SDK writes
/// 0 on app takeover / 1 on release, approved 2026-07-07).
fn log_config_write(data: &[u8]) {
    let sensitivity = data.first().copied();
    let raw_opt_in = data.get(1).map(|b| *b != 0);
    let gesture_mask = data.get(2).copied();
    let hid_enabled = data.get(3).map(|b| *b != 0);
    tracing::info!(
        "⚙️ Config write: sensitivity={:?} raw_stream={:?} gesture_mask={:?} hid={}",
        sensitivity,
        raw_opt_in,
        gesture_mask,
        match hid_enabled {
            Some(true) => "ON (restored)",
            Some(false) => "OFF (app takeover)",
            None => "unchanged (byte 3 omitted)",
        }
    );
}

/// Stores a characteristic write into the retained value. Config (C2) writes
/// use partial-update semantics — the contract treats shorter writes as
/// leaving trailing fields unchanged (2026-07-07 firmware note), so a write
/// that omits byte 3 (HID) must not drop the stored HID state that a
/// readable-C2 read would later expose. All other characteristics replace
/// their value wholesale.
fn store_characteristic_write(value: &mut Vec<u8>, uuid: &str, data: &[u8]) {
    if uuid == ring_uuids::CONFIG_UUID && data.len() < value.len() {
        value[..data.len()].copy_from_slice(data);
    } else {
        *value = data.to_vec();
    }
}

/// Extracts the command sequence from raw haptic-write bytes.
/// Legacy (non-envelope) writes have no sequence; 0 = unsequenced.
fn haptic_command_sequence(data: &[u8]) -> u64 {
    serde_json::from_slice::<ProtocolEnvelope<SimulatorCommand>>(data)
        .map(|envelope| envelope.sequence)
        .unwrap_or(0)
}

/// Outcome of a haptic-command write: distinguishes commands that actually
/// executed from payloads that never decoded into a command (the raw-data
/// fallback), so acks don't report `Ok` for writes that did nothing.
enum HapticCommandOutcome {
    Executed,
    Unparsed,
}

/// Builds the v0.3.0 ack envelope for a haptic-command outcome. Acks ride the
/// state-snapshot characteristic as full envelopes (projection decision —
/// no dedicated characteristic in the frozen UUID table).
fn build_haptic_ack_payload(
    sequence: u64,
    result: &Result<HapticCommandOutcome>,
) -> Option<Vec<u8>> {
    let ack = match result {
        Ok(HapticCommandOutcome::Executed) => AckPayload {
            sequence,
            status: AckStatus::Ok,
            reason: None,
        },
        Ok(HapticCommandOutcome::Unparsed) => AckPayload {
            sequence,
            status: AckStatus::Error,
            reason: Some("payload did not decode as a haptic command".to_string()),
        },
        // The only Err path in handle_haptic_command_shared is the
        // privileged-action gate, so errors are policy denials.
        Err(error) => AckPayload {
            sequence,
            status: AckStatus::Denied,
            reason: Some(error.to_string()),
        },
    };
    let envelope = ProtocolEnvelope::event(
        sequence,
        crate::protocol::current_protocol_timestamp_ms(),
        SimulatorEvent::Ack(ack),
    );
    serde_json::to_vec(&envelope).ok()
}

async fn handle_haptic_command_shared(
    data: Vec<u8>,
    safety_policy: &Arc<RwLock<TrustPolicy>>,
    haptic_tx: Option<&mpsc::UnboundedSender<HapticEvent>>,
    event_tx: &broadcast::Sender<BlePeripheralEvent>,
) -> Result<HapticCommandOutcome> {
    if let Ok(command) = HapticCommand::try_from_protocol_bytes(&data) {
        ensure_privileged_action_allowed_shared(safety_policy, PrivilegedAction::HapticCommand)
            .await?;
        let _ = event_tx.send(BlePeripheralEvent::HapticCommandReceived(command.clone()));
        process_haptic_command_shared(command, haptic_tx).await?;
        return Ok(HapticCommandOutcome::Executed);
    }

    if let Ok(command_str) = String::from_utf8(data.clone())
        && let Ok(command) = serde_json::from_str::<HapticCommand>(&command_str)
    {
        ensure_privileged_action_allowed_shared(safety_policy, PrivilegedAction::HapticCommand)
            .await?;
        let _ = event_tx.send(BlePeripheralEvent::HapticCommandReceived(command.clone()));
        process_haptic_command_shared(command, haptic_tx).await?;
        return Ok(HapticCommandOutcome::Executed);
    }

    tracing::info!("📳 Raw Haptic Data Received: {} bytes", data.len());
    display_raw_haptic_data_shared(&data).await;
    Ok(HapticCommandOutcome::Unparsed)
}

async fn ensure_privileged_action_allowed_shared(
    safety_policy: &Arc<RwLock<TrustPolicy>>,
    action: PrivilegedAction,
) -> Result<()> {
    let decision = safety_policy.read().await.evaluate(action);
    if decision.allowed {
        return Ok(());
    }

    let reason = decision
        .reason
        .unwrap_or_else(|| "privileged action denied by safety policy".to_string());
    tracing::warn!("Denied privileged BLE action: {}", reason);
    anyhow::bail!(reason)
}

async fn process_haptic_command_shared(
    command: HapticCommand,
    haptic_tx: Option<&mpsc::UnboundedSender<HapticEvent>>,
) -> Result<()> {
    display_haptic_command_shared(&command).await;

    if let Some(haptic_tx) = haptic_tx {
        let haptic_event = HapticEvent {
            pattern: match command.pattern.as_str() {
                // Ratified v0.2.0 names first; v0.1.0 legacy names accepted.
                "confirm" | "success" => crate::emulator::HapticPattern::Success,
                "tick" | "notify" => crate::emulator::HapticPattern::Notify,
                "double_tick" => crate::emulator::HapticPattern::DoubleTick,
                "error" => crate::emulator::HapticPattern::Error,
                _ => crate::emulator::HapticPattern::Custom {
                    intensity: command.intensity,
                    duration: Duration::from_millis(command.duration_ms as u64),
                },
            },
            timestamp: tokio::time::Instant::now(),
        };
        let _ = haptic_tx.send(haptic_event);
    }

    Ok(())
}

async fn display_haptic_command_shared(command: &HapticCommand) {
    let intensity_bar = create_intensity_bar(command.intensity);
    let pattern_emoji = match command.pattern.as_str() {
        "confirm" | "success" => "✅",
        "tick" | "notify" => "🔔",
        "double_tick" => "🔔🔔",
        "error" => "❌",
        "warning" => "⚠️",
        "pulse" => "💓",
        "buzz" => "📳",
        "waveform" => "🌊",
        _ => "🎛️",
    };

    tracing::info!("📳 Haptic Command from gestura.app:");
    tracing::info!("   {} Pattern: {}", pattern_emoji, command.pattern);
    tracing::info!(
        "   🔊 Intensity: {:.1}% {}",
        command.intensity * 100.0,
        intensity_bar
    );
    tracing::info!("   ⏱️ Duration: {}ms", command.duration_ms);

    simulate_haptic_feedback_shared(command).await;
}

async fn display_raw_haptic_data_shared(data: &[u8]) {
    tracing::info!("📳 Raw Haptic Data from gestura.app:");
    tracing::info!("   📊 Size: {} bytes", data.len());
    if data.len() <= 16 {
        let hex_data: String = data
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        tracing::info!("   🔢 Data: {}", hex_data);
    } else {
        let preview: String = data[..8]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ");
        tracing::info!(
            "   🔢 Preview: {} ... ({} more bytes)",
            preview,
            data.len() - 8
        );
    }
}

fn create_intensity_bar(intensity: f32) -> String {
    let bars = (intensity * 10.0) as usize;
    let filled = "█".repeat(bars);
    let empty = "░".repeat(10 - bars);
    format!("[{filled}{empty}]")
}

async fn simulate_haptic_feedback_shared(command: &HapticCommand) {
    let pattern = match command.pattern.as_str() {
        "notify" => "📳 buzz",
        "success" => "✨ gentle pulse",
        "error" => "⚡ sharp buzz",
        "warning" => "⚠️ double tap",
        "pulse" => "💓 rhythmic pulse",
        "buzz" => "📳 continuous buzz",
        _ => "🎛️ custom pattern",
    };

    tracing::info!("   🎭 Simulating: {}", pattern);

    if command.duration_ms > 1000 {
        tracing::info!("   ⏳ Long haptic feedback...");
    } else if command.duration_ms > 500 {
        tracing::info!("   ⏱️ Medium haptic feedback...");
    } else {
        tracing::info!("   ⚡ Quick haptic feedback!");
    }
}

async fn handle_ota_update_shared(data: Vec<u8>) -> Result<()> {
    tracing::info!("🔄 OTA Update Command: {} bytes", data.len());

    if !data.is_empty() {
        match data[0] {
            0x01 => tracing::info!("🔄 OTA: Starting firmware update..."),
            0x02 => tracing::info!("🔄 OTA: Firmware update in progress..."),
            0x03 => tracing::info!("✅ OTA: Firmware update completed successfully"),
            0x04 => tracing::error!("❌ OTA: Firmware update failed"),
            _ => tracing::debug!("🔄 OTA: Unknown command: 0x{:02x}", data[0]),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unparsed_haptic_payload_acks_error_not_ok() {
        // A write that never decoded into a command must not be acked Ok —
        // hosts would believe an unexecuted command succeeded.
        let payload = build_haptic_ack_payload(7, &Ok(HapticCommandOutcome::Unparsed))
            .expect("ack payload should serialize");
        let envelope: ProtocolEnvelope<SimulatorEvent> =
            serde_json::from_slice(&payload).expect("ack envelope should parse");
        match envelope.payload {
            SimulatorEvent::Ack(ack) => {
                assert_eq!(ack.sequence, 7);
                assert_eq!(ack.status, AckStatus::Error);
                assert!(ack.reason.is_some());
            }
            other => panic!("expected ack event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn notify_characteristic_delivers_to_live_notifiers() {
        let peripheral = BlePeripheral::new(ConnectionConfig::default()).unwrap();
        let task = peripheral.clone_for_task();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let (client_id, _) = task
            .register_live_notifier(ring_uuids::BATTERY_LEVEL_UUID, tx)
            .await
            .unwrap();
        task.notify_characteristic(ring_uuids::BATTERY_LEVEL_UUID, vec![1, 2, 3])
            .await
            .unwrap();

        assert_eq!(rx.recv().await.unwrap(), vec![1, 2, 3]);
        assert_eq!(
            peripheral
                .read_characteristic(ring_uuids::BATTERY_LEVEL_UUID)
                .await
                .unwrap(),
            vec![1, 2, 3]
        );

        task.unregister_live_notifier(ring_uuids::BATTERY_LEVEL_UUID, &client_id)
            .await
            .unwrap();
    }
}
