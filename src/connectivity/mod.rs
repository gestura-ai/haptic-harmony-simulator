//! Connectivity modules for mock BLE and socket communication
//!
//! This module provides mock implementations for connecting to gestura.app

pub mod ble_mock;
pub mod ble_peripheral;
mod native_ble_backend;
pub mod socket_client;

pub use ble_mock::BleMockAdapter;
pub use ble_peripheral::BlePeripheral;
pub use socket_client::SocketClient;

use anyhow::Result;
use std::time::Duration;

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Connection configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConnectionConfig {
    /// Target host for gestura.app
    pub host: String,
    /// Target port
    pub port: u16,
    /// Connection timeout
    pub timeout: Duration,
    /// Retry attempts
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Enable mock authentication
    pub enable_auth: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_delay: Duration::from_secs(2),
            enable_auth: true,
        }
    }
}

/// Trait for connection implementations
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait Connection: Send + Sync {
    /// Connect to the target
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the target
    async fn disconnect(&mut self) -> Result<()>;

    /// Send data
    async fn send(&self, data: &[u8]) -> Result<()>;

    /// Receive data (non-blocking)
    async fn receive(&mut self) -> Result<Option<Vec<u8>>>;

    /// Get connection state
    fn get_state(&self) -> ConnectionState;

    /// Check if connected
    fn is_connected(&self) -> bool {
        matches!(self.get_state(), ConnectionState::Connected)
    }
}

/// Connection manager for handling multiple connection types
#[allow(dead_code)]
pub struct ConnectionManager {
    ble_adapter: Option<BleMockAdapter>,
    socket_client: Option<SocketClient>,
    config: ConnectionConfig,
    current_state: ConnectionState,
}

#[allow(dead_code)]
impl ConnectionManager {
    /// Create new connection manager
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            ble_adapter: None,
            socket_client: None,
            config,
            current_state: ConnectionState::Disconnected,
        }
    }

    /// Initialize BLE mock adapter
    pub async fn init_ble(&mut self) -> Result<()> {
        let adapter = BleMockAdapter::new(self.config.clone()).await?;
        self.ble_adapter = Some(adapter);
        Ok(())
    }

    /// Initialize socket client
    pub async fn init_socket(&mut self) -> Result<()> {
        let client = SocketClient::new(self.config.clone()).await?;
        self.socket_client = Some(client);
        Ok(())
    }

    /// Connect using preferred method
    pub async fn connect(&mut self) -> Result<()> {
        self.current_state = ConnectionState::Connecting;

        // Try socket first, then BLE
        if let Some(ref mut client) = self.socket_client {
            match client.connect().await {
                Ok(()) => {
                    self.current_state = ConnectionState::Connected;
                    tracing::info!(
                        "Connected via socket to {}:{}",
                        self.config.host,
                        self.config.port
                    );
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Socket connection failed: {}", e);
                }
            }
        }

        if let Some(ref mut adapter) = self.ble_adapter {
            match adapter.connect().await {
                Ok(()) => {
                    self.current_state = ConnectionState::Connected;
                    tracing::info!("Connected via BLE mock");
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("BLE connection failed: {}", e);
                }
            }
        }

        self.current_state = ConnectionState::Error("All connection methods failed".to_string());
        anyhow::bail!("Failed to establish connection");
    }

    /// Disconnect all connections
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut client) = self.socket_client {
            let _ = client.disconnect().await;
        }

        if let Some(ref mut adapter) = self.ble_adapter {
            let _ = adapter.disconnect().await;
        }

        self.current_state = ConnectionState::Disconnected;
        tracing::info!("All connections disconnected");
        Ok(())
    }

    /// Send data through active connection
    pub async fn send_data(&self, data: &[u8]) -> Result<()> {
        if let Some(ref client) = self.socket_client
            && client.is_connected()
        {
            return client.send(data).await;
        }

        if let Some(ref adapter) = self.ble_adapter
            && adapter.is_connected()
        {
            return adapter.send(data).await;
        }

        anyhow::bail!("No active connection available");
    }

    /// Get current connection state
    pub fn get_state(&self) -> &ConnectionState {
        &self.current_state
    }

    /// Check if any connection is active
    pub fn is_connected(&self) -> bool {
        matches!(self.current_state, ConnectionState::Connected)
    }
}
