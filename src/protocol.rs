//! Shared semantic protocol types for simulator convergence.
//!
//! This module defines the transport-agnostic event and command shapes that BLE,
//! socket, and MCP adapters can all project from the same semantic source of truth.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::emulator::{GestureEvent, GestureType, HapticPattern, SlideDirection};
use crate::trust::{DegradedMode, TrustState};

/// The current shared simulator protocol version.
///
/// v0.2.0 (2026-07-02): haptic vocabulary ratified as {Confirm, Error, Tick,
/// DoubleTick, Waveform} per user decision. NOTE: per the same decision, the
/// canonical home of this contract is now the gestura-app SDK
/// (`gestura-core-ring/src/protocol.rs`) — this file follows it.
///
/// v0.3.0 (2026-07-02, all proposals approved): new GATT UUID base,
/// first-class waveform payload, swipe/rotate gesture kinds, ack event type.
pub const SHARED_PROTOCOL_VERSION: &str = "0.3.0";

/// Returns the current wall-clock timestamp in milliseconds.
pub fn current_protocol_timestamp_ms() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

/// Identifies whether an envelope carries an event or a command payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolMessageKind {
    /// A host-observable simulator event.
    Event,
    /// A command directed at the simulator.
    Command,
}

/// Versioned envelope shared by all simulator transports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolEnvelope<T> {
    /// Semantic protocol version.
    pub protocol_version: String,
    /// Whether the payload is an event or command.
    pub message_kind: ProtocolMessageKind,
    /// Unique identifier for the message instance.
    pub message_id: String,
    /// Per-session sequence value. A value of `0` means unsequenced.
    pub sequence: u64,
    /// Millisecond timestamp associated with the payload.
    pub timestamp_ms: u64,
    /// Transport-independent payload.
    pub payload: T,
}

impl<T> ProtocolEnvelope<T> {
    /// Creates a new event envelope.
    pub fn event(sequence: u64, timestamp_ms: u64, payload: T) -> Self {
        Self {
            protocol_version: SHARED_PROTOCOL_VERSION.to_string(),
            message_kind: ProtocolMessageKind::Event,
            message_id: uuid::Uuid::new_v4().to_string(),
            sequence,
            timestamp_ms,
            payload,
        }
    }

    /// Creates a new command envelope.
    pub fn command(sequence: u64, timestamp_ms: u64, payload: T) -> Self {
        Self {
            protocol_version: SHARED_PROTOCOL_VERSION.to_string(),
            message_kind: ProtocolMessageKind::Command,
            message_id: uuid::Uuid::new_v4().to_string(),
            sequence,
            timestamp_ms,
            payload,
        }
    }

    /// Creates a new event envelope timestamped at creation time.
    #[allow(dead_code)]
    pub fn event_now(sequence: u64, payload: T) -> Self {
        Self::event(sequence, current_protocol_timestamp_ms(), payload)
    }

    /// Creates a new command envelope timestamped at creation time.
    pub fn command_now(sequence: u64, payload: T) -> Self {
        Self::command(sequence, current_protocol_timestamp_ms(), payload)
    }
}

/// Semantic slide directions preserved across transports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSlideDirection {
    /// Upward slide.
    Up,
    /// Downward slide.
    Down,
    /// Leftward slide.
    Left,
    /// Rightward slide.
    Right,
}

impl From<&SlideDirection> for SemanticSlideDirection {
    fn from(value: &SlideDirection) -> Self {
        match value {
            SlideDirection::Up => Self::Up,
            SlideDirection::Down => Self::Down,
            SlideDirection::Left => Self::Left,
            SlideDirection::Right => Self::Right,
        }
    }
}

/// Horizontal swipe directions (device touch strip is single-axis).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSwipeDirection {
    /// Leftward swipe.
    Left,
    /// Rightward swipe.
    Right,
}

/// Rotation directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRotateDirection {
    /// Clockwise.
    Cw,
    /// Counter-clockwise.
    Ccw,
}

