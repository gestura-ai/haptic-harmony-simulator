//! Command-line interface implementation

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;

use super::{UiConfig, UserInterface};
use crate::emulator::InputEvent;

/// CLI interface implementation
#[allow(dead_code)]
pub struct CliInterface {
    config: UiConfig,
    is_active: bool,
    is_raw_mode: bool,
}

#[allow(dead_code)]
impl CliInterface {
    /// Create new CLI interface
    pub fn new(config: UiConfig) -> Self {
        Self {
            config,
            is_active: false,
            is_raw_mode: false,
        }
    }

    /// Display help information
    pub fn show_help(&self) -> Result<()> {
        println!("🎮 Gestura Ring Simulation Kit - CLI Interface");
        println!("===============================================");
        println!();
        println!("Gesture Controls:");
        println!("  Enter       - Tap gesture");
        println!("  Space       - Hold gesture");
        println!("  t           - Tilt gesture");
        println!("  s           - Slide gesture");
        println!("  d           - Double tap gesture");
        println!("  Ctrl+C      - Exit simulation");
        println!();
        println!("Status:");
        println!("  🟢 Ready    - System ready for input");
        println!("  📳 Vibrate  - Haptic feedback");
        println!("  ✋ Gesture  - Gesture detected");
        println!("  🔔 MCP      - MCP message");
        println!();
        Ok(())
    }

    /// Process keyboard input
    fn process_key_event(&self, key_event: KeyEvent) -> Option<InputEvent> {
        match key_event {
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::KeyPress {
                key: "Enter".to_string(),
            }),
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::KeyPress {
                key: "Space".to_string(),
            }),
            KeyEvent {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::KeyPress {
                key: "t".to_string(),
            }),
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::KeyPress {
                key: "s".to_string(),
            }),
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(InputEvent::KeyPress {
                key: "d".to_string(),
            }),
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                // Handle Ctrl+C gracefully
                println!("\n👋 Shutting down simulation...");
                std::process::exit(0);
            }
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl UserInterface for CliInterface {
    async fn initialize(&mut self) -> Result<()> {
        if self.config.show_help {
            self.show_help()?;
        }

        enable_raw_mode()?;
        self.is_raw_mode = true;

        println!("🟢 CLI interface initialized. Ready for input...");
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        if !self.is_raw_mode {
            self.initialize().await?;
        }

        self.is_active = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.is_active = false;

        if self.is_raw_mode {
            disable_raw_mode()?;
            self.is_raw_mode = false;
        }

        println!("CLI interface stopped");
        Ok(())
    }

    async fn get_next_input(&mut self) -> Result<Option<InputEvent>> {
        if !self.is_active {
            return Ok(None);
        }

        // Check for input with timeout
        if event::poll(Duration::from_millis(self.config.refresh_rate))?
            && let Event::Key(key_event) = event::read()?
        {
            return Ok(self.process_key_event(key_event));
        }

        Ok(None)
    }

    async fn display_message(&self, message: &str) -> Result<()> {
        println!("{}", message);
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Drop for CliInterface {
    fn drop(&mut self) {
        if self.is_raw_mode {
            let _ = disable_raw_mode();
        }
    }
}
