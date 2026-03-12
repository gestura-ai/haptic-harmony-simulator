//! Integration tests for the Gestura Ring Simulation Kit

use haptic_harmony_simulation::BlePeripheral;
use haptic_harmony_simulation::ConnectionConfig;
use haptic_harmony_simulation::ble_peripheral::{BleGestureData, HapticCommand, ring_uuids};
use haptic_harmony_simulation::emulator::*;
use haptic_harmony_simulation::feedback::*;
use haptic_harmony_simulation::mcp_mock::*;
use haptic_harmony_simulation::protocol::*;
use haptic_harmony_simulation::transport_adapters::*;
use haptic_harmony_simulation::trust::*;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_basic_gesture_emulation() {
    // Test basic gesture emulation functionality
    let config = GestureConfig::default();

    // Verify default configuration values
    assert_eq!(config.double_tap_threshold.as_millis(), 300);
    assert_eq!(config.hold_threshold.as_millis(), 500);
    assert_eq!(config.sensitivity, 0.8);
    assert_eq!(config.max_tilt_angle, 45.0);
}

#[tokio::test]
async fn test_mcp_mock_server() {
    let config = McpConfig::default();
    let mut server = McpMockServer::new(config);

    // Start server
    server.start().await.expect("Failed to start MCP server");
    assert!(server.is_running());

    // Send a notification
    server
        .send_notification(
            "Test notification".to_string(),
            NotificationPriority::Normal,
        )
        .await
        .expect("Failed to send notification");

    // Check for messages
    tokio::time::sleep(Duration::from_millis(10)).await;
    let message = server.try_recv_message();
    assert!(message.is_some());

    // Stop server
    server.stop().await.expect("Failed to stop server");
    assert!(!server.is_running());
}

#[tokio::test]
async fn test_feedback_loop_basic() {
    let config = FeedbackConfig::default();
    let mut feedback_loop = FeedbackLoop::new(config);

    // Start feedback loop
    feedback_loop
        .start()
        .await
        .expect("Failed to start feedback loop");
    assert!(feedback_loop.is_running());

    // Send a test event
    let input_event = InputEvent::KeyPress {
        key: "Enter".to_string(),
    };
    let feedback_event = FeedbackEvent::Input(input_event);

    feedback_loop
        .send_event(feedback_event)
        .await
        .expect("Failed to send event");

    // Process the event
    let result = timeout(
        Duration::from_millis(100),
        feedback_loop.process_next_event(),
    )
    .await;
    assert!(result.is_ok());

    // Stop feedback loop
    feedback_loop
        .stop()
        .await
        .expect("Failed to stop feedback loop");
    assert!(!feedback_loop.is_running());
}

#[tokio::test]
async fn test_response_time_requirement() {
    // Test that response time is under 50ms requirement
    let config = FeedbackConfig {
        max_response_time: Duration::from_millis(50),
        ..Default::default()
    };

    let mut feedback_loop = FeedbackLoop::new(config);
    feedback_loop
        .start()
        .await
        .expect("Failed to start feedback loop");

    let start_time = std::time::Instant::now();

    // Send and process an event
    let input_event = InputEvent::KeyPress {
        key: "test".to_string(),
    };
    let feedback_event = FeedbackEvent::Input(input_event);

    feedback_loop
        .send_event(feedback_event)
        .await
        .expect("Failed to send event");
    feedback_loop
        .process_next_event()
        .await
        .expect("Failed to process event");

    let elapsed = start_time.elapsed();
    assert!(
        elapsed < Duration::from_millis(50),
        "Response time exceeded 50ms: {elapsed:?}"
    );

    feedback_loop
        .stop()
        .await
        .expect("Failed to stop feedback loop");
}

#[tokio::test]
async fn test_gesture_types() {
    // Test all gesture types can be created
    let tap = GestureType::Tap;
    let double_tap = GestureType::DoubleTap;
    let hold = GestureType::Hold {
        duration: Duration::from_millis(500),
    };
    let slide = GestureType::Slide {
        direction: SlideDirection::Up,
    };
    let tilt = GestureType::Tilt { angle: 15.0 };

    // Basic validation
    assert_eq!(tap, GestureType::Tap);
    assert_eq!(double_tap, GestureType::DoubleTap);

    match hold {
        GestureType::Hold { duration } => assert_eq!(duration, Duration::from_millis(500)),
        _ => panic!("Expected Hold gesture"),
    }

    match slide {
        GestureType::Slide { direction } => assert_eq!(direction, SlideDirection::Up),
        _ => panic!("Expected Slide gesture"),
    }

    match tilt {
        GestureType::Tilt { angle } => assert_eq!(angle, 15.0),
        _ => panic!("Expected Tilt gesture"),
    }
}

