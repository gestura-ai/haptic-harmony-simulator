//! Mock BLE adapter for simulation

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::{Connection, ConnectionConfig, ConnectionState};

/// Mock BLE adapter
#[allow(dead_code)]
pub struct BleMockAdapter {
    config: ConnectionConfig,
    state: ConnectionState,
    services: HashMap<String, BleService>,
    message_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    message_rx: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    last_activity: Option<Instant>,
}

/// Mock BLE service
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleService {
    pub uuid: String,
    pub characteristics: HashMap<String, BleCharacteristic>,
}

/// Mock BLE characteristic
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleCharacteristic {
    pub uuid: String,
    pub properties: BleProperties,
    pub value: Vec<u8>,
}

/// BLE characteristic properties
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleProperties {
    pub read: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
}

#[allow(dead_code)]
impl BleMockAdapter {
    /// Create new mock BLE adapter
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        let mut adapter = Self {
            config,
            state: ConnectionState::Disconnected,
            services: HashMap::new(),
            message_tx: None,
            message_rx: None,
            last_activity: None,
        };

        // Initialize mock services
        adapter.init_mock_services();

        Ok(adapter)
    }

    /// Initialize mock BLE services that match Haptic Harmony Ring
    fn init_mock_services(&mut self) {
        // Gesture service
        let mut gesture_service = BleService {
            uuid: "12345678-1234-1234-1234-123456789abc".to_string(),
            characteristics: HashMap::new(),
        };

        gesture_service.characteristics.insert(
            "gesture_data".to_string(),
            BleCharacteristic {
                uuid: "12345678-1234-1234-1234-123456789abd".to_string(),
                properties: BleProperties {
                    read: true,
                    write: false,
                    notify: true,
                    indicate: false,
                },
                value: vec![],
            },
        );

        // Haptic service
        let mut haptic_service = BleService {
            uuid: "87654321-4321-4321-4321-cba987654321".to_string(),
            characteristics: HashMap::new(),
        };

        haptic_service.characteristics.insert(
            "haptic_control".to_string(),
            BleCharacteristic {
                uuid: "87654321-4321-4321-4321-cba987654322".to_string(),
                properties: BleProperties {
                    read: false,
                    write: true,
                    notify: false,
                    indicate: false,
                },
                value: vec![],
            },
        );

        self.services.insert("gesture".to_string(), gesture_service);
        self.services.insert("haptic".to_string(), haptic_service);
    }

    /// Simulate device discovery
    pub async fn discover_devices(&self) -> Result<Vec<MockDevice>> {
        // Simulate discovery delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(vec![MockDevice {
            name: "Haptic Harmony Ring".to_string(),
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            rssi: -45,
            services: self.services.keys().cloned().collect(),
        }])
    }

    /// Write to a characteristic
    pub async fn write_characteristic(
        &mut self,
        service: &str,
        characteristic: &str,
        data: &[u8],
    ) -> Result<()> {
        if let Some(service_obj) = self.services.get_mut(service)
            && let Some(char_obj) = service_obj.characteristics.get_mut(characteristic)
            && char_obj.properties.write
        {
            char_obj.value = data.to_vec();
            self.last_activity = Some(Instant::now());

            tracing::debug!(
                "BLE write to {}:{} - {} bytes",
                service,
                characteristic,
                data.len()
            );
            return Ok(());
        }

        anyhow::bail!(
            "Characteristic not found or not writable: {}:{}",
            service,
            characteristic
        );
    }

    /// Read from a characteristic
    pub async fn read_characteristic(
        &self,
        service: &str,
        characteristic: &str,
    ) -> Result<Vec<u8>> {
        if let Some(service_obj) = self.services.get(service)
            && let Some(char_obj) = service_obj.characteristics.get(characteristic)
            && char_obj.properties.read
        {
            tracing::debug!(
                "BLE read from {}:{} - {} bytes",
                service,
                characteristic,
                char_obj.value.len()
            );
            return Ok(char_obj.value.clone());
        }

        anyhow::bail!(
            "Characteristic not found or not readable: {}:{}",
            service,
            characteristic
        );
    }

    /// Subscribe to notifications
    pub async fn subscribe_notifications(
        &mut self,
        service: &str,
        characteristic: &str,
    ) -> Result<()> {
        if let Some(service_obj) = self.services.get(service)
            && let Some(char_obj) = service_obj.characteristics.get(characteristic)
            && char_obj.properties.notify
        {
            tracing::info!(
                "Subscribed to notifications: {}:{}",
                service,
                characteristic
            );
            return Ok(());
        }

        anyhow::bail!(
            "Characteristic not found or notifications not supported: {}:{}",
            service,
            characteristic
        );
    }

    /// Simulate sending a notification
    pub async fn simulate_notification(
        &self,
        service: &str,
        characteristic: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            tx.send(data)?;
            tracing::debug!("BLE notification sent from {}:{}", service, characteristic);
        }
        Ok(())
    }

    /// Get connection statistics
    pub fn get_stats(&self) -> BleStats {
        BleStats {
            connected_time: self.last_activity.map(|t| t.elapsed()),
            services_count: self.services.len(),
            characteristics_count: self
                .services
                .values()
                .map(|s| s.characteristics.len())
                .sum(),
        }
    }
}

#[async_trait::async_trait]
impl Connection for BleMockAdapter {
    async fn connect(&mut self) -> Result<()> {
        if matches!(self.state, ConnectionState::Connected) {
            return Ok(());
        }

        self.state = ConnectionState::Connecting;

        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Create message channel
        let (tx, rx) = mpsc::unbounded_channel();
        self.message_tx = Some(tx);
        self.message_rx = Some(rx);

        self.state = ConnectionState::Connected;
        self.last_activity = Some(Instant::now());

        tracing::info!("BLE mock adapter connected");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.state = ConnectionState::Disconnected;
        self.message_tx = None;
        self.message_rx = None;
        self.last_activity = None;

        tracing::info!("BLE mock adapter disconnected");
        Ok(())
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        if !self.is_connected() {
            anyhow::bail!("BLE adapter not connected");
        }

        // Simulate sending to haptic control characteristic
        tracing::debug!("BLE sending {} bytes", data.len());

        // In a real implementation, this would write to the appropriate characteristic
        // For simulation, we just log it
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<Vec<u8>>> {
        if let Some(ref mut rx) = self.message_rx {
            match rx.try_recv() {
                Ok(data) => Ok(Some(data)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    anyhow::bail!("BLE message channel disconnected");
                }
            }
        } else {
            Ok(None)
        }
    }

    fn get_state(&self) -> ConnectionState {
        self.state.clone()
    }
}

/// Mock BLE device
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MockDevice {
    pub name: String,
    pub address: String,
    pub rssi: i16,
    pub services: Vec<String>,
}

/// BLE connection statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BleStats {
    pub connected_time: Option<Duration>,
    pub services_count: usize,
    pub characteristics_count: usize,
}
