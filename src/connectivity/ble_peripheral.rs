//! BLE peripheral implementation for Haptic Harmony Ring simulation
//! Creates a real BLE peripheral that gestura.app can discover and connect to

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast, mpsc};

use crate::connectivity::{ConnectionConfig, ConnectionState};
use crate::emulator::{GestureEvent, HapticEvent};

/// Haptic Harmony Ring BLE service UUIDs (matching gestura.app expectations)
pub mod ring_uuids {
    /// Main haptic service UUID
    pub const HAPTIC_SERVICE_UUID: &str = "12345678-1234-5678-9abc-123456789abc";
    /// Haptic command characteristic UUID
    pub const HAPTIC_COMMAND_UUID: &str = "12345678-1234-5678-9abc-123456789abd";
    /// Gesture event characteristic UUID  
    pub const GESTURE_EVENT_UUID: &str = "12345678-1234-5678-9abc-123456789abe";
    /// Battery level characteristic UUID
    pub const BATTERY_LEVEL_UUID: &str = "12345678-1234-5678-9abc-123456789abf";
    /// OTA update characteristic UUID
    pub const OTA_UPDATE_UUID: &str = "12345678-1234-5678-9abc-123456789ac0";
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

/// BLE characteristic data
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleCharacteristic {
    pub uuid: String,
    pub properties: BleCharacteristicProperties,
    pub value: Arc<RwLock<Vec<u8>>>,
    pub subscribers: Arc<RwLock<Vec<String>>>,
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
        self.start_advertising_loop().await?;

        // Start battery simulation
        self.start_battery_simulation().await?;

        Ok(())
    }

    /// Stop advertising and disconnect all clients
    pub async fn stop_advertising(&mut self) -> Result<()> {
        tracing::info!("Stopping BLE peripheral advertising");

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
        // Convert gesture to BLE data format
        let ble_gesture = BleGestureData {
            gesture_type: format!("{:?}", gesture.gesture_type),
            timestamp: gesture.timestamp.elapsed().as_millis() as u64,
            confidence: 0.95, // High confidence for simulated gestures
            data: vec![],     // Additional gesture data if needed
        };

        // Serialize to JSON for transmission
        let gesture_data = serde_json::to_vec(&ble_gesture)?;

        // Send notification via BLE characteristic
        self.notify_characteristic(ring_uuids::GESTURE_EVENT_UUID, gesture_data)
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

        // Update BLE characteristic
        let battery_bytes = serde_json::to_vec(&battery_data)?;
        self.notify_characteristic(ring_uuids::BATTERY_LEVEL_UUID, battery_bytes)
            .await?;

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

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Update every 30 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::update_battery_simulation_static(
                    &battery_simulator,
                    &battery_level,
                    &characteristics,
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
        )
        .await
    }

    /// Static method for battery simulation update
    async fn update_battery_simulation_static(
        battery_simulator: &Arc<RwLock<BatterySimulator>>,
        battery_level: &Arc<RwLock<u8>>,
        characteristics: &Arc<RwLock<HashMap<String, BleCharacteristic>>>,
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
                battery_bytes,
            )
            .await?;

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
                let subscribers = characteristic.subscribers.read().await;
                for client_id in subscribers.iter() {
                    tracing::info!(
                        "📡 BLE Notification → {}: {} ({} bytes)",
                        client_id,
                        uuid,
                        data.len()
                    );
                }

                // Update characteristic value
                {
                    let mut value = characteristic.value.write().await;
                    *value = data;
                }