/// Semantic gesture kind shared across adapters.
///
/// v0.3.0: `swipe`/`rotate` are device-truth kinds; `slide`/`tilt` remain
/// simulator-only (produced by the pad/tilt UI, no ring equivalent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "gesture_kind", rename_all = "snake_case")]
pub enum SemanticGesture {
    /// Single tap.
    Tap,
    /// Double tap.
    DoubleTap,
    /// Hold with duration.
    Hold { duration_ms: u64 },
    /// Horizontal swipe (device-truth kind).
    Swipe { direction: SemanticSwipeDirection },
    /// Rotation (device-truth kind).
    Rotate { direction: SemanticRotateDirection },
    /// Slide with direction (simulator-only).
    Slide { direction: SemanticSlideDirection },
    /// Tilt with angle (simulator-only).
    Tilt { angle_degrees: f32 },
}

impl From<&GestureType> for SemanticGesture {
    fn from(value: &GestureType) -> Self {
        match value {
            GestureType::Tap => Self::Tap,
            GestureType::DoubleTap => Self::DoubleTap,
            GestureType::Hold { duration } => Self::Hold {
                duration_ms: duration.as_millis() as u64,
            },
            GestureType::Slide { direction } => Self::Slide {
                direction: direction.into(),
            },
            GestureType::Tilt { angle } => Self::Tilt {
                angle_degrees: *angle,
            },
        }
    }
}

/// Semantic gesture event payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticGestureEvent {
    /// Gesture classification.
    pub gesture: SemanticGesture,
    /// Confidence reported by the simulator.
    pub confidence: f32,
    /// Millisecond timestamp for the gesture.
    pub timestamp_ms: u64,
}

impl SemanticGestureEvent {
    /// Converts a runtime emulator gesture event into its semantic protocol form.
    pub fn from_runtime_event(event: &GestureEvent) -> Self {
        Self {
            gesture: (&event.gesture_type).into(),
            confidence: event.confidence,
            timestamp_ms: event.timestamp.elapsed().as_millis() as u64,
        }
    }

    /// Converts a legacy gesture string and payload into the semantic protocol form.
    pub fn from_legacy_parts(gesture_type: &str, data: &Value, timestamp_ms: u64) -> Result<Self> {
        let normalized = gesture_type.to_ascii_lowercase();
        let gesture = match normalized.as_str() {
            "tap" => SemanticGesture::Tap,
            "double_tap" | "doubletap" => SemanticGesture::DoubleTap,
            "hold" => SemanticGesture::Hold {
                duration_ms: data
                    .get("duration_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(500),
            },
            "slide" => {
                let direction = match data
                    .get("direction")
                    .and_then(Value::as_str)
                    .unwrap_or("up")
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "up" => SemanticSlideDirection::Up,
                    "down" => SemanticSlideDirection::Down,
                    "left" => SemanticSlideDirection::Left,
                    "right" => SemanticSlideDirection::Right,
                    other => {
                        return Err(anyhow!("unsupported legacy slide direction: {other}"));
                    }
                };

                SemanticGesture::Slide { direction }
            }
            "tilt" => SemanticGesture::Tilt {
                angle_degrees: data.get("angle").and_then(Value::as_f64).unwrap_or(0.0) as f32,
            },
            other => return Err(anyhow!("unsupported legacy gesture type: {other}")),
        };

        Ok(Self {
            gesture,
            confidence: data
                .get("confidence")
                .and_then(Value::as_f64)
                .unwrap_or(0.95) as f32,
            timestamp_ms,
        })
    }
}

/// Semantic haptic pattern payload — v0.2.0 ratified vocabulary
/// {Confirm, Error, Tick, DoubleTick, Waveform} (user decision 2026-07-02).
///
/// `success`/`notify` are accepted as read-aliases for v0.1.0 peers but never
/// emitted. `Waveform` travels as the interim `Custom` variant until its
/// first-class payload encoding is ratified (pending firmware cross-check +
/// user confirmation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "pattern_kind", rename_all = "snake_case")]
pub enum SemanticHapticPattern {
    /// Confirmation haptic.
    #[serde(alias = "success")]
    Confirm,
    /// Error haptic.
    Error,
    /// Single tick haptic.
    #[serde(alias = "notify")]
    Tick,
    /// Double tick haptic.
    DoubleTick,
    /// First-class waveform playback (v0.3.0): base64 samples + sample rate.
    Waveform {
        /// Base64-encoded samples.
        data: String,
        /// Sample rate of `data`.
        sample_rate_hz: u32,
        /// Playback intensity.
        intensity: f32,
    },
    /// Custom haptic (generic pulse).
    Custom { intensity: f32, duration_ms: u64 },
}

