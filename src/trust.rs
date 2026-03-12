//! Trust and safety policy for the simulator.
//!
//! This module centralizes trust-state transitions, degraded-mode handling,
//! revocation, and privileged-action gating so transports do not invent their
//! own security semantics.

use serde::{Deserialize, Serialize};

/// Trust state of the simulated device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustState {
    /// Device is visible but not trusted for privileged actions.
    Discovered,
    /// BLE pairing or equivalent secure association exists.
    Bonded,
    /// Device is explicitly trusted for privileged interaction flows.
    Enrolled,
    /// Device has stronger identity guarantees.
    Attested,
    /// Device has been revoked and must fail closed.
    Revoked,
}

/// Degraded conditions that can gate privileged behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradedMode {
    /// Battery level is too low for non-essential activity.
    LowBattery,
    /// Sensor health or calibration is degraded.
    SensorFault,
    /// Firmware compatibility requirements are not met.
    FirmwareMismatch,
    /// Operator policy has blocked privileged activity.
    OperatorBlocked,
}

/// Privileged actions evaluated by the safety policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegedAction {
    /// Execute a haptic command.
    HapticCommand,
    /// Execute an arbitrary protocol command.
    ExecuteProtocolCommand,
    /// Access higher-sensitivity diagnostics.
    SensitiveDiagnostics,
}

/// Result of evaluating a privileged action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Whether the action is allowed.
    pub allowed: bool,
    /// Human-readable explanation for denial.
    pub reason: Option<String>,
}

impl PolicyDecision {
    /// Returns an allow decision.
    pub fn allow() -> Self {
        Self {
            allowed: true,
            reason: None,
        }
    }

    /// Returns a deny decision with a reason.
    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: Some(reason.into()),
        }
    }
}

/// Mutable trust and degraded-state policy for the simulated device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Current trust state.
    pub trust_state: TrustState,
    /// Active degraded modes.
    pub degraded_modes: Vec<DegradedMode>,
    /// Whether the current firmware is compatible with policy.
    pub firmware_compatible: bool,
    /// Optional revocation reason if the device has been revoked.
    pub revocation_reason: Option<String>,
}

impl Default for TrustPolicy {
    fn default() -> Self {
        Self {
            trust_state: TrustState::Enrolled,
            degraded_modes: Vec::new(),
            firmware_compatible: true,
            revocation_reason: None,
        }
    }
}

impl TrustPolicy {
    /// Transition to a new trust state.
    pub fn transition_to(&mut self, trust_state: TrustState) {
        self.trust_state = trust_state;
        if trust_state != TrustState::Revoked {
            self.revocation_reason = None;
        }
    }

    /// Revoke the device and persist a human-readable reason.
    pub fn revoke(&mut self, reason: impl Into<String>) {
        self.trust_state = TrustState::Revoked;
        self.revocation_reason = Some(reason.into());
    }

    /// Enable or disable a degraded mode.
    pub fn set_degraded_mode(&mut self, mode: DegradedMode, enabled: bool) {
        let existing = self
            .degraded_modes
            .iter()
            .position(|current| current == &mode);
        match (enabled, existing) {
            (true, None) => self.degraded_modes.push(mode),
            (false, Some(index)) => {
                self.degraded_modes.remove(index);
            }
            _ => {}
        }
    }

    /// Update low-battery degraded state from a battery percentage.
    pub fn sync_low_battery(&mut self, battery_level_percent: u8) {
        self.set_degraded_mode(DegradedMode::LowBattery, battery_level_percent <= 10);
    }

    /// Update firmware compatibility and the corresponding degraded state.
    pub fn set_firmware_compatible(&mut self, firmware_compatible: bool) {
        self.firmware_compatible = firmware_compatible;
        self.set_degraded_mode(DegradedMode::FirmwareMismatch, !firmware_compatible);
    }

    /// Evaluate whether a privileged action is allowed.
    pub fn evaluate(&self, action: PrivilegedAction) -> PolicyDecision {
        if self.trust_state == TrustState::Revoked {
            return PolicyDecision::deny(
                self.revocation_reason
                    .clone()
                    .unwrap_or_else(|| "device trust has been revoked".to_string()),
            );
        }

        if !self.firmware_compatible
            || self
                .degraded_modes
                .contains(&DegradedMode::FirmwareMismatch)
        {
            return PolicyDecision::deny("firmware compatibility policy is not satisfied");
        }

        if self.degraded_modes.contains(&DegradedMode::OperatorBlocked) {
            return PolicyDecision::deny("operator policy is blocking privileged actions");
        }

        match action {
            PrivilegedAction::HapticCommand | PrivilegedAction::ExecuteProtocolCommand => {
                if self.trust_state < TrustState::Enrolled {
                    return PolicyDecision::deny(
                        "device is not enrolled for privileged command execution",
                    );
                }

                if self.degraded_modes.contains(&DegradedMode::LowBattery) {
                    return PolicyDecision::deny(
                        "device is in low-battery degraded mode and must fail closed",
                    );
                }

                if self.degraded_modes.contains(&DegradedMode::SensorFault) {
                    return PolicyDecision::deny(
                        "device sensors are degraded and privileged actions are disabled",
                    );
                }

                PolicyDecision::allow()
            }
            PrivilegedAction::SensitiveDiagnostics => {
                if self.trust_state < TrustState::Bonded {
                    PolicyDecision::deny(
                        "device must be bonded before sensitive diagnostics are exposed",
                    )
                } else {
                    PolicyDecision::allow()
                }
            }
        }
    }
}
