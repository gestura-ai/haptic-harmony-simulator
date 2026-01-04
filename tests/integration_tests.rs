//! Integration tests for the Gestura Ring Simulation Kit

use haptic_harmony_simulation::emulator::*;
use haptic_harmony_simulation::feedback::*;
use haptic_harmony_simulation::mcp_mock::*;
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
