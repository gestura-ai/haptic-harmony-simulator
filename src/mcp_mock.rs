//! MCP (Model Context Protocol) simulation module
//!
//! This module provides mock implementation of MCP protocol for testing
//! and development without requiring real MCP infrastructure.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::protocol::{ProtocolEnvelope, SimulatorEvent};
use crate::transport_adapters::McpProtocolAdapter;

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpMessage {
    /// Ready status message
    Ready { timestamp: u64, session_id: String },
    /// Notification message
    Notification {
        id: String,
        content: String,
        priority: NotificationPriority,
        timestamp: u64,
    },
    /// Response to a request
    Response {
        request_id: String,
        status: ResponseStatus,
        data: Option<serde_json::Value>,
        timestamp: u64,
    },
    /// Error message
    Error {
        code: u32,
        message: String,
        timestamp: u64,
    },
}

/// Notification priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error,
    Timeout,
    InvalidRequest,
}

/// MCP event for internal handling
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct McpEvent {
    pub message: McpMessage,
    pub timestamp: Instant,
}

/// MCP mock server configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct McpConfig {
    /// Session ID for this simulation
    pub session_id: String,
    /// Response delay simulation (ms)
    pub response_delay: Duration,
    /// Enable automatic ready messages
    pub auto_ready: bool,
    /// Ready message interval
    pub ready_interval: Duration,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            response_delay: Duration::from_millis(10),
            auto_ready: true,
            ready_interval: Duration::from_secs(30),
        }
    }
}

/// MCP mock server
#[allow(dead_code)]
pub struct McpMockServer {
    config: McpConfig,
    message_tx: mpsc::UnboundedSender<McpEvent>,
    message_rx: mpsc::UnboundedReceiver<McpEvent>,
    is_running: bool,
    message_history: Vec<McpEvent>,
}

#[allow(dead_code)]
impl McpMockServer {
    /// Create new MCP mock server
    pub fn new(config: McpConfig) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            config,
            message_tx,
            message_rx,
            is_running: false,
            message_history: Vec::new(),
        }
    }

    /// Start the mock server
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        self.is_running = true;

        // Send initial ready message
        self.send_ready_message().await?;

        // Start background tasks if needed
        if self.config.auto_ready {
            self.start_ready_heartbeat().await;
        }

        tracing::info!(
            "MCP mock server started with session ID: {}",
            self.config.session_id
        );
        Ok(())
    }

    /// Stop the mock server
    pub async fn stop(&mut self) -> Result<()> {
        self.is_running = false;
        tracing::info!("MCP mock server stopped");
        Ok(())
    }

    /// Send a ready message
    pub async fn send_ready_message(&self) -> Result<()> {
        let message = McpMessage::Ready {
            timestamp: chrono::Utc::now().timestamp() as u64,
            session_id: self.config.session_id.clone(),
        };

        self.send_message(message).await
    }

    /// Send a notification
    pub async fn send_notification(
        &self,
        content: String,
        priority: NotificationPriority,
    ) -> Result<()> {
        let message = McpMessage::Notification {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            priority,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_message(message).await
    }

    /// Send a response
    pub async fn send_response(
        &self,
        request_id: String,
        status: ResponseStatus,
        data: Option<serde_json::Value>,
    ) -> Result<()> {
        let message = McpMessage::Response {
            request_id,
            status,
            data,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_message(message).await
    }

    /// Send an error message
    pub async fn send_error(&self, code: u32, message: String) -> Result<()> {
        let error_msg = McpMessage::Error {
            code,
            message,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.send_message(error_msg).await
    }

    /// Project a shared simulator event through the MCP notification surface.
    pub async fn send_protocol_event(
        &self,
        event: &ProtocolEnvelope<SimulatorEvent>,
    ) -> Result<()> {
        let adapter = McpProtocolAdapter;
        self.send_message(adapter.project_event(event)?).await
    }

    /// Internal method to send messages
    async fn send_message(&self, message: McpMessage) -> Result<()> {
        let event = McpEvent {
            message,
            timestamp: Instant::now(),
        };

        self.message_tx.send(event)?;
        Ok(())
    }

    /// Start ready message heartbeat
    async fn start_ready_heartbeat(&self) {
        let tx = self.message_tx.clone();
        let interval = self.config.ready_interval;
        let session_id = self.config.session_id.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let message = McpMessage::Ready {
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    session_id: session_id.clone(),
                };

                let event = McpEvent {
                    message,
                    timestamp: Instant::now(),
                };

                if tx.send(event).is_err() {
                    break;
                }
            }
        });
    }

    /// Get next message (non-blocking)
    pub fn try_recv_message(&mut self) -> Option<McpEvent> {
        match self.message_rx.try_recv() {
            Ok(event) => {
                self.message_history.push(event.clone());
                Some(event)
            }
            Err(_) => None,
        }
    }

    /// Get message history
    pub fn get_message_history(&self) -> &[McpEvent] {
        &self.message_history
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

/// Helper function to create a send_notification equivalent
#[allow(dead_code)]
pub async fn send_notification_to_ring(content: String) -> Result<()> {
    // This simulates the Python send_notification tool functionality
    tracing::info!("Sending notification to ring: {}", content);

    // In a real implementation, this would trigger haptic feedback
    // For simulation, we just log it
    println!("🔔 Ring Notification: {content}");

    Ok(())
}