impl From<&HapticPattern> for SemanticHapticPattern {
    fn from(value: &HapticPattern) -> Self {
        match value {
            // The emulator's internal preset names predate the ratified
            // vocabulary; map them onto it at the protocol boundary.
            HapticPattern::Notify => Self::Tick,
            HapticPattern::Success => Self::Confirm,
            HapticPattern::Error => Self::Error,
            HapticPattern::DoubleTick => Self::DoubleTick,
            HapticPattern::Custom {
                intensity,
                duration,
            } => Self::Custom {
                intensity: *intensity,
                duration_ms: duration.as_millis() as u64,
            },
        }
    }
}

impl From<SemanticHapticPattern> for HapticPattern {
    fn from(value: SemanticHapticPattern) -> Self {
        match value {
            SemanticHapticPattern::Confirm => Self::Success,
            SemanticHapticPattern::Error => Self::Error,
            SemanticHapticPattern::Tick => Self::Notify,
            SemanticHapticPattern::DoubleTick => Self::DoubleTick,
            // The emulator cannot play raw samples; approximate the waveform
            // with a pulse of its true playback duration.
            SemanticHapticPattern::Waveform {
                data,
                sample_rate_hz,
                intensity,
            } => Self::Custom {
                intensity,
                duration: waveform_playback_duration(&data, sample_rate_hz),
            },
            SemanticHapticPattern::Custom {
                intensity,
                duration_ms,
            } => Self::Custom {
                intensity,
                duration: std::time::Duration::from_millis(duration_ms),
            },
        }
    }
}

/// Computes the playback duration of a base64-encoded sample buffer.
/// Falls back to 100 ms when the payload doesn't decode or the rate is zero.
pub fn waveform_playback_duration(data: &str, sample_rate_hz: u32) -> std::time::Duration {
    use base64::Engine as _;
    let sample_count = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map(|samples| samples.len() as u64)
        .unwrap_or(0);
    if sample_count == 0 || sample_rate_hz == 0 {
        return std::time::Duration::from_millis(100);
    }
    std::time::Duration::from_millis(sample_count * 1000 / sample_rate_hz as u64)
}

/// Semantic haptic command payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HapticCommandPayload {
    /// Requested haptic pattern.
    pub pattern: SemanticHapticPattern,
}

impl HapticCommandPayload {
    /// Builds a semantic haptic command from the legacy simulator fields.
    /// Accepts both the v0.2.0 ratified names and the v0.1.0 legacy names.
    pub fn from_legacy_parts(pattern: &str, intensity: f32, duration_ms: u32) -> Self {
        let normalized = pattern.to_ascii_lowercase();
        let pattern = match normalized.as_str() {
            "confirm" | "success" => SemanticHapticPattern::Confirm,
            "error" => SemanticHapticPattern::Error,
            "tick" | "notify" => SemanticHapticPattern::Tick,
            "double_tick" | "doubletick" => SemanticHapticPattern::DoubleTick,
            _ => SemanticHapticPattern::Custom {
                intensity,
                duration_ms: duration_ms as u64,
            },
        };

        Self { pattern }
    }
}

/// Shared battery snapshot payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatterySnapshot {
    /// Battery percentage.
    pub level_percent: u8,
    /// Whether the device is charging.
    pub is_charging: bool,
    /// Current battery voltage.
    pub voltage: f32,
    /// Current battery temperature in Celsius.
    pub temperature_celsius: f32,
    /// Battery health label.
    pub health: String,
    /// Remaining time estimate in minutes.
    pub time_remaining_minutes: Option<u32>,
}

impl BatterySnapshot {
    /// Returns true when the battery level should trigger low-power degraded mode.
    #[allow(dead_code)]
    pub fn is_low_power(&self) -> bool {
        self.level_percent <= 10
    }
}

