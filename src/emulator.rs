//! Core emulation engine for gesture and haptic simulation
//!
//! This module provides the foundational traits and types for emulating
//! the Haptic Harmony Ring's behavior without physical hardware.

use anyhow::Result;
use std::time::Duration;
use tokio::time::Instant;

/// Types of gestures that can be emulated
#[derive(Debug, Clone, PartialEq)]
pub enum GestureType {
    /// Single tap gesture
    Tap,
    /// Double tap gesture with timing validation
    DoubleTap,
    /// Hold gesture with duration
    Hold { duration: Duration },
    /// Slide gesture with direction
    Slide { direction: SlideDirection },
    /// Tilt gesture with angle (degrees)
    Tilt { angle: f32 },
}

/// Direction for slide gestures
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Haptic feedback patterns
#[derive(Debug, Clone, PartialEq)]
pub enum HapticPattern {
    /// Standard notification vibration
    Notify,
    /// Custom vibration with intensity and duration
    Custom { intensity: f32, duration: Duration },
    /// Success feedback pattern
    Success,
    /// Error feedback pattern
    Error,
}

/// Gesture event with timing information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GestureEvent {
    pub gesture_type: GestureType,
    pub timestamp: Instant,
    pub confidence: f32,
}

/// Haptic feedback event
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HapticEvent {
    pub pattern: HapticPattern,
    pub timestamp: Instant,
}

/// Trait for gesture emulation
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait GestureEmulator: Send + Sync {
    /// Process input and detect gestures
    async fn process_input(&mut self, input: InputEvent) -> Result<Option<GestureEvent>>;

    /// Validate gesture timing and constraints
    fn validate_gesture(&self, gesture: &GestureType) -> bool;

    /// Get current emulator state
    fn get_state(&self) -> EmulatorState;
}

/// Trait for haptic feedback emulation
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait HapticEmulator: Send + Sync {
    /// Generate haptic feedback
    async fn generate_feedback(&mut self, pattern: HapticPattern) -> Result<HapticEvent>;

    /// Check if haptic feedback is supported
    fn supports_pattern(&self, pattern: &HapticPattern) -> bool;
}

/// Input event types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InputEvent {
    KeyPress { key: String },
    MouseClick { x: u16, y: u16 },
    Touch { x: u16, y: u16, pressure: f32 },
}

/// Current state of the emulator
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EmulatorState {
    pub is_active: bool,
    pub last_gesture: Option<GestureEvent>,
    pub gesture_count: u64,
    pub session_start: Instant,
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self {
            is_active: false,
            last_gesture: None,
            gesture_count: 0,
            session_start: Instant::now(),
        }
    }
}

/// Configuration for gesture emulation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GestureConfig {
    /// Minimum time between taps for double-tap detection (ms)
    pub double_tap_threshold: Duration,
    /// Minimum hold duration (ms)
    pub hold_threshold: Duration,
    /// Gesture sensitivity (0.0 - 1.0)
    pub sensitivity: f32,
    /// Maximum tilt angle for detection (degrees)
    pub max_tilt_angle: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            double_tap_threshold: Duration::from_millis(300),
            hold_threshold: Duration::from_millis(500),
            sensitivity: 0.8,
            max_tilt_angle: 45.0,
        }
    }
}

/// Configuration for haptic feedback
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HapticConfig {
    /// Default intensity (0.0 - 1.0)
    pub default_intensity: f32,
    /// Maximum vibration duration
    pub max_duration: Duration,
    /// Enable/disable haptic feedback
    pub enabled: bool,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            default_intensity: 0.7,
            max_duration: Duration::from_secs(2),
            enabled: true,
        }
    }
}
