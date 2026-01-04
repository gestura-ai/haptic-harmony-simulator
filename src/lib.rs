//! # Haptic Harmony Simulation Library
//!
//! A comprehensive Rust library for simulating haptic feedback rings with BLE peripheral emulation,
//! gesture recognition, and real-time feedback systems.
//!
//! ## Features
//!
//! - **BLE Peripheral Simulation** - Complete Bluetooth Low Energy GATT server
//! - **Gesture Emulation** - Tap, slide, tilt, and custom gesture recognition
//! - **Haptic Feedback** - Pattern generation and real-time feedback
//! - **MCP Integration** - Model Context Protocol support
//! - **GUI/CLI Interfaces** - Both graphical and command-line interfaces
//! - **Real-time Monitoring** - Performance metrics and system status
//!
//! ## Quick Start
//!
//! ```rust
//! use haptic_harmony_simulation::*;
//! use haptic_harmony_simulation::feedback::*;
//! use haptic_harmony_simulation::emulator::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create feedback loop configuration
//!     let config = FeedbackConfig::default();
//!
//!     // Start the feedback loop
//!     let mut feedback_loop = FeedbackLoop::new(config);
//!     feedback_loop.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod connectivity;
pub mod emulator;
pub mod feedback;
pub mod mcp_mock;
pub mod ring_specs;
pub mod ui;

#[cfg(feature = "tauri-gui")]
pub mod tauri_app;

// Re-export commonly used types
pub use connectivity::*;
pub use emulator::HapticPattern as EmulatorHapticPattern;
pub use emulator::{
    EmulatorState, GestureConfig, GestureEmulator, GestureEvent, GestureType, HapticConfig,
    HapticEmulator, HapticEvent, InputEvent, SlideDirection,
};
pub use feedback::*;
pub use mcp_mock::*;
pub use ring_specs::HapticPattern as RingHapticPattern;
pub use ring_specs::{RingSpec, RingSpecManager, RingType};
pub use ui::*;