#[tokio::test]
async fn test_haptic_patterns() {
    // Test all haptic patterns can be created
    let notify = HapticPattern::Notify;
    let custom = HapticPattern::Custom {
        intensity: 0.8,
        duration: Duration::from_millis(200),
    };
    let success = HapticPattern::Success;
    let error = HapticPattern::Error;

    assert_eq!(notify, HapticPattern::Notify);
    assert_eq!(success, HapticPattern::Success);
    assert_eq!(error, HapticPattern::Error);

    match custom {
        HapticPattern::Custom {
            intensity,
            duration,
        } => {
            assert_eq!(intensity, 0.8);
            assert_eq!(duration, Duration::from_millis(200));
        }
        _ => panic!("Expected Custom haptic pattern"),
    }
}

#[tokio::test]
async fn test_deterministic_scenario_compiles_stable_sequences() {
    let scenario = demo_ring_interaction_scenario();
    let compiled = scenario.compile();

    assert_eq!(compiled.len(), 3);
    assert_eq!(compiled[0].sequence, 1);
    assert_eq!(compiled[1].sequence, 2);
    assert_eq!(compiled[2].sequence, 3);
    assert_eq!(compiled[0].timestamp_ms, 0);
    assert_eq!(compiled[1].timestamp_ms, 75);
    assert_eq!(compiled[2].timestamp_ms, 200);

    match &compiled[1].payload {
        SimulatorEvent::Gesture(event) => assert_eq!(event.gesture, SemanticGesture::Tap),
        other => panic!("Expected gesture event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_ble_gesture_data_embeds_protocol_event() {
    let gesture = GestureEvent {
        gesture_type: GestureType::Slide {
            direction: SlideDirection::Left,
        },
        timestamp: tokio::time::Instant::now(),
        confidence: 0.91,
    };

    let ble_gesture = BleGestureData::from_gesture_event(&gesture, 7)
        .expect("BLE gesture conversion should succeed");
    let embedded: ProtocolEnvelope<SimulatorEvent> =
        serde_json::from_slice(&ble_gesture.data).expect("embedded protocol JSON should decode");

    assert_eq!(embedded.sequence, 7);

    match embedded.payload {
        SimulatorEvent::Gesture(event) => match event.gesture {
            SemanticGesture::Slide { direction } => {
                assert_eq!(direction, SemanticSlideDirection::Left)
            }
            other => panic!("Expected slide gesture, got {other:?}"),
        },
        other => panic!("Expected embedded gesture event, got {other:?}"),
    }
}

#[tokio::test]
async fn test_haptic_command_protocol_round_trip() {
    let command = HapticCommand {
        pattern: "notify".to_string(),
        intensity: 0.8,
        duration_ms: 300,
    };

    let encoded = serde_json::to_vec(&command.to_protocol_envelope(11))
        .expect("protocol encoding should succeed");
    let decoded =
        HapticCommand::try_from_protocol_bytes(&encoded).expect("protocol decoding should succeed");

    assert_eq!(decoded.pattern, "notify");
    assert_eq!(decoded.duration_ms, 200);
    assert_eq!(decoded.intensity, 1.0);
}

#[tokio::test]
async fn test_mcp_protocol_projection_uses_notification_channel() {
    let config = McpConfig::default();
    let mut server = McpMockServer::new(config);
    server.start().await.expect("server should start");

    let event = ProtocolEnvelope::event(
        1,
        42,
        SimulatorEvent::Gesture(SemanticGestureEvent {
            gesture: SemanticGesture::Tap,
            confidence: 0.99,
            timestamp_ms: 42,
        }),
    );

    server
        .send_protocol_event(&event)
        .await
        .expect("protocol projection should succeed");

    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut saw_protocol_payload = false;
    while let Some(message) = server.try_recv_message() {
        if let McpMessage::Notification { content, .. } = message.message
            && content.contains("gesture")
            && content.contains("protocol_version")
        {
            saw_protocol_payload = true;
            break;
        }
    }

    assert!(
        saw_protocol_payload,
        "expected MCP notification with embedded protocol payload"
    );
}

#[tokio::test]
async fn test_adapter_hub_projects_same_event_across_ble_socket_and_mcp() {
    let event = ProtocolEnvelope::event(
        4,
        250,
        SimulatorEvent::Gesture(SemanticGestureEvent {
            gesture: SemanticGesture::DoubleTap,
            confidence: 0.97,
            timestamp_ms: 250,
        }),
    );

    let hub = ProtocolAdapterHub::new(Some("session-123".to_string()));
    let projection = hub
        .fan_out_event(&event)
        .expect("projection should succeed");

    let ble_payload: BleGestureData =
        serde_json::from_slice(&projection.ble.payload).expect("BLE payload should decode");
    assert_eq!(
        projection.ble.characteristic_uuid,
        ring_uuids::GESTURE_EVENT_UUID
    );
    assert_eq!(ble_payload.gesture_type, "double_tap");

    let socket_event: ProtocolEnvelope<SimulatorEvent> =
        serde_json::from_value(projection.socket.data.clone())
            .expect("socket message should contain protocol envelope");
    assert_eq!(socket_event.sequence, 4);

    match projection.mcp {
        McpMessage::Notification { content, .. } => {
            assert!(content.contains("double_tap"));
            assert!(content.contains("protocol_version"));
        }
        other => panic!("expected MCP notification, got {other:?}"),
    }
}

#[tokio::test]
async fn test_ble_peripheral_denies_haptics_when_trust_is_not_enrolled() {
    let peripheral =
        BlePeripheral::new(ConnectionConfig::default()).expect("BLE peripheral should initialize");

    peripheral
        .transition_trust_state(TrustState::Bonded)
        .await
        .expect("trust state transition should succeed");

    let error = peripheral
        .simulate_haptic_command("notify", 0.8, 300)
        .await
        .expect_err("bonded-only device should not allow privileged haptics");

    assert!(
        error
            .to_string()
            .contains("device is not enrolled for privileged command execution")
    );
}

#[tokio::test]
async fn test_ble_peripheral_denies_haptics_when_low_battery_degraded() {
    let peripheral =
        BlePeripheral::new(ConnectionConfig::default()).expect("BLE peripheral should initialize");

    peripheral
        .set_battery_level(5)
        .await
        .expect("battery update should succeed");

    let snapshot = peripheral.get_protocol_state_snapshot().await;
    assert!(snapshot.degraded_modes.contains(&DegradedMode::LowBattery));
    assert!(!snapshot.privileged_actions_enabled);

    let error = peripheral
        .simulate_haptic_command("notify", 0.8, 300)
        .await
        .expect_err("low-battery degraded mode should block privileged haptics");

    assert!(
        error
            .to_string()
            .contains("device is in low-battery degraded mode")
    );
}

#[tokio::test]
async fn test_trust_policy_revocation_fails_closed() {
    let mut policy = TrustPolicy::default();
    policy.revoke("operator revoked device after integrity check failure");

    let decision = policy.evaluate(PrivilegedAction::ExecuteProtocolCommand);
    assert!(!decision.allowed);
    assert!(
        decision
            .reason
            .expect("revocation should provide a reason")
            .contains("operator revoked device")
    );
}

#[tokio::test]
async fn test_state_snapshot_projection_uses_state_characteristic_and_high_priority_mcp() {
    let snapshot = DeviceStateSnapshot {
        battery: BatterySnapshot {
            level_percent: 8,
            is_charging: false,
            voltage: 3.45,
            temperature_celsius: 26.0,
            health: "Fair".to_string(),
            time_remaining_minutes: Some(12),
        },
        trust_state: TrustState::Enrolled,
        degraded_modes: vec![DegradedMode::LowBattery],
        firmware_version: "1.0.0-sim".to_string(),
        protocol_version: SHARED_PROTOCOL_VERSION.to_string(),
        revocation_reason: None,
        privileged_actions_enabled: false,
    };

    let event = ProtocolEnvelope::event(9, 900, SimulatorEvent::StateSnapshot(snapshot));
    let hub = ProtocolAdapterHub::new(None);
    let projection = hub
        .fan_out_event(&event)
        .expect("state snapshot projection should succeed");

    assert_eq!(
        projection.ble.characteristic_uuid,
        ring_uuids::STATE_SNAPSHOT_UUID
    );

    match projection.mcp {
        McpMessage::Notification { priority, .. } => match priority {
            NotificationPriority::High => {}
            other => panic!("expected high-priority MCP notification, got {other:?}"),
        },
        other => panic!("expected MCP notification, got {other:?}"),
    }
}
