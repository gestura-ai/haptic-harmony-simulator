//! Feedback loop system for real-time simulation
//!
//! This module handles the core feedback loop: Input → Simulation → MCP → Response Display

use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::emulator::{GestureEvent, HapticEvent, InputEvent};
use crate::mcp_mock::{McpEvent, McpMessage, NotificationPriority};

/// Feedback loop events
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FeedbackEvent {
    /// Input received from user
    Input(InputEvent),
    /// Gesture detected
    GestureDetected(GestureEvent),
    /// Haptic feedback generated
    HapticGenerated(HapticEvent),
    /// MCP message received
    McpMessage(McpEvent),
    /// System status update
    StatusUpdate(SystemStatus),
}

/// System status information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemStatus {
    pub is_active: bool,
    pub response_time_ms: u64,
    pub gesture_count: u64,
    pub error_count: u64,
    pub uptime: Duration,
}

/// Performance metrics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PerformanceMetrics {
    pub average_response_time: Duration,
    pub max_response_time: Duration,
    pub min_response_time: Duration,
    pub total_events: u64,
    pub events_per_second: f64,
}

/// Feedback loop configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FeedbackConfig {
    /// Maximum allowed response time (SRS requirement: <50ms)
    pub max_response_time: Duration,
    /// Performance monitoring interval
    pub metrics_interval: Duration,
    /// Enable detailed logging
    pub verbose_logging: bool,
    /// Buffer size for events
    pub event_buffer_size: usize,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            max_response_time: Duration::from_millis(50),
            metrics_interval: Duration::from_secs(10),
            verbose_logging: false,
            event_buffer_size: 1000,
        }
    }
}

/// Main feedback loop system
#[allow(dead_code)]
pub struct FeedbackLoop {
    config: FeedbackConfig,
    event_tx: mpsc::UnboundedSender<FeedbackEvent>,
    event_rx: mpsc::UnboundedReceiver<FeedbackEvent>,
    is_running: bool,
    start_time: Instant,
    metrics: PerformanceMetrics,
    event_history: Vec<(FeedbackEvent, Instant)>,
}

