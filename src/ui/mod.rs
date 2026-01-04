//! User interface modules for the simulation kit
//!
//! This module provides both CLI and optional GUI interfaces for gesture simulation

pub mod cli;
pub mod pad;
pub mod tilt;

use crate::emulator::InputEvent;
use anyhow::Result;

/// Trait for user interface implementations
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait UserInterface: Send + Sync {
    /// Initialize the interface
    async fn initialize(&mut self) -> Result<()>;

    /// Start the interface event loop
    async fn start(&mut self) -> Result<()>;

    /// Stop the interface
    async fn stop(&mut self) -> Result<()>;

    /// Get the next input event (non-blocking)
    async fn get_next_input(&mut self) -> Result<Option<InputEvent>>;

    /// Display a message to the user
    async fn display_message(&self, message: &str) -> Result<()>;

    /// Check if the interface is active
    fn is_active(&self) -> bool;
}

/// UI configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UiConfig {
    /// Interface type to use
    pub interface_type: InterfaceType,
    /// Enable color output
    pub enable_colors: bool,
    /// Show help on startup
    pub show_help: bool,
    /// Refresh rate for UI updates (ms)
    pub refresh_rate: u64,
}

/// Available interface types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InterfaceType {
    /// Command-line interface only
    Cli,
    /// GUI interface (Tauri)
    Gui,
    /// Both CLI and GUI
    Hybrid,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            interface_type: InterfaceType::Cli,
            enable_colors: true,
            show_help: true,
            refresh_rate: 16, // ~60 FPS
        }
    }
}
