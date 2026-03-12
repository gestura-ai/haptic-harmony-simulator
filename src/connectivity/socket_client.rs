//! Socket client for connecting to gestura.app

use anyhow::Result;
use base64::engine::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::{Connection, ConnectionConfig, ConnectionState};
use crate::protocol::{
    HapticCommandPayload, ProtocolEnvelope, SemanticGestureEvent, SimulatorCommand, SimulatorEvent,
    current_protocol_timestamp_ms,
};
use crate::transport_adapters::SocketProtocolAdapter;

/// Socket client for HTTP/WebSocket communication
pub struct SocketClient {
    config: ConnectionConfig,
    state: ConnectionState,
    client: Client,
    base_url: String,
    session_id: Option<String>,
    message_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    message_rx: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    last_activity: Option<Instant>,
}

/// Message format for gestura.app communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GesturaMessage {
    pub message_type: String,
    pub session_id: Option<String>,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

/// Response from gestura.app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GesturaResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[allow(dead_code)]
impl SocketClient {
    /// Create new socket client
    pub async fn new(config: ConnectionConfig) -> Result<Self> {
        let client = Client::builder().timeout(config.timeout).build()?;

        let base_url = format!("http://{}:{}", config.host, config.port);

        Ok(Self {
            config,
            state: ConnectionState::Disconnected,
            client,
            base_url,
            session_id: None,
            message_tx: None,
            message_rx: None,
            last_activity: None,
        })
    }

    /// Authenticate with gestura.app
    async fn authenticate(&mut self) -> Result<()> {
        if !self.config.enable_auth {
            return Ok(());
        }

        let auth_data = serde_json::json!({
            "device_type": "haptic_harmony_ring_sim",
            "version": "0.1.0",
            "capabilities": ["gesture", "haptic", "mcp"]
        });

        let response = self
            .client
            .post(format!("{}/api/auth", self.base_url))
            .json(&auth_data)
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: GesturaResponse = response.json().await?;

            if let Some(data) = auth_response.data
                && let Some(session_id) = data.get("session_id").and_then(|v| v.as_str())
            {
                self.session_id = Some(session_id.to_string());
                tracing::info!("Authenticated with session ID: {}", session_id);
                return Ok(());
            }
        }