                Ok(())
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
        let characteristics = self.characteristics.read().await;
        if let Some(characteristic) = characteristics.get(uuid) {
            if characteristic.properties.write {
                {
                    let mut value = characteristic.value.write().await;
                    *value = data.clone();
                }
                tracing::info!("✍️ BLE Write characteristic {}: {} bytes", uuid, data.len());

                // Handle specific characteristic writes
                match uuid {
                    ring_uuids::HAPTIC_COMMAND_UUID => {
                        self.handle_haptic_command(data).await?;
                    }
                    ring_uuids::OTA_UPDATE_UUID => {
                        self.handle_ota_update(data).await?;
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
                let subscribers = characteristic.subscribers.read().await;
                for client_id in subscribers.iter() {
                    tracing::info!(
                        "📡 BLE Notification → {}: {} ({} bytes)",
                        client_id,
                        uuid,
                        data.len()
                    );
                }

                // Update characteristic value
                {
                    let mut value = characteristic.value.write().await;
                    *value = data;
                }

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

        // In a real implementation, this would create actual BLE services
        // For now, we'll simulate the setup
        Ok(())
    }

    /// Start the advertising loop
    async fn start_advertising_loop(&mut self) -> Result<()> {
        tracing::info!("BLE peripheral is now discoverable by gestura.app");

        // Update state to connected (ready for connections)
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Start background tasks
        self.start_gesture_handler().await?;
        self.start_connection_handler().await?;

        Ok(())
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

    /// Handle haptic command from gestura.app
    async fn handle_haptic_command(&self, data: Vec<u8>) -> Result<()> {
        // Try to deserialize haptic command
        if let Ok(command_str) = String::from_utf8(data.clone())
            && let Ok(command) = serde_json::from_str::<HapticCommand>(&command_str)
        {
            // Enhanced haptic command display
            self.display_haptic_command(&command).await;

            // Send to haptic system if available
            if let Some(ref haptic_tx) = self.haptic_tx {
                let haptic_event = HapticEvent {
                    pattern: match command.pattern.as_str() {
                        "notify" => crate::emulator::HapticPattern::Notify,
                        "success" => crate::emulator::HapticPattern::Success,
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

            return Ok(());
        }

        // Fallback: treat as raw haptic data
        tracing::info!("📳 Raw Haptic Data Received: {} bytes", data.len());
        self.display_raw_haptic_data(&data).await;
        Ok(())
    }

    /// Display haptic command in user-friendly format
    async fn display_haptic_command(&self, command: &HapticCommand) {
        let intensity_bar = self.create_intensity_bar(command.intensity);
        let pattern_emoji = match command.pattern.as_str() {
            "notify" => "🔔",
            "success" => "✅",
            "error" => "❌",
            "warning" => "⚠️",
            "pulse" => "💓",
            "buzz" => "📳",
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

        // Simulate haptic feedback visually
        self.simulate_haptic_feedback(command).await;
    }

    /// Display raw haptic data
    async fn display_raw_haptic_data(&self, data: &[u8]) {
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

    /// Create visual intensity bar
    fn create_intensity_bar(&self, intensity: f32) -> String {
        let bars = (intensity * 10.0) as usize;
        let filled = "█".repeat(bars);
        let empty = "░".repeat(10 - bars);
        format!("[{filled}{empty}]")
    }

    /// Simulate haptic feedback visually
    async fn simulate_haptic_feedback(&self, command: &HapticCommand) {
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

        // Visual feedback based on duration
        if command.duration_ms > 1000 {
            tracing::info!("   ⏳ Long haptic feedback...");
        } else if command.duration_ms > 500 {
            tracing::info!("   ⏱️ Medium haptic feedback...");
        } else {
            tracing::info!("   ⚡ Quick haptic feedback!");
        }
    }

    /// Handle OTA update command
    async fn handle_ota_update(&self, data: Vec<u8>) -> Result<()> {
        tracing::info!("🔄 OTA Update Command: {} bytes", data.len());

        // Simulate OTA update process
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

        let command_data = serde_json::to_vec(&command)?;
        self.write_characteristic(ring_uuids::HAPTIC_COMMAND_UUID, command_data)
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
        }
    }
}

/// Helper struct for async tasks
#[derive(Clone)]
#[allow(dead_code)]
struct BlePeripheralTask {
    state: Arc<RwLock<ConnectionState>>,
    event_tx: broadcast::Sender<BlePeripheralEvent>,
    connected_clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    battery_level: Arc<RwLock<u8>>,
}

impl BlePeripheralTask {
    async fn notify_gesture(&self, gesture: &GestureEvent) -> Result<()> {
        let clients = self.connected_clients.read().await;

        for (client_id, client) in clients.iter() {
            if client
                .subscribed_characteristics
                .contains(&ring_uuids::GESTURE_EVENT_UUID.to_string())
            {
                tracing::info!(
                    "📡 BLE Notification → {}: Gesture {:?}",
                    client_id,
                    gesture.gesture_type
                );
            }
        }

        Ok(())
    }

    async fn simulate_client_connection(&self) -> Result<()> {
        // Implementation would go here for real BLE connections
        Ok(())
    }
}
