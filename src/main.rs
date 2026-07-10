//! Gestura Ring Simulation Kit
//!
//! A standalone Rust application that emulates the Haptic Harmony Ring's behavior
//! for developer testing and solution creation.

mod connectivity;
// The bin compiles its own module tree; it uses a subset of the device-core
// API (the lib exports all of it), so silence dead-code for this copy only.
#[cfg(feature = "device-core")]
#[allow(dead_code)]
mod device_core;
mod emulator;
mod feedback;
mod mcp_mock;
mod protocol;
mod ring_specs;
mod transport_adapters;
mod trust;
mod ui;

#[cfg(feature = "tauri-gui")]
mod tauri_app;

use anyhow::Result;
use clap::Parser;
#[cfg(feature = "cli-only")]
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
#[cfg(feature = "cli-only")]
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, info};

use crate::connectivity::*;
use crate::emulator::*;
use crate::feedback::*;
use crate::mcp_mock::*;
use crate::ui::*;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "gestura-ring-sim")]
#[command(about = "Gestura Ring Simulation Kit - Emulate Haptic Harmony Ring behavior")]
#[command(version = "0.1.0")]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Interface mode (gui, cli)
    #[arg(short, long, default_value = "gui")]
    mode: String,

    /// Host for gestura.app connection
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Port for gestura.app connection
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Ring type to simulate (b1, a1, p1)
    #[arg(long, default_value = "b1")]
    ring_type: String,
}

/// Main application configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
#[derive(Default)]
struct AppConfig {
    pub gesture_config: GestureConfig,
    pub haptic_config: HapticConfig,
    pub mcp_config: McpConfig,
    pub feedback_config: FeedbackConfig,
    pub ui_config: UiConfig,
    pub connection_config: ConnectionConfig,
}

/// Main simulation application
#[allow(dead_code)]
struct SimulationApp {
    config: AppConfig,
    feedback_loop: FeedbackLoop,
    connection_manager: ConnectionManager,
    ble_peripheral: Option<BlePeripheral>,
    gesture_tx: Option<mpsc::UnboundedSender<GestureEvent>>,
    gesture_rx: Option<mpsc::UnboundedReceiver<GestureEvent>>,
    is_running: bool,
}

impl SimulationApp {
    /// Create new simulation application
    #[allow(dead_code)]
    fn new(config: AppConfig) -> Self {
        let feedback_loop = FeedbackLoop::new(config.feedback_config.clone());
        let connection_manager = ConnectionManager::new(config.connection_config.clone());

        // Create gesture channel
        let (gesture_tx, gesture_rx) = mpsc::unbounded_channel();

        Self {
            config,
            feedback_loop,
            connection_manager,
            ble_peripheral: None,
            gesture_tx: Some(gesture_tx),
            gesture_rx: Some(gesture_rx),
            is_running: false,
        }
    }

    /// Start the simulation
    #[allow(dead_code)]
    async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        info!("Starting Gestura Simulation Kit...");

        // Start feedback loop
        self.feedback_loop.start().await?;
        info!("Feedback loop started");

        // Initialize connections
        self.connection_manager.init_socket().await?;
        info!("Connection manager initialized");

        // Initialize and start BLE peripheral
        let mut ble_peripheral = BlePeripheral::new(self.config.connection_config.clone())?;

        // Connect gesture channel to BLE peripheral
        if let Some(gesture_rx) = self.gesture_rx.take() {
            ble_peripheral.set_gesture_receiver(gesture_rx);
        }

        ble_peripheral.start_advertising().await?;
        self.ble_peripheral = Some(ble_peripheral);
        info!("🔵 BLE peripheral started - now discoverable by gestura.app");

        self.is_running = true;
        info!("Simulation kit is now running");

