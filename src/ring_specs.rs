//! Ring Specifications Module
//!
//! Defines the specifications and capabilities for different Haptic Harmony ring models.
//! This module provides a modular system for supporting multiple ring types with
//! different features, haptic zones, and capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ring type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingType {
    /// B1 - Basic Haptic Harmony Ring (Current)
    B1,
    /// A1 - Advanced Ring (Coming 2025)
    A1,
    /// P1 - Pro Ring (Coming 2025)
    P1,
}

impl std::fmt::Display for RingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingType::B1 => write!(f, "B1"),
            RingType::A1 => write!(f, "A1"),
            RingType::P1 => write!(f, "P1"),
        }
    }
}

impl std::str::FromStr for RingType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "b1" => Ok(RingType::B1),
            "a1" => Ok(RingType::A1),
            "p1" => Ok(RingType::P1),
            _ => Err(format!("Unknown ring type: {s}")),
        }
    }
}

/// Haptic pattern types supported by different rings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticPattern {
    Pulse,
    Wave,
    Burst,
    Cascade,
    Spiral,
    Neural,
    Adaptive,
}

/// Ring capabilities and specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSpec {
    /// Ring model name
    pub name: String,
    /// Number of haptic zones
    pub haptic_zones: u8,
    /// Maximum haptic intensity (0-255)
    pub max_intensity: u8,
    /// Supported haptic patterns
    pub patterns: Vec<HapticPattern>,
    /// Supported gesture types
    pub gestures: Vec<String>,
    /// Firmware version
    pub firmware_version: String,
    /// Battery capacity (mAh)
    pub battery_capacity: u16,
    /// BLE version
    pub ble_version: String,
    /// Additional features
    pub features: Vec<String>,
    /// Whether this ring is currently available
    pub available: bool,
}

impl RingSpec {
    /// Create specification for B1 ring
    pub fn b1() -> Self {
        Self {
            name: "B1 Haptic Harmony Ring".to_string(),
            haptic_zones: 4,
            max_intensity: 100,
            patterns: vec![
                HapticPattern::Pulse,
                HapticPattern::Wave,
                HapticPattern::Burst,
            ],
            gestures: vec![
                "tap".to_string(),
                "double_tap".to_string(),
                "hold".to_string(),
                "slide".to_string(),
                "tilt".to_string(),
            ],
            firmware_version: "v1.2.3".to_string(),
            battery_capacity: 150,
            ble_version: "5.0".to_string(),
            features: vec![
                "Basic Gestures".to_string(),
                "Haptic Feedback".to_string(),
                "BLE Connectivity".to_string(),
                "Battery Monitoring".to_string(),
            ],
            available: true,
        }
    }

    /// Create specification for A1 ring (future)
    pub fn a1() -> Self {
        Self {
            name: "A1 Advanced Ring".to_string(),
            haptic_zones: 8,
            max_intensity: 150,
            patterns: vec![
                HapticPattern::Pulse,
                HapticPattern::Wave,
                HapticPattern::Burst,
                HapticPattern::Cascade,
                HapticPattern::Spiral,
            ],
            gestures: vec![
                "tap".to_string(),
                "double_tap".to_string(),
                "triple_tap".to_string(),
                "hold".to_string(),
                "slide".to_string(),
                "swipe".to_string(),
                "tilt".to_string(),
                "rotate".to_string(),
                "pinch".to_string(),
            ],
            firmware_version: "v2.0.0".to_string(),
            battery_capacity: 300,
            ble_version: "5.2".to_string(),
            features: vec![
                "Advanced Gestures".to_string(),
                "Multi-Zone Haptics".to_string(),
                "AI Processing".to_string(),
                "Wireless Charging".to_string(),
                "Health Monitoring".to_string(),
            ],
            available: false, // Coming 2025
        }
    }

    /// Create specification for P1 ring (future)
    pub fn p1() -> Self {
        Self {
            name: "P1 Pro Ring".to_string(),
            haptic_zones: 12,
            max_intensity: 200,
            patterns: vec![
                HapticPattern::Pulse,
                HapticPattern::Wave,
                HapticPattern::Burst,
                HapticPattern::Cascade,
                HapticPattern::Spiral,
                HapticPattern::Neural,
                HapticPattern::Adaptive,
            ],
            gestures: vec![
                "tap".to_string(),
                "double_tap".to_string(),
                "triple_tap".to_string(),
                "hold".to_string(),
                "slide".to_string(),
                "swipe".to_string(),
                "tilt".to_string(),
                "rotate".to_string(),
                "pinch".to_string(),
                "neural_gesture".to_string(),
                "custom_gesture".to_string(),
            ],
            firmware_version: "v3.0.0".to_string(),
            battery_capacity: 500,
            ble_version: "5.3".to_string(),
            features: vec![
                "Pro Gestures".to_string(),
                "Precision Haptics".to_string(),
                "Neural Interface".to_string(),
                "Wireless Charging".to_string(),
                "Health Monitoring".to_string(),
                "Biometric Authentication".to_string(),
                "Environmental Sensing".to_string(),
            ],
            available: false, // Coming 2025
        }
    }
}

/// Ring specification manager
#[derive(Debug, Clone)]
pub struct RingSpecManager {
    specs: HashMap<RingType, RingSpec>,
    current_ring: RingType,
}

#[allow(dead_code)]
impl RingSpecManager {
    /// Create a new ring specification manager
    pub fn new() -> Self {
        let mut specs = HashMap::new();
        specs.insert(RingType::B1, RingSpec::b1());
        specs.insert(RingType::A1, RingSpec::a1());
        specs.insert(RingType::P1, RingSpec::p1());

        Self {
            specs,
            current_ring: RingType::B1,
        }
    }

    /// Get specification for a ring type
    pub fn get_spec(&self, ring_type: &RingType) -> Option<&RingSpec> {
        self.specs.get(ring_type)
    }

    /// Get current ring specification
    pub fn current_spec(&self) -> &RingSpec {
        self.specs.get(&self.current_ring).unwrap()
    }

    /// Switch to a different ring type
    pub fn switch_ring(&mut self, ring_type: RingType) -> Result<(), String> {
        if self.specs.contains_key(&ring_type) {
            self.current_ring = ring_type;
            Ok(())
        } else {
            Err(format!("Unknown ring type: {ring_type}"))
        }
    }

    /// Get current ring type
    pub fn current_ring_type(&self) -> &RingType {
        &self.current_ring
    }

    /// Get all available ring types
    pub fn available_rings(&self) -> Vec<&RingType> {
        self.specs
            .iter()
            .filter(|(_, spec)| spec.available)
            .map(|(ring_type, _)| ring_type)
            .collect()
    }

    /// Get all ring types (including future ones)
    pub fn all_rings(&self) -> Vec<&RingType> {
        self.specs.keys().collect()
    }

    /// Check if a ring type is available
    pub fn is_available(&self, ring_type: &RingType) -> bool {
        self.specs
            .get(ring_type)
            .map(|spec| spec.available)
            .unwrap_or(false)
    }
}

impl Default for RingSpecManager {
    fn default() -> Self {
        Self::new()
    }
}
