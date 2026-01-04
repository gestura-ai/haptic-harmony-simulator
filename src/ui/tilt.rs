//! Tilt gesture simulation

use anyhow::Result;
use tokio::time::Instant;

use crate::emulator::{GestureEvent, GestureType, InputEvent};

/// Tilt simulator for tilt gesture detection
#[allow(dead_code)]
pub struct TiltSimulator {
    current_angle: f32,
    max_angle: f32,
    sensitivity: f32,
}

#[allow(dead_code)]
impl TiltSimulator {
    /// Create new tilt simulator
    pub fn new() -> Self {
        Self {
            current_angle: 0.0,
            max_angle: 45.0,
            sensitivity: 1.0,
        }
    }

    /// Create with custom configuration
    pub fn with_config(max_angle: f32, sensitivity: f32) -> Self {
        Self {
            current_angle: 0.0,
            max_angle,
            sensitivity,
        }
    }

    /// Process input and detect tilt gestures
    pub fn process_input(&mut self, input: &InputEvent) -> Result<Option<GestureEvent>> {
        let now = Instant::now();

        match input {
            InputEvent::KeyPress { key } => {
                if key == "t" {
                    self.simulate_tilt(now)
                } else {
                    Ok(None)
                }
            }
            InputEvent::MouseClick { x, y } => {
                // Convert mouse position to tilt angle
                let angle = self.calculate_angle_from_position(*x, *y);
                self.handle_tilt(angle, now)
            }
            InputEvent::Touch { x, y, pressure: _ } => {
                // Convert touch position to tilt angle
                let angle = self.calculate_angle_from_position(*x, *y);
                self.handle_tilt(angle, now)
            }
        }
    }

    /// Simulate a tilt gesture
    fn simulate_tilt(&mut self, now: Instant) -> Result<Option<GestureEvent>> {
        // Cycle through different tilt angles for demonstration
        let angles = [10.0, 20.0, 30.0, -10.0, -20.0, -30.0];
        let index = (now.elapsed().as_secs() % angles.len() as u64) as usize;
        let angle = angles[index];

        self.handle_tilt(angle, now)
    }

    /// Handle tilt with specific angle
    fn handle_tilt(&mut self, angle: f32, now: Instant) -> Result<Option<GestureEvent>> {
        // Apply sensitivity
        let adjusted_angle = angle * self.sensitivity;

        // Clamp to max angle
        let clamped_angle = adjusted_angle.clamp(-self.max_angle, self.max_angle);

        // Only trigger if angle is significant enough
        if clamped_angle.abs() >= 5.0 {
            self.current_angle = clamped_angle;

            // Calculate confidence based on angle magnitude
            let confidence = (clamped_angle.abs() / self.max_angle).min(1.0);

            return Ok(Some(GestureEvent {
                gesture_type: GestureType::Tilt {
                    angle: clamped_angle,
                },
                timestamp: now,
                confidence,
            }));
        }

        Ok(None)
    }

    /// Calculate angle from screen position
    fn calculate_angle_from_position(&self, x: u16, y: u16) -> f32 {
        // Assume screen center is at (40, 12) for terminal
        let center_x = 40.0;
        let center_y = 12.0;

        let dx = x as f32 - center_x;
        let _dy = y as f32 - center_y;

        // Convert to angle (simplified)
        let angle = (dx / center_x) * self.max_angle;
        angle.clamp(-self.max_angle, self.max_angle)
    }

    /// Get current tilt angle
    pub fn get_current_angle(&self) -> f32 {
        self.current_angle
    }

    /// Set maximum tilt angle
    pub fn set_max_angle(&mut self, max_angle: f32) {
        self.max_angle = max_angle.abs();
    }

    /// Set sensitivity
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.1, 2.0);
    }

    /// Check if currently tilted
    pub fn is_tilted(&self) -> bool {
        self.current_angle.abs() >= 5.0
    }

    /// Get tilt direction
    pub fn get_tilt_direction(&self) -> TiltDirection {
        if self.current_angle > 5.0 {
            TiltDirection::Right
        } else if self.current_angle < -5.0 {
            TiltDirection::Left
        } else {
            TiltDirection::Neutral
        }
    }

    /// Reset tilt state
    pub fn reset(&mut self) {
        self.current_angle = 0.0;
    }
}

/// Tilt direction
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TiltDirection {
    Left,
    Right,
    Neutral,
}

impl Default for TiltSimulator {
    fn default() -> Self {
        Self::new()
    }
}
