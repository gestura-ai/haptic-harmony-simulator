//! Thin transport adapters for the shared simulator protocol.
//!
//! These adapters project one transport-agnostic semantic envelope into BLE,
//! socket, and MCP-specific message shapes without redefining payload meaning.

use anyhow::Result;

use crate::connectivity::ble_peripheral::{BleBatteryData, BleGestureData, ring_uuids};
use crate::connectivity::socket_client::GesturaMessage;
use crate::mcp_mock::{McpMessage, NotificationPriority};
use crate::protocol::{ProtocolEnvelope, SemanticGesture, SimulatorCommand, SimulatorEvent};
use crate::trust::{DegradedMode, TrustState};

/// BLE projection frame generated from a shared semantic envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedBleFrame {
    /// BLE characteristic UUID that should carry the payload.
    pub characteristic_uuid: &'static str,
    /// Encoded transport payload.
    pub payload: Vec<u8>,
}

/// Socket projection helper for shared semantic events and commands.
#[derive(Debug, Clone, Default)]
pub struct SocketProtocolAdapter {
    session_id: Option<String>,
}

impl SocketProtocolAdapter {
    /// Creates a new socket protocol adapter.
    pub fn new(session_id: Option<String>) -> Self {
        Self { session_id }
    }

    /// Projects an event envelope into the socket transport message shape.
    pub fn project_event(
        &self,
        event: &ProtocolEnvelope<SimulatorEvent>,
    ) -> Result<GesturaMessage> {
        Ok(GesturaMessage {
            message_type: "protocol_event".to_string(),
            session_id: self.session_id.clone(),
            timestamp: event.timestamp_ms,
            data: serde_json::to_value(event)?,
        })
    }

    /// Projects a command envelope into the socket transport message shape.
    pub fn project_command(
        &self,
        command: &ProtocolEnvelope<SimulatorCommand>,
    ) -> Result<GesturaMessage> {
        Ok(GesturaMessage {
            message_type: "protocol_command".to_string(),
            session_id: self.session_id.clone(),
            timestamp: command.timestamp_ms,
            data: serde_json::to_value(command)?,
        })
    }
}

/// MCP projection helper for shared semantic events.
#[derive(Debug, Clone, Default)]
pub struct McpProtocolAdapter;

impl McpProtocolAdapter {
    /// Projects an event envelope into an MCP notification message.
    pub fn project_event(&self, event: &ProtocolEnvelope<SimulatorEvent>) -> Result<McpMessage> {
        Ok(McpMessage::Notification {
            id: uuid::Uuid::new_v4().to_string(),
            content: serde_json::to_string(event)?,
            priority: notification_priority_for_event(&event.payload),
            timestamp: event.timestamp_ms,
        })
    }
}

/// BLE projection helper for shared semantic events and commands.
#[derive(Debug, Clone, Default)]
pub struct BleProtocolAdapter;

impl BleProtocolAdapter {
    /// Projects an event envelope into a BLE characteristic frame.
    pub fn project_event(
        &self,
        event: &ProtocolEnvelope<SimulatorEvent>,
    ) -> Result<ProjectedBleFrame> {
        match &event.payload {
            SimulatorEvent::Gesture(gesture) => {
                let ble_gesture = BleGestureData {
                    gesture_type: legacy_gesture_label(&gesture.gesture).to_string(),
                    timestamp: event.timestamp_ms,
                    confidence: gesture.confidence,
                    data: serde_json::to_vec(event)?,
                };

                Ok(ProjectedBleFrame {
                    characteristic_uuid: ring_uuids::GESTURE_EVENT_UUID,
                    payload: serde_json::to_vec(&ble_gesture)?,
                })
            }
            SimulatorEvent::Battery(snapshot) => {
                let ble_battery = BleBatteryData {
                    level: snapshot.level_percent,
                    is_charging: snapshot.is_charging,
                    voltage: snapshot.voltage,
                    temperature: snapshot.temperature_celsius,
                    health: snapshot.health.clone(),
                    time_remaining: snapshot.time_remaining_minutes,
                };

                Ok(ProjectedBleFrame {
                    characteristic_uuid: ring_uuids::BATTERY_LEVEL_UUID,
                    payload: serde_json::to_vec(&ble_battery)?,
                })
            }
            SimulatorEvent::StateSnapshot(snapshot) => Ok(ProjectedBleFrame {
                characteristic_uuid: ring_uuids::STATE_SNAPSHOT_UUID,
                payload: serde_json::to_vec(snapshot)?,
            }),
        }
    }

    /// Projects a command envelope into a BLE characteristic frame.
    pub fn project_command(
        &self,
        command: &ProtocolEnvelope<SimulatorCommand>,
    ) -> Result<ProjectedBleFrame> {
        Ok(ProjectedBleFrame {
            characteristic_uuid: ring_uuids::HAPTIC_COMMAND_UUID,
            payload: serde_json::to_vec(command)?,
        })
    }
}

/// Combined projection set for parity testing and shared fan-out.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProtocolProjectionSet {
    /// BLE projection.
    pub ble: ProjectedBleFrame,
    /// Socket projection.
    pub socket: GesturaMessage,
    /// MCP projection.
    pub mcp: McpMessage,
}

/// Shared adapter hub that fans one semantic event out to all transport projections.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ProtocolAdapterHub {
    ble: BleProtocolAdapter,
    socket: SocketProtocolAdapter,
    mcp: McpProtocolAdapter,
}

impl ProtocolAdapterHub {
    /// Creates a new hub with an optional socket session identifier.
    #[allow(dead_code)]
    pub fn new(socket_session_id: Option<String>) -> Self {
        Self {
            ble: BleProtocolAdapter,
            socket: SocketProtocolAdapter::new(socket_session_id),
            mcp: McpProtocolAdapter,
        }
    }

    /// Fans one semantic event into all supported transport projections.
    #[allow(dead_code)]
    pub fn fan_out_event(
        &self,
        event: &ProtocolEnvelope<SimulatorEvent>,
    ) -> Result<ProtocolProjectionSet> {
        Ok(ProtocolProjectionSet {
            ble: self.ble.project_event(event)?,
            socket: self.socket.project_event(event)?,
            mcp: self.mcp.project_event(event)?,
        })
    }
}

fn legacy_gesture_label(gesture: &SemanticGesture) -> &'static str {
    match gesture {
        SemanticGesture::Tap => "tap",
        SemanticGesture::DoubleTap => "double_tap",
        SemanticGesture::Hold { .. } => "hold",
        SemanticGesture::Slide { .. } => "slide",
        SemanticGesture::Tilt { .. } => "tilt",
    }
}

fn notification_priority_for_event(event: &SimulatorEvent) -> NotificationPriority {
    match event {
        SimulatorEvent::Battery(snapshot) if snapshot.level_percent <= 10 => {
            NotificationPriority::High
        }
        SimulatorEvent::StateSnapshot(snapshot) => {
            if snapshot.trust_state == TrustState::Revoked {
                NotificationPriority::Critical
            } else if snapshot.degraded_modes.contains(&DegradedMode::LowBattery)
                || snapshot.degraded_modes.contains(&DegradedMode::SensorFault)
                || snapshot
                    .degraded_modes
                    .contains(&DegradedMode::FirmwareMismatch)
            {
                NotificationPriority::High
            } else {
                NotificationPriority::Normal
            }
        }
        _ => NotificationPriority::Normal,
    }
}