        anyhow::bail!("Authentication failed");
    }

    /// Send a gesture event to gestura.app
    pub async fn send_gesture(&self, gesture_type: &str, data: serde_json::Value) -> Result<()> {
        let timestamp_ms = current_protocol_timestamp_ms();
        let protocol_event = ProtocolEnvelope::event(
            0,
            timestamp_ms,
            SimulatorEvent::Gesture(SemanticGestureEvent::from_legacy_parts(
                gesture_type,
                &data,
                timestamp_ms,
            )?),
        );

        let message = GesturaMessage {
            message_type: "gesture".to_string(),
            session_id: self.session_id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: serde_json::json!({
                "protocol": protocol_event,
                "legacy": {
                    "gesture_type": gesture_type,
                    "gesture_data": data
                }
            }),
        };

        self.send_message(message).await
    }

    /// Send a haptic feedback request
    pub async fn send_haptic_request(&self, pattern: &str, intensity: f32) -> Result<()> {
        let protocol_command = ProtocolEnvelope::command_now(
            0,
            SimulatorCommand::Haptic(HapticCommandPayload::from_legacy_parts(
                pattern, intensity, 0,
            )),
        );

        let message = GesturaMessage {
            message_type: "haptic".to_string(),
            session_id: self.session_id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: serde_json::json!({
                "protocol": protocol_command,
                "legacy": {
                    "pattern": pattern,
                    "intensity": intensity
                }
            }),
        };

        self.send_message(message).await
    }

    /// Send a shared protocol event to gestura.app.
    pub async fn send_protocol_event(&self, event: ProtocolEnvelope<SimulatorEvent>) -> Result<()> {
        let adapter = SocketProtocolAdapter::new(self.session_id.clone());
        let message = adapter.project_event(&event)?;

        self.send_message(message).await
    }

    /// Send a shared protocol command to gestura.app.
    pub async fn send_protocol_command(
        &self,
        command: ProtocolEnvelope<SimulatorCommand>,
    ) -> Result<()> {
        let adapter = SocketProtocolAdapter::new(self.session_id.clone());
        let message = adapter.project_command(&command)?;

        self.send_message(message).await
    }

    /// Send an MCP message
    pub async fn send_mcp_message(&self, mcp_data: serde_json::Value) -> Result<()> {
        let message = GesturaMessage {
            message_type: "mcp".to_string(),
            session_id: self.session_id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: mcp_data,
        };

        self.send_message(message).await
    }

    /// Send a message to gestura.app
    async fn send_message(&self, message: GesturaMessage) -> Result<()> {
        if !self.is_connected() {
            anyhow::bail!("Socket client not connected");
        }

        let response = self
            .client
            .post(format!("{}/api/message", self.base_url))
            .json(&message)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::debug!("Message sent successfully: {}", message.message_type);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to send message: {} - {}", status, error_text);
        }
    }

    /// Poll for messages from gestura.app
    pub async fn poll_messages(&mut self) -> Result<Vec<GesturaMessage>> {
        if !self.is_connected() {
            return Ok(vec![]);
        }

        let url = if let Some(ref session_id) = self.session_id {
            format!("{}/api/poll?session_id={}", self.base_url, session_id)
        } else {
            format!("{}/api/poll", self.base_url)
        };

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let messages: Vec<GesturaMessage> = response.json().await?;
            self.last_activity = Some(Instant::now());
            Ok(messages)
        } else {
            Ok(vec![])
        }
    }

    /// Check connection health
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/api/health", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get connection statistics
    pub fn get_stats(&self) -> SocketStats {
        SocketStats {
            connected_time: self.last_activity.map(|t| t.elapsed()),
            session_id: self.session_id.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Connection for SocketClient {
    async fn connect(&mut self) -> Result<()> {
        if matches!(self.state, ConnectionState::Connected) {
            return Ok(());
        }

        self.state = ConnectionState::Connecting;

        // Test connection
        match self.health_check().await {
            Ok(true) => {
                // Authenticate if required
                if let Err(e) = self.authenticate().await {
                    self.state = ConnectionState::Error(format!("Authentication failed: {e}"));
                    return Err(e);
                }

                // Create message channel
                let (tx, rx) = mpsc::unbounded_channel();
                self.message_tx = Some(tx);
                self.message_rx = Some(rx);

                self.state = ConnectionState::Connected;
                self.last_activity = Some(Instant::now());

                tracing::info!("Socket client connected to {}", self.base_url);
                Ok(())
            }
            Ok(false) => {
                self.state = ConnectionState::Error("Health check failed".to_string());
                anyhow::bail!("gestura.app health check failed");
            }
            Err(e) => {
                self.state = ConnectionState::Error(format!("Connection failed: {e}"));
                Err(e)
            }
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Send disconnect message if authenticated
        if let Some(ref session_id) = self.session_id {
            let disconnect_message = GesturaMessage {
                message_type: "disconnect".to_string(),
                session_id: Some(session_id.clone()),
                timestamp: chrono::Utc::now().timestamp() as u64,
                data: serde_json::json!({}),
            };

            let _ = self.send_message(disconnect_message).await;
        }

        self.state = ConnectionState::Disconnected;
        self.session_id = None;
        self.message_tx = None;
        self.message_rx = None;
        self.last_activity = None;

        tracing::info!("Socket client disconnected");
        Ok(())
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        if !self.is_connected() {
            anyhow::bail!("Socket client not connected");
        }

        // Convert raw data to JSON message
        let message_data = serde_json::json!({
            "raw_data": base64::engine::general_purpose::STANDARD.encode(data)
        });

        let message = GesturaMessage {
            message_type: "raw".to_string(),
            session_id: self.session_id.clone(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: message_data,
        };

        self.send_message(message).await
    }

    async fn receive(&mut self) -> Result<Option<Vec<u8>>> {
        if let Some(ref mut rx) = self.message_rx {
            match rx.try_recv() {
                Ok(data) => Ok(Some(data)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    anyhow::bail!("Socket message channel disconnected");
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

/// Socket connection statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SocketStats {
    pub connected_time: Option<Duration>,
    pub session_id: Option<String>,
    pub base_url: String,
}