#[allow(dead_code)]
impl FeedbackLoop {
    /// Create new feedback loop
    pub fn new(config: FeedbackConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            config,
            event_tx,
            event_rx,
            is_running: false,
            start_time: Instant::now(),
            metrics: PerformanceMetrics {
                average_response_time: Duration::ZERO,
                max_response_time: Duration::ZERO,
                min_response_time: Duration::from_secs(1),
                total_events: 0,
                events_per_second: 0.0,
            },
            event_history: Vec::new(),
        }
    }

    /// Start the feedback loop
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        self.is_running = true;
        self.start_time = Instant::now();

        tracing::info!(
            "Starting feedback loop with max response time: {:?}",
            self.config.max_response_time
        );

        // Start performance monitoring
        self.start_performance_monitoring().await;

        Ok(())
    }

    /// Stop the feedback loop
    pub async fn stop(&mut self) -> Result<()> {
        self.is_running = false;
        tracing::info!("Feedback loop stopped");
        Ok(())
    }

    /// Send an event to the feedback loop
    pub async fn send_event(&self, event: FeedbackEvent) -> Result<()> {
        self.event_tx.send(event)?;
        Ok(())
    }

    /// Process the next event (non-blocking)
    pub async fn process_next_event(&mut self) -> Result<Option<FeedbackEvent>> {
        match self.event_rx.try_recv() {
            Ok(event) => {
                let start_time = Instant::now();

                // Process the event
                self.handle_event(&event).await?;

                // Record timing
                let processing_time = start_time.elapsed();
                self.update_metrics(processing_time);

                // Store in history
                self.event_history.push((event.clone(), start_time));

                // Trim history if too large
                if self.event_history.len() > self.config.event_buffer_size {
                    self.event_history.remove(0);
                }

                // Check response time requirement
                if processing_time > self.config.max_response_time {
                    tracing::warn!(
                        "Response time exceeded limit: {:?} > {:?}",
                        processing_time,
                        self.config.max_response_time
                    );
                }

                Ok(Some(event))
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                anyhow::bail!("Event channel disconnected");
            }
        }
    }

    /// Handle a specific event
    async fn handle_event(&self, event: &FeedbackEvent) -> Result<()> {
        match event {
            FeedbackEvent::Input(input) => {
                if self.config.verbose_logging {
                    tracing::debug!("Processing input: {:?}", input);
                }
                self.display_input_feedback(input).await?;
            }
            FeedbackEvent::GestureDetected(gesture) => {
                tracing::info!("Gesture detected: {:?}", gesture.gesture_type);
                self.display_gesture_feedback(gesture).await?;
            }
            FeedbackEvent::HapticGenerated(haptic) => {
                tracing::info!("Haptic feedback: {:?}", haptic.pattern);
                self.display_haptic_feedback(haptic).await?;
            }
            FeedbackEvent::McpMessage(mcp_event) => {
                tracing::info!("MCP message: {:?}", mcp_event.message);
                self.display_mcp_feedback(mcp_event).await?;
            }
            FeedbackEvent::StatusUpdate(status) => {
                if self.config.verbose_logging {
                    tracing::debug!("Status update: {:?}", status);
                }
                self.display_status_feedback(status).await?;
            }
        }
        Ok(())
    }

    /// Display input feedback
    async fn display_input_feedback(&self, input: &InputEvent) -> Result<()> {
        match input {
            InputEvent::KeyPress { key } => {
                println!("⌨️  Key pressed: {key}");
            }
            InputEvent::MouseClick { x, y } => {
                println!("🖱️  Mouse click at ({x}, {y})");
            }
            InputEvent::Touch { x, y, pressure } => {
                println!("👆 Touch at ({x}, {y}) with pressure: {pressure:.2}");
            }
        }
        Ok(())
    }

    /// Display gesture feedback
    async fn display_gesture_feedback(&self, gesture: &GestureEvent) -> Result<()> {
        let confidence_bar = "█".repeat((gesture.confidence * 10.0) as usize);
        println!(
            "✋ Gesture: {:?} (confidence: {:.1}% {})",
            gesture.gesture_type,
            gesture.confidence * 100.0,
            confidence_bar
        );
        Ok(())
    }

    /// Display haptic feedback
    async fn display_haptic_feedback(&self, haptic: &HapticEvent) -> Result<()> {
        match &haptic.pattern {
            crate::emulator::HapticPattern::Notify => {
                println!("📳 Vibrate: notify pattern");
            }
            crate::emulator::HapticPattern::Custom {
                intensity,
                duration,
            } => {
                println!("📳 Vibrate: custom (intensity: {intensity:.1}, duration: {duration:?})");
            }
            crate::emulator::HapticPattern::Success => {
                println!("📳 Vibrate: success pattern");
            }
            crate::emulator::HapticPattern::Error => {
                println!("📳 Vibrate: error pattern");
            }
            crate::emulator::HapticPattern::DoubleTick => {
                println!("📳 Vibrate: double-tick pattern");
            }
        }
        Ok(())
    }

    /// Display MCP feedback
    async fn display_mcp_feedback(&self, mcp_event: &McpEvent) -> Result<()> {
        match &mcp_event.message {
            McpMessage::Ready { session_id, .. } => {
                println!("🟢 MCP Ready (session: {session_id})");
            }
            McpMessage::Notification {
                content, priority, ..
            } => {
                let priority_icon = match priority {
                    NotificationPriority::Low => "🔵",
                    NotificationPriority::Normal => "🟡",
                    NotificationPriority::High => "🟠",
                    NotificationPriority::Critical => "🔴",
                };
                println!("{priority_icon} MCP Notification: {content}");
            }
            McpMessage::Response { status, .. } => {
                println!("📨 MCP Response: {status:?}");
            }
            McpMessage::Error { code, message, .. } => {
                println!("❌ MCP Error {code}: {message}");
            }
        }
        Ok(())
    }

    /// Display status feedback
    async fn display_status_feedback(&self, status: &SystemStatus) -> Result<()> {
        println!(
            "📊 Status: {} | Response: {}ms | Gestures: {} | Uptime: {:?}",
            if status.is_active {
                "Active"
            } else {
                "Inactive"
            },
            status.response_time_ms,
            status.gesture_count,
            status.uptime
        );
        Ok(())
    }

    /// Update performance metrics
    fn update_metrics(&mut self, processing_time: Duration) {
        self.metrics.total_events += 1;

        if processing_time > self.metrics.max_response_time {
            self.metrics.max_response_time = processing_time;
        }

        if processing_time < self.metrics.min_response_time {
            self.metrics.min_response_time = processing_time;
        }

        // Update average (simple moving average)
        let total_time = self.metrics.average_response_time.as_nanos() as u64
            * (self.metrics.total_events - 1)
            + processing_time.as_nanos() as u64;
        self.metrics.average_response_time =
            Duration::from_nanos(total_time / self.metrics.total_events);

        // Calculate events per second
        let elapsed = self.start_time.elapsed();
        self.metrics.events_per_second = self.metrics.total_events as f64 / elapsed.as_secs_f64();
    }

    /// Start performance monitoring task
    async fn start_performance_monitoring(&self) {
        let tx = self.event_tx.clone();
        let interval_duration = self.config.metrics_interval;
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);
            let mut gesture_count = 0u64;
            let error_count = 0u64;

            loop {
                interval_timer.tick().await;

                let status = SystemStatus {
                    is_active: true,
                    response_time_ms: 0, // This would be calculated from recent events
                    gesture_count,
                    error_count,
                    uptime: start_time.elapsed(),
                };

                if tx.send(FeedbackEvent::StatusUpdate(status)).is_err() {
                    break;
                }

                gesture_count += 1; // Placeholder increment
            }
        });
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Get event history
    pub fn get_event_history(&self) -> &[(FeedbackEvent, Instant)] {
        &self.event_history
    }

    /// Check if loop is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
