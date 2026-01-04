use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::connectivity::ble_peripheral::{BleBatteryData, BlePeripheral};
use crate::emulator::GestureType;
use crate::ring_specs::{HapticPattern, RingSpecManager, RingType};

/// Tauri application state
#[derive(Clone)]
pub struct AppState {
    pub ble_peripheral: Arc<RwLock<Option<BlePeripheral>>>,
    pub app_handle: Arc<RwLock<Option<AppHandle>>>,
    pub ring_manager: Arc<RwLock<RingSpecManager>>,
}

#[allow(dead_code)]
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ble_peripheral: Arc::new(RwLock::new(None)),
            app_handle: Arc::new(RwLock::new(None)),
            ring_manager: Arc::new(RwLock::new(RingSpecManager::new())),
        }
    }

    #[allow(dead_code)]
    pub async fn set_ble_peripheral(&self, peripheral: BlePeripheral) {
        let mut ble = self.ble_peripheral.write().await;
        *ble = Some(peripheral);
    }

    pub async fn set_app_handle(&self, handle: AppHandle) {
        let mut app = self.app_handle.write().await;
        *app = Some(handle);
    }

    pub async fn emit_event<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<()> {
        if let Some(handle) = self.app_handle.read().await.as_ref() {
            handle.emit(event, payload)?;
        }
        Ok(())
    }
}

/// Gesture event payload for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureEvent {
    pub gesture: String,
    pub confidence: u8,
    pub timestamp: u64,
}

/// Haptic command payload for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticEvent {
    pub pattern: String,
    pub intensity: f32,
    pub duration: u32,
    pub timestamp: u64,
}

/// Battery update payload for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryEvent {
    pub level: u8,
    pub charging: bool,
    pub voltage: f32,
    pub temperature: f32,
    pub health: String,
    pub time_remaining: Option<u32>,
}

/// BLE status payload for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleStatusEvent {
    pub status: String,
    pub connected_clients: u32,
    pub device_name: String,
}

/// Log message payload for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