        Ok(())
    }

    /// Stop the simulation
    #[cfg(feature = "cli-only")]
    async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }

        info!("Stopping Gestura Ring Simulation Kit...");

        // Stop all components
        if let Some(ref mut ble_peripheral) = self.ble_peripheral {
            ble_peripheral.stop_advertising().await?;
            info!("BLE peripheral stopped");
        }
        self.feedback_loop.stop().await?;
        self.connection_manager.disconnect().await?;

        self.is_running = false;

        // Disable raw mode
        disable_raw_mode()?;

        info!("Simulation kit stopped");

        Ok(())
    }

    /// Handle keyboard input and generate gesture events
    #[cfg(feature = "cli-only")]
    async fn handle_key_input(&self, key_event: KeyEvent) -> Result<bool> {
        let gesture_type = match key_event.code {
            KeyCode::Enter => {
                info!("⌨️ Key pressed: Enter → Tap gesture");
                Some(GestureType::Tap)
            }
            KeyCode::Char(' ') => {
                info!("⌨️ Key pressed: Space → Hold gesture");
                Some(GestureType::Hold {
                    duration: Duration::from_millis(500),
                })
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                info!("⌨️ Key pressed: T → Tilt gesture");
                Some(GestureType::Tilt { angle: 45.0 })
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                info!("⌨️ Key pressed: S → Slide gesture");
                Some(GestureType::Slide {
                    direction: SlideDirection::Up,
                })
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                info!("⌨️ Key pressed: D → Double Tap gesture");
                Some(GestureType::DoubleTap)
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                info!("⌨️ Key pressed: H → Simulate haptic command");
                // Simulate receiving a haptic command from gestura.app
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral
                        .simulate_haptic_command("notify", 0.8, 300)
                        .await;
                }
                None // No gesture to send
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                info!("⌨️ Key pressed: B → Simulate buzz haptic");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral
                        .simulate_haptic_command("buzz", 1.0, 500)
                        .await;
                }
                None
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                info!("⌨️ Key pressed: P → Simulate pulse haptic");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral
                        .simulate_haptic_command("pulse", 0.6, 800)
                        .await;
                }
                None
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                info!("⌨️ Key pressed: C → Toggle charging");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral.toggle_charging().await;
                }
                None
            }
            KeyCode::Char('1') => {
                info!("⌨️ Key pressed: 1 → Set battery to 10%");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral.set_battery_level(10).await;
                }
                None
            }
            KeyCode::Char('2') => {
                info!("⌨️ Key pressed: 2 → Set battery to 50%");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral.set_battery_level(50).await;
                }
                None
            }
            KeyCode::Char('3') => {
                info!("⌨️ Key pressed: 3 → Set battery to 90%");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let _ = ble_peripheral.set_battery_level(90).await;
                }
                None
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                info!("⌨️ Key pressed: I → Battery info");
                if let Some(ref ble_peripheral) = self.ble_peripheral {
                    let status = ble_peripheral.get_battery_status().await;
                    info!("🔋 Battery Status:");
                    info!("   Level: {}%", status.level);
                    info!(
                        "   Charging: {}",
                        if status.is_charging { "Yes" } else { "No" }
                    );
                    info!("   Voltage: {:.2}V", status.voltage);
                    info!("   Temperature: {:.1}°C", status.temperature);
                    info!("   Health: {}", status.health);
                    if let Some(time) = status.time_remaining {
                        info!("   Time remaining: {} minutes", time);
                    }
                }
                None
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                info!("⌨️ Q pressed - exiting...");
                return Ok(true); // Exit requested
            }
            KeyCode::Esc => {
                info!("⌨️ Escape pressed - exiting...");
                return Ok(true); // Exit requested
            }
            _ => {
                // Ignore other keys
                None
            }
        };

        // Send gesture event if recognized
        if let Some(gesture) = gesture_type {
            let gesture_event = GestureEvent {
                gesture_type: gesture,
                timestamp: tokio::time::Instant::now(),
                confidence: 0.95, // High confidence for keyboard input
            };

            if let Some(ref gesture_tx) = self.gesture_tx {
                if let Err(e) = gesture_tx.send(gesture_event) {
                    error!("Failed to send gesture event: {}", e);
                } else {
                    info!("✋ Gesture event sent to BLE peripheral");
                }
            }
        }

        Ok(false) // Continue running
    }

    /// Start BLE peripheral for GUI mode
    #[cfg(feature = "tauri-gui")]
    async fn start_ble_peripheral(&mut self) -> Result<()> {
        self.start().await
    }

    /// Run the main simulation loop
    #[cfg(feature = "cli-only")]
    async fn run(&mut self) -> Result<()> {
        self.start().await?;

        info!("🎮 Simulation loop started. Press Ctrl+C to exit.");
        info!("🔵 BLE Status: Advertising as 'Haptic Harmony Ring Simulator'");
        info!("📱 gestura.app can now discover and connect to this simulator");
        info!("");
        info!("Available gestures:");
        info!("  Enter - Tap");
        info!("  Space - Hold");
        info!("  t - Tilt");
        info!("  s - Slide");
        info!("  d - Double Tap");
        info!("");
        info!("Haptic command simulation:");
        info!("  h - Notify haptic");
        info!("  b - Buzz haptic");
        info!("  p - Pulse haptic");
        info!("");
        info!("Battery simulation:");
        info!("  c - Toggle charging");
        info!("  1 - Set battery to 10%");
        info!("  2 - Set battery to 50%");
        info!("  3 - Set battery to 90%");
        info!("  i - Battery info");

        // Enable raw mode for keyboard input
        enable_raw_mode()?;

        // Main event loop
        loop {
            // Handle keyboard input
            if event::poll(Duration::from_millis(1))?
                && let Event::Key(key_event) = event::read()?
                && key_event.kind == KeyEventKind::Press
                && self.handle_key_input(key_event).await?
            {
                break; // Exit requested
            }

            // Process feedback events
            if let Some(_event) = self.feedback_loop.process_next_event().await? {
                // Event processed
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(1)).await;

            // Check for shutdown signal (simplified for now)
            // In a real implementation, this would handle Ctrl+C properly
        }

        self.stop().await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Gestura Ring Simulation Kit v0.1.0");
    info!("Emulating Haptic Harmony Ring behavior");

    // Create application configuration
    let mut config = AppConfig::default();
    config.connection_config.host = args.host;
    config.connection_config.port = args.port;
    config.feedback_config.verbose_logging = args.verbose;

    // Choose interface mode
    match args.mode.as_str() {
        #[cfg(feature = "tauri-gui")]
        "gui" => {
            info!("🖥️ Starting Tauri GUI mode");
            run_gui_mode(config).await?;
        }
        #[cfg(not(feature = "tauri-gui"))]
        "gui" => {
            error!("GUI mode not available. Compile with --features tauri-gui");
            std::process::exit(1);
        }
        "cli" => {
            info!("💻 Starting CLI mode");
            run_cli_mode(config).await?;
        }
        _ => {
            error!("Unknown interface mode: {}. Use 'gui' or 'cli'", args.mode);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(feature = "tauri-gui")]
async fn run_gui_mode(config: AppConfig) -> Result<()> {
    // Create and start the simulation application backend
    let mut app = SimulationApp::new(config);

    // Start BLE peripheral
    app.start_ble_peripheral().await?;

    // Get BLE peripheral for Tauri integration
    if let Some(ble_peripheral) = app.ble_peripheral.as_ref() {
        // Start Tauri application
        tauri_app::setup_tauri_app(ble_peripheral).await?;
    } else {
        error!("Failed to start BLE peripheral for GUI mode");
        return Err(anyhow::anyhow!("BLE peripheral not available"));
    }

    Ok(())
}

#[cfg(not(feature = "tauri-gui"))]
#[allow(dead_code)]
async fn run_gui_mode(_config: AppConfig) -> Result<()> {
    error!("GUI mode not available. Compile with --features tauri-gui");
    Err(anyhow::anyhow!("GUI mode not available"))
}

#[cfg(feature = "cli-only")]
async fn run_cli_mode(config: AppConfig) -> Result<()> {
    // Create and run simulation
    let mut app = SimulationApp::new(config);

    match app.run().await {
        Ok(()) => {
            info!("Simulation completed successfully");
        }
        Err(e) => {
            error!("Simulation failed: {}", e);
            app.stop().await?;
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli-only"))]
async fn run_cli_mode(_config: AppConfig) -> Result<()> {
    error!("CLI mode not available. Compile with --features cli-only");
    Err(anyhow::anyhow!("CLI mode not available"))
}