/// Shared device state snapshot used for transport parity and policy visibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceStateSnapshot {
    /// Battery-related state.
    pub battery: BatterySnapshot,
    /// Current trust state.
    pub trust_state: TrustState,
    /// Active degraded modes.
    pub degraded_modes: Vec<DegradedMode>,
    /// Current firmware revision.
    pub firmware_version: String,
    /// Active shared protocol version.
    pub protocol_version: String,
    /// Optional revocation reason if trust has been revoked.
    pub revocation_reason: Option<String>,
    /// Whether privileged actions are currently enabled.
    pub privileged_actions_enabled: bool,
}

/// Ack status for command acknowledgements (v0.3.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckStatus {
    /// Command accepted and executed.
    Ok,
    /// Command refused by policy (e.g. trust gate).
    Denied,
    /// Command failed.
    Error,
}

/// Command acknowledgement payload (v0.3.0). Correlates via the command
/// envelope's `sequence`, making policy denials visible to the host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AckPayload {
    /// Sequence of the command being acknowledged.
    pub sequence: u64,
    /// Outcome.
    pub status: AckStatus,
    /// Optional human-readable reason (set on denial/error).
    pub reason: Option<String>,
}

/// Transport-agnostic simulator events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_kind", content = "event", rename_all = "snake_case")]
pub enum SimulatorEvent {
    /// Semantic gesture event.
    Gesture(SemanticGestureEvent),
    /// Battery-state snapshot event.
    Battery(BatterySnapshot),
    /// Full device-state snapshot.
    StateSnapshot(DeviceStateSnapshot),
    /// Command acknowledgement (v0.3.0; emission wiring is a follow-up).
    Ack(AckPayload),
}

/// Transport-agnostic simulator commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command_kind", content = "command", rename_all = "snake_case")]
pub enum SimulatorCommand {
    /// Haptic command.
    Haptic(HapticCommandPayload),
}

/// One deterministic scenario step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeterministicScenarioStep {
    /// Delay after the prior step in milliseconds.
    pub delay_ms: u64,
    /// Event payload emitted at this step.
    pub payload: SimulatorEvent,
}

/// A deterministic, transport-agnostic simulator scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeterministicScenario {
    /// Human-readable scenario name.
    pub name: String,
    /// Ordered steps emitted by the scenario.
    pub steps: Vec<DeterministicScenarioStep>,
}

impl DeterministicScenario {
    /// Compiles the scenario into ordered protocol event envelopes.
    #[allow(dead_code)]
    pub fn compile(&self) -> Vec<ProtocolEnvelope<SimulatorEvent>> {
        let mut elapsed_ms = 0_u64;

        self.steps
            .iter()
            .enumerate()
            .map(|(index, step)| {
                elapsed_ms += step.delay_ms;
                ProtocolEnvelope::event(index as u64 + 1, elapsed_ms, step.payload.clone())
            })
            .collect()
    }
}

/// Returns a representative deterministic ring interaction scenario.
#[allow(dead_code)]
pub fn demo_ring_interaction_scenario() -> DeterministicScenario {
    DeterministicScenario {
        name: "demo_ring_interaction".to_string(),
        steps: vec![
            DeterministicScenarioStep {
                delay_ms: 0,
                payload: SimulatorEvent::Battery(BatterySnapshot {
                    level_percent: 85,
                    is_charging: false,
                    voltage: 4.11,
                    temperature_celsius: 25.0,
                    health: "Good".to_string(),
                    time_remaining_minutes: Some(170),
                }),
            },
            DeterministicScenarioStep {
                delay_ms: 75,
                payload: SimulatorEvent::Gesture(SemanticGestureEvent {
                    gesture: SemanticGesture::Tap,
                    confidence: 0.98,
                    timestamp_ms: 75,
                }),
            },
            DeterministicScenarioStep {
                delay_ms: 125,
                payload: SimulatorEvent::Gesture(SemanticGestureEvent {
                    gesture: SemanticGesture::Slide {
                        direction: SemanticSlideDirection::Up,
                    },
                    confidence: 0.93,
                    timestamp_ms: 200,
                }),
            },
        ],
    }
}