/// Tauri command to trigger a gesture
#[tauri::command]
pub async fn trigger_gesture(gesture: String, state: State<'_, AppState>) -> Result<(), String> {
    info!("🎮 Tauri command: trigger_gesture({})", gesture);

    let gesture_type = match gesture.as_str() {
        "tap" => GestureType::Tap,
        "double-tap" => GestureType::DoubleTap,
        "hold" => GestureType::Hold {
            duration: std::time::Duration::from_millis(1000),
        },
        "slide" => GestureType::Slide {
            direction: crate::emulator::SlideDirection::Up,
        },
        "tilt" => GestureType::Tilt { angle: 45.0 },
        _ => return Err(format!("Unknown gesture: {gesture}")),
    };

    // Get BLE peripheral and send gesture
    if let Some(ble) = state.ble_peripheral.read().await.as_ref() {
        // Create a gesture event
        let gesture_event = crate::emulator::GestureEvent {
            gesture_type,
            timestamp: tokio::time::Instant::now(),
            confidence: 0.9,
        };

        if let Err(e) = ble.notify_gesture(&gesture_event).await {
            error!("Failed to notify gesture: {}", e);
            return Err(format!("Failed to notify gesture: {e}"));
        }
    }

    // Emit event to frontend
    let event = GestureEvent {
        gesture: gesture.clone(),
        confidence: 85 + (rand::random::<u8>() % 15), // 85-99% confidence
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    if let Err(e) = state.emit_event("gesture-detected", event).await {
        error!("Failed to emit gesture event: {}", e);
    }

    Ok(())
}

/// Tauri command to trigger haptic feedback
#[tauri::command]
pub async fn trigger_haptic(
    pattern: String,
    intensity: f32,
    duration: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!(
        "📳 Tauri command: trigger_haptic({}, {}, {})",
        pattern, intensity, duration
    );

    let _haptic_pattern = match pattern.as_str() {
        "pulse" => HapticPattern::Pulse,
        "wave" => HapticPattern::Wave,
        "burst" => HapticPattern::Burst,
        "cascade" => HapticPattern::Cascade,
        "spiral" => HapticPattern::Spiral,
        "neural" => HapticPattern::Neural,
        "adaptive" => HapticPattern::Adaptive,
        _ => HapticPattern::Pulse, // Default to pulse for unknown patterns
    };

    // Simulate haptic command via BLE peripheral
    if let Some(ble) = state.ble_peripheral.read().await.as_ref()
        && let Err(e) = ble
            .simulate_haptic_command(&pattern, intensity, duration)
            .await
    {
        error!("Failed to simulate haptic: {}", e);
        return Err(format!("Failed to simulate haptic: {e}"));
    }

    // Emit event to frontend
    let event = HapticEvent {
        pattern: pattern.clone(),
        intensity,
        duration,
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    if let Err(e) = state.emit_event("haptic-command", event).await {
        error!("Failed to emit haptic event: {}", e);
    }

    Ok(())
}

/// Tauri command to toggle charging
#[tauri::command]
pub async fn toggle_charging(state: State<'_, AppState>) -> Result<(), String> {
    info!("🔌 Tauri command: toggle_charging");

    if let Some(ble) = state.ble_peripheral.read().await.as_ref() {
        if let Err(e) = ble.toggle_charging().await {
            error!("Failed to toggle charging: {}", e);
            return Err(format!("Failed to toggle charging: {e}"));
        }

        // Get updated battery status and emit event
        let battery_data = ble.get_battery_status().await;
        let event = BatteryEvent {
            level: battery_data.level,
            charging: battery_data.is_charging,
            voltage: battery_data.voltage,
            temperature: battery_data.temperature,
            health: battery_data.health,
            time_remaining: battery_data.time_remaining,
        };

        if let Err(e) = state.emit_event("battery-update", event).await {
            error!("Failed to emit battery event: {}", e);
        }
    }

    Ok(())
}

/// Tauri command to set battery level
#[tauri::command]
pub async fn set_battery_level(level: u8, state: State<'_, AppState>) -> Result<(), String> {
    info!("🔋 Tauri command: set_battery_level({})", level);

    if let Some(ble) = state.ble_peripheral.read().await.as_ref() {
        if let Err(e) = ble.set_battery_level(level).await {
            error!("Failed to set battery level: {}", e);
            return Err(format!("Failed to set battery level: {e}"));
        }

        // Get updated battery status and emit event
        let battery_data = ble.get_battery_status().await;
        let event = BatteryEvent {
            level: battery_data.level,
            charging: battery_data.is_charging,
            voltage: battery_data.voltage,
            temperature: battery_data.temperature,
            health: battery_data.health,
            time_remaining: battery_data.time_remaining,
        };

        if let Err(e) = state.emit_event("battery-update", event).await {
            error!("Failed to emit battery event: {}", e);
        }
    }

    Ok(())
}

/// Tauri command to get battery info
#[tauri::command]
pub async fn get_battery_info(state: State<'_, AppState>) -> Result<BleBatteryData, String> {
    info!("ℹ️ Tauri command: get_battery_info");

    if let Some(ble) = state.ble_peripheral.read().await.as_ref() {
        let battery_data = ble.get_battery_status().await;
        Ok(battery_data)
    } else {
        Err("BLE peripheral not available".to_string())
    }
}

/// Tauri command to get BLE status
#[tauri::command]
pub async fn get_ble_status(state: State<'_, AppState>) -> Result<BleStatusEvent, String> {
    info!("📡 Tauri command: get_ble_status");

    if let Some(_ble) = state.ble_peripheral.read().await.as_ref() {
        let status = BleStatusEvent {
            status: "Advertising".to_string(),
            connected_clients: 0, // TODO: Get actual client count
            device_name: "Haptic Harmony Ring Simulator".to_string(),
        };
        Ok(status)
    } else {
        Err("BLE peripheral not available".to_string())
    }
}

/// Switch ring type
#[tauri::command]
async fn switch_ring_type(ring_type: String, state: State<'_, AppState>) -> Result<String, String> {
    let ring_type = ring_type
        .parse::<RingType>()
        .map_err(|e| format!("Invalid ring type: {e}"))?;

    let mut manager = state.ring_manager.write().await;
    manager
        .switch_ring(ring_type.clone())
        .map_err(|e| format!("Failed to switch ring: {e}"))?;

    let spec = manager.current_spec();
    info!("Switched to ring type: {} ({})", ring_type, spec.name);

    Ok(spec.name.clone())
}

/// Get ring specifications for all ring types
#[tauri::command]
async fn get_ring_specs(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let manager = state.ring_manager.read().await;
    let specs: std::collections::HashMap<String, _> = manager
        .all_rings()
        .iter()
        .map(|ring_type| {
            let spec = manager.get_spec(ring_type).unwrap();
            (ring_type.to_string().to_lowercase(), spec.clone())
        })
        .collect();

    serde_json::to_value(specs).map_err(|e| format!("Failed to serialize specs: {e}"))
}

/// Get current ring information
#[tauri::command]
async fn get_current_ring(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let manager = state.ring_manager.read().await;
    let current_type = manager.current_ring_type();
    let current_spec = manager.current_spec();

    let info = serde_json::json!({
        "type": current_type.to_string().to_lowercase(),
        "spec": current_spec
    });

    Ok(info)
}

/// Window size structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

/// Get current window size
#[tauri::command]
async fn get_window_size(app_handle: tauri::AppHandle) -> Result<WindowSize, String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let size = window
        .inner_size()
        .map_err(|e| format!("Failed to get window size: {e}"))?;

    Ok(WindowSize {
        width: size.width,
        height: size.height,
    })
}

/// Set window size
#[tauri::command]
async fn set_window_size(
    width: u32,
    height: u32,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let size = tauri::Size::Physical(tauri::PhysicalSize { width, height });

    window
        .set_size(size)
        .map_err(|e| format!("Failed to set window size: {e}"))?;

    Ok(())
}

/// Initialize Tauri application with BLE integration
pub async fn setup_tauri_app(_ble_peripheral: &BlePeripheral) -> Result<()> {
    let state = AppState::new();
    // Note: For now we'll skip setting the BLE peripheral in state
    // This would require more complex lifetime management

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();

            // Store app handle for event emission
            let state_clone = state.inner().clone();
            tauri::async_runtime::spawn(async move {
                state_clone.set_app_handle(handle).await;
            });

            info!("🚀 Tauri application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            trigger_gesture,
            trigger_haptic,
            toggle_charging,
            set_battery_level,
            get_battery_info,
            get_ble_status,
            switch_ring_type,
            get_ring_specs,
            get_current_ring,
            get_window_size,
            set_window_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
