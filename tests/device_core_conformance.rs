//! Conformance: replays the firmware repo's checked-in golden vectors through
//! the EMBEDDED device core and diffs byte-for-byte against the `.expected`
//! files. Parity with the ring is enforced, not assumed — if firmware changes
//! its engine/codec and regenerates vectors, this test tells the simulator.
//!
//! Requires the firmware checkout (GESTURA_FIRMWARE_DIR or the sibling
//! ../haptic-basic-firmware) — same requirement as building with the
//! `device-core` feature at all.
//!
//! NOTE: the scenarios share the single global core instance and are
//! stateful (sequence numbers, config), so all vectors run inside ONE test
//! fn, in filename order, exactly like `run_conformance.sh`.

#![cfg(feature = "device-core")]

use haptic_harmony_simulation::device_core::DeviceCore;
use std::path::PathBuf;

fn firmware_dir() -> PathBuf {
    std::env::var("GESTURA_FIRMWARE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../haptic-basic-firmware"))
}

/// Renumbers `"message_id":"fw-N"` occurrences to start at fw-1, preserving
/// order (see the comparison note in the test body).
fn rebase_message_ids(text: &str) -> String {
    const NEEDLE: &str = "\"message_id\":\"fw-";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut offset: Option<i64> = None;
    while let Some(idx) = rest.find(NEEDLE) {
        let after = idx + NEEDLE.len();
        out.push_str(&rest[..after]);
        rest = &rest[after..];
        let digits_end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let n: i64 = rest[..digits_end].parse().expect("fw-N digits");
        let base = *offset.get_or_insert(n - 1);
        out.push_str(&(n - base).to_string());
        rest = &rest[digits_end..];
    }
    out.push_str(rest);
    out
}

#[test]
fn golden_vectors_match_firmware() {
    let vectors = firmware_dir().join("conformance/vectors");
    assert!(
        vectors.is_dir(),
        "firmware conformance vectors not found at {} — set GESTURA_FIRMWARE_DIR",
        vectors.display()
    );

    let mut scenario_paths: Vec<PathBuf> = std::fs::read_dir(&vectors)
        .expect("read vectors dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "scenario"))
        .collect();
    scenario_paths.sort();
    assert!(
        !scenario_paths.is_empty(),
        "no .scenario files in {}",
        vectors.display()
    );

    let core = DeviceCore::global();
    for scenario_path in scenario_paths {
        let expected_path = scenario_path.with_extension("expected");
        let scenario = std::fs::read_to_string(&scenario_path).expect("read scenario");
        // Windows checkouts may materialize the vectors with CRLF (git
        // autocrlf); the core always emits LF. Normalize before comparing —
        // line endings are checkout artifacts, not contract bytes.
        let expected = std::fs::read_to_string(&expected_path)
            .expect("read expected")
            .replace("\r\n", "\n");

        let actual = core
            .replay_scenario(&scenario)
            .unwrap_or_else(|e| panic!("replay failed for {}: {e}", scenario_path.display()));

        // The core's message-id counter (fw-N) is per-process bookkeeping:
        // run_conformance.sh resets it by running each scenario in a fresh
        // process, while this test replays all scenarios in one. Rebase the
        // ids to start at fw-1 per scenario — ordering within a scenario is
        // preserved exactly; everything else is compared byte-for-byte.
        assert_eq!(
            rebase_message_ids(&actual),
            rebase_message_ids(&expected),
            "golden-vector mismatch for {} — the embedded core diverges from \
             the checked-in firmware vectors",
            scenario_path.display()
        );
    }

    // Dynamic case from run_conformance.sh, run LAST in the same test fn (the
    // core's message-id counter is global — a separate #[test] would race it
    // and shift every golden's fw-N id): a waveform beyond the 1024-sample
    // device FIFO cap must be rejected with the exact firmware ack.
    {
        use base64::Engine as _;
        let data = base64::engine::general_purpose::STANDARD.encode(vec![0u8; 2 * 1500]);
        let cmd = format!(
            "{{\"sequence\":77,\"payload\":{{\"pattern\":{{\"pattern_kind\":\"waveform\",\
             \"data\":\"{data}\",\"sample_rate_hz\":8000,\"intensity\":1.0}}}}}}"
        );

        let decode = core.decode_haptic(cmd.as_bytes());
        assert!(decode.err != 0, "oversized waveform must be rejected");
        assert!(
            decode
                .ack_json
                .contains("\"status\":\"error\",\"reason\":\"waveform too large\""),
            "ack must carry the firmware's exact rejection: {}",
            decode.ack_json
        );
    }
}
