//! PAD (Pressure Area Device) simulation

use anyhow::Result;
use std::time::Duration;
use tokio::time::Instant;

use crate::emulator::{GestureEvent, GestureType, InputEvent, SlideDirection};

/// PAD simulator for gesture detection
#[allow(dead_code)]
pub struct PadSimulator {
    last_input_time: Option<Instant>,
    hold_start_time: Option<Instant>,
    tap_count: u32,
    last_tap_time: Option<Instant>,
    is_holding: bool,
    double_tap_threshold: Duration,
    hold_threshold: Duration,
}

#[allow(dead_code)]
impl PadSimulator {
    /// Create new PAD simulator
    pub fn new() -> Self {
        Self {
            last_input_time: None,
            hold_start_time: None,
            tap_count: 0,
            last_tap_time: None,
            is_holding: false,
            double_tap_threshold: Duration::from_millis(300),
            hold_threshold: Duration::from_millis(500),
        }
    }

    /// Process input and detect gestures
    pub fn process_input(&mut self, input: &InputEvent) -> Result<Option<GestureEvent>> {
        let now = Instant::now();

        match input {
            InputEvent::KeyPress { key } => match key.as_str() {
                "Enter" => self.handle_tap(now),
                "Space" => self.handle_hold_start(now),
                "s" => self.handle_slide(now),
                _ => Ok(None),
            },
            InputEvent::MouseClick { x: _, y: _ } => self.handle_tap(now),
            InputEvent::Touch {
                x: _,
                y: _,
                pressure,
            } => {
                if *pressure > 0.5 {
                    self.handle_hold_start(now)
                } else {
                    self.handle_tap(now)
                }
            }
        }
    }

    /// Handle tap gesture
    fn handle_tap(&mut self, now: Instant) -> Result<Option<GestureEvent>> {
        // Check for double tap
        if let Some(last_tap) = self.last_tap_time {
            if now.duration_since(last_tap) <= self.double_tap_threshold {
                self.tap_count += 1;
                if self.tap_count >= 2 {
                    self.tap_count = 0;
                    self.last_tap_time = None;
                    return Ok(Some(GestureEvent {
                        gesture_type: GestureType::DoubleTap,
                        timestamp: now,
                        confidence: 0.9,
                    }));
                }
            } else {
                // Too much time passed, reset
                self.tap_count = 1;
            }
        } else {
            self.tap_count = 1;
        }

        self.last_tap_time = Some(now);
        self.last_input_time = Some(now);

        // Return single tap for now, might become double tap later
        Ok(Some(GestureEvent {
            gesture_type: GestureType::Tap,
            timestamp: now,
            confidence: 0.8,
        }))
    }

    /// Handle hold gesture start
    fn handle_hold_start(&mut self, now: Instant) -> Result<Option<GestureEvent>> {
        if !self.is_holding {
            self.hold_start_time = Some(now);
            self.is_holding = true;
        }

        self.last_input_time = Some(now);
        Ok(None) // Hold gesture is detected on release
    }

    /// Handle hold gesture end
    pub fn handle_hold_end(&mut self, now: Instant) -> Result<Option<GestureEvent>> {
        if let Some(start_time) = self.hold_start_time {
            let duration = now.duration_since(start_time);

            if duration >= self.hold_threshold {
                self.is_holding = false;
                self.hold_start_time = None;

                return Ok(Some(GestureEvent {
                    gesture_type: GestureType::Hold { duration },
                    timestamp: now,
                    confidence: 0.9,
                }));
            }
        }

        self.is_holding = false;
        self.hold_start_time = None;
        Ok(None)
    }

    /// Handle slide gesture
    fn handle_slide(&mut self, now: Instant) -> Result<Option<GestureEvent>> {
        // For simplicity, we'll cycle through slide directions
        let direction = match (now.elapsed().as_secs()) % 4 {
            0 => SlideDirection::Up,
            1 => SlideDirection::Down,
            2 => SlideDirection::Left,
            _ => SlideDirection::Right,
        };

        self.last_input_time = Some(now);

        Ok(Some(GestureEvent {
            gesture_type: GestureType::Slide { direction },
            timestamp: now,
            confidence: 0.8,
        }))
    }

    /// Check for timeout on ongoing gestures
    pub fn check_timeouts(&mut self, now: Instant) -> Result<Option<GestureEvent>> {
        // Check if we should finalize a hold gesture
        if self.is_holding
            && let Some(start_time) = self.hold_start_time
        {
            let duration = now.duration_since(start_time);
            if duration >= Duration::from_secs(3) {
                // Auto-release after 3 seconds
                return self.handle_hold_end(now);
            }
        }

        // Check for double tap timeout
        if let Some(last_tap) = self.last_tap_time
            && now.duration_since(last_tap) > self.double_tap_threshold
            && self.tap_count == 1
        {
            // Single tap confirmed
            self.tap_count = 0;
            self.last_tap_time = None;
        }

        Ok(None)
    }

    /// Get current state
    pub fn is_holding(&self) -> bool {
        self.is_holding
    }

    /// Get hold duration if currently holding
    pub fn get_hold_duration(&self) -> Option<Duration> {
        self.hold_start_time
            .map(|start_time| Instant::now().duration_since(start_time))
    }
}

impl Default for PadSimulator {
    fn default() -> Self {
        Self::new()
    }
}
