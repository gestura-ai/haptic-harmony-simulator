//! FFI embedding of the ring firmware's ACTUAL decision-making code
//! (`device_core/` in haptic-basic-firmware; compiled by `build.rs`).
//!
//! NOT a reimplementation: the gesture engine, semantic layer, and wire codec
//! are the same translation units the firmware ships — thresholds, masking,
//! hold synthesis, tap arbitration, confidence quantization, sequence
//! numbering, and the v0.3.0 wire JSON are firmware-identical by
//! construction. The `device_core` C API is a consumed contract surface:
//! changes follow propose → cross-check → confirm (firmware handoff,
//! 2026-07-08).
//!
//! Bindings are hand-written rather than bindgen-generated: the header is a
//! ten-function governed contract, and skipping bindgen keeps libclang out of
//! every build environment. If the header changes, the conformance replay
//! test (`tests/device_core_conformance.rs`) is the drift alarm.
//!
//! The C core is a single global instance (C statics) and not thread-safe;
//! [`DeviceCore::global`] wraps every call in one process-wide mutex.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::sync::{Mutex, OnceLock};

mod ffi {
    use super::*;

    pub type GdcEventCb =
        Option<unsafe extern "C" fn(json: *const c_char, len: usize, user: *mut c_void)>;

    /// Mirrors `struct gdc_haptic_result` in gestura_device_core.h.
    #[repr(C)]
    pub struct GdcHapticResult {
        pub pattern: c_int,
        pub sequence: u64,
        pub samples: [i16; 1024],
        pub n_samples: c_int,
        pub sample_rate_hz: u32,
        pub ack_json: [c_char; 512],
        pub ack_len: usize,
    }

    unsafe extern "C" {
        pub fn gdc_init(cb: GdcEventCb, user: *mut c_void) -> c_int;
        pub fn gdc_set_time_ms(now_ms: i64);
        pub fn gdc_feed_imu(
            ax_mg: i32,
            ay_mg: i32,
            az_mg: i32,
            gx_mdps: i32,
            gy_mdps: i32,
            gz_mdps: i32,
        );
        pub fn gdc_feed_touch(touched: c_int, slider_pos: u16, gesture_code: c_int);
        pub fn gdc_write_config(bytes: *const u8, len: usize) -> c_int;
        pub fn gdc_read_config(out: *mut u8);
        pub fn gdc_hid_enabled() -> c_int;
        pub fn gdc_decode_haptic(buf: *const u8, len: usize, out: *mut GdcHapticResult) -> c_int;
        pub fn gdc_state_snapshot_json(
            battery_pct: u8,
            charging: c_int,
            battery_mv: u16,
            trust: c_int,
            degraded_sensor: c_int,
            out: *mut c_char,
            out_len: usize,
        ) -> c_int;
        // C3 sensor-frame encoder (device-core v1.1, additive; schema
        // RATIFIED 2026-07-09/10 — layout normative in sensor_frame.h).
        pub fn gdc_frame_begin(t0_ms: u32, period_ms: u8, touch_valid: c_int) -> c_int;
        pub fn gdc_frame_add(
            ax_mg: i32,
            ay_mg: i32,
            az_mg: i32,
            gx_mdps: i32,
            gy_mdps: i32,
            gz_mdps: i32,
            slider_pos: u16,
            touched: c_int,
        ) -> c_int;
        pub fn gdc_frame_finish(out: *mut u8, out_len: usize) -> c_int;
        pub fn gdc_version() -> *const c_char;
    }
}

/// Events collected from the core's synchronous callback. The trampoline
/// pushes into this under the same lock that serializes all core calls.
static PENDING: Mutex<Vec<String>> = Mutex::new(Vec::new());

unsafe extern "C" fn trampoline(json: *const c_char, len: usize, _user: *mut c_void) {
    if json.is_null() || len == 0 {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(json as *const u8, len) };
    if let Ok(s) = std::str::from_utf8(bytes) {
        PENDING.lock().unwrap().push(s.to_string());
    }
}

/// Touch gesture codes accepted by `gdc_feed_touch` (from the header).
pub const TOUCH_NONE: i32 = 0;
pub const TOUCH_TAP: i32 = 1;
pub const TOUCH_SWIPE_LEFT: i32 = 2;
pub const TOUCH_SWIPE_RIGHT: i32 = 3;

/// Device-reportable trust codes for `gdc_state_snapshot_json` (the ring only
/// ever reports these three — richer simulator states collapse on the wire).
pub const TRUST_DISCOVERED: i32 = 0;
pub const TRUST_BONDED: i32 = 1;
pub const TRUST_REVOKED: i32 = 2;

/// Result of a haptic decode through the firmware's own parser.
pub struct HapticDecode {
    /// 0 = ok; negative errno-style values exactly as the device returns.
    pub err: i32,
    /// 0 confirm, 1 error, 2 tick, 3 double_tick, 4 waveform (valid when err == 0).
    pub pattern: i32,
    pub sequence: u64,
    pub samples: Vec<i16>,
    pub sample_rate_hz: u32,
    /// The exact ack envelope the ring would notify on the state characteristic.
    pub ack_json: String,
}

/// UI-level gestures the simulator can drive through the real engine.
/// Recipes below are the ones proven by the firmware's conformance vectors
/// (`conformance/vectors/gestures.scenario`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiGesture {
    Tap,
    DoubleTap,
    Hold { duration_ms: u64 },
    SwipeLeft,
    SwipeRight,
    RotateCw,
    RotateCcw,
}

pub struct DeviceCore {
    /// Serializes every core call (the C core is one global instance) and
    /// carries the virtual device-uptime clock (ms, monotonic, starts at 0 —
    /// matching real-hardware timestamp semantics).
    state: Mutex<i64>,
}

static CORE: OnceLock<DeviceCore> = OnceLock::new();

impl DeviceCore {
    /// The process-wide core instance (initializes the engine on first use).
    pub fn global() -> &'static DeviceCore {
        CORE.get_or_init(|| {
            unsafe {
                ffi::gdc_init(Some(trampoline), std::ptr::null_mut());
            }
            DeviceCore {
                state: Mutex::new(0),
            }
        })
    }

    /// Core + wire protocol version string, e.g. "device-core 1.0 / wire 0.3.0".
    pub fn version(&self) -> String {
        let _guard = self.state.lock().unwrap();
        unsafe { CStr::from_ptr(ffi::gdc_version()) }
            .to_string_lossy()
            .into_owned()
    }

    /// Applies a config write through the firmware's own semantics
    /// (1..4 bytes, trailing bytes unchanged, sensitivity/mask applied to the
    /// engine immediately). Returns the canonical 4-byte read-back, or `None`
    /// when the core rejects the write (invalid length).
    pub fn write_config(&self, bytes: &[u8]) -> Option<[u8; 4]> {
        let _guard = self.state.lock().unwrap();
        let rc = unsafe { ffi::gdc_write_config(bytes.as_ptr(), bytes.len()) };
        if rc != 0 {
            return None;
        }
        let mut out = [0u8; 4];
        unsafe { ffi::gdc_read_config(out.as_mut_ptr()) };
        Some(out)
    }

    /// Canonical 4-byte config read-back (readable-C2).
    pub fn read_config(&self) -> [u8; 4] {
        let _guard = self.state.lock().unwrap();
        let mut out = [0u8; 4];
        unsafe { ffi::gdc_read_config(out.as_mut_ptr()) };
        out
    }

    pub fn hid_enabled(&self) -> bool {
        let _guard = self.state.lock().unwrap();
        unsafe { ffi::gdc_hid_enabled() != 0 }
    }

    /// Decodes a haptic command with the device's own parser; the returned
    /// ack is byte-exact with what the ring notifies.
    pub fn decode_haptic(&self, buf: &[u8]) -> HapticDecode {
        let _guard = self.state.lock().unwrap();
        let mut raw = ffi::GdcHapticResult {
            pattern: 0,
            sequence: 0,
            samples: [0; 1024],
            n_samples: 0,
            sample_rate_hz: 0,
            ack_json: [0; 512],
            ack_len: 0,
        };
        let err = unsafe { ffi::gdc_decode_haptic(buf.as_ptr(), buf.len(), &mut raw) };
        let n = raw.n_samples.clamp(0, 1024) as usize;
        let ack_len = raw.ack_len.min(511);
        let ack_bytes =
            unsafe { std::slice::from_raw_parts(raw.ack_json.as_ptr() as *const u8, ack_len) };
        HapticDecode {
            err,
            pattern: raw.pattern,
            sequence: raw.sequence,
            samples: raw.samples[..n].to_vec(),
            sample_rate_hz: raw.sample_rate_hz,
            ack_json: String::from_utf8_lossy(ack_bytes).into_owned(),
        }
    }

    /// State snapshot exactly as the ring emits (full envelope JSON).
    pub fn state_snapshot_json(
        &self,
        battery_pct: u8,
        charging: bool,
        battery_mv: u16,
        trust: i32,
        degraded_sensor: bool,
    ) -> Option<String> {
        let _guard = self.state.lock().unwrap();
        let mut out = [0i8; 600];
        let n = unsafe {
            ffi::gdc_state_snapshot_json(
                battery_pct,
                charging as c_int,
                battery_mv,
                trust,
                degraded_sensor as c_int,
                out.as_mut_ptr() as *mut c_char,
                out.len(),
            )
        };
        if n <= 0 {
            return None;
        }
        let bytes = unsafe { std::slice::from_raw_parts(out.as_ptr() as *const u8, n as usize) };
        Some(String::from_utf8_lossy(bytes).into_owned())
    }

    /// Drives a UI-level gesture through the real engine and returns the
    /// wire-exact gesture envelope(s) the ring would notify. Input recipes
    /// mirror `conformance/vectors/gestures.scenario` (vector-proven).
    ///
    /// The virtual clock advances by the recipe's duration; timestamps in the
    /// emitted envelopes are device-uptime-like, exactly as on hardware.
    pub fn emit_ui_gesture(&self, gesture: UiGesture) -> Vec<String> {
        let mut now = self.state.lock().unwrap();
        PENDING.lock().unwrap().clear();

        // Leave a quiet gap from whatever came before.
        *now += 500;
        unsafe {
            match gesture {
                UiGesture::Tap => {
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_imu(0, 0, 900, 0, 0, 0);
                    ffi::gdc_feed_imu(0, 0, 0, 0, 0, 0);
                    *now += 500; // past the double-tap window → flush as single tap
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_imu(0, 0, 1, 0, 0, 0);
                }
                UiGesture::DoubleTap => {
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_imu(0, 0, 901, 0, 0, 0);
                    ffi::gdc_feed_imu(0, 0, 1, 0, 0, 0);
                    *now += 200;
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_imu(0, 0, 901, 0, 0, 0);
                }
                UiGesture::RotateCw | UiGesture::RotateCcw => {
                    let gx = if gesture == UiGesture::RotateCw {
                        30_000
                    } else {
                        -30_000
                    };
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_imu(0, 0, 900, gx, 0, 0);
                    *now += 200;
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_imu(0, 0, 899, gx, 0, 0);
                }
                UiGesture::SwipeLeft | UiGesture::SwipeRight => {
                    let code = if gesture == UiGesture::SwipeLeft {
                        TOUCH_SWIPE_LEFT
                    } else {
                        TOUCH_SWIPE_RIGHT
                    };
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_touch(1, 100, code);
                    *now += 50;
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_touch(0, 100, TOUCH_NONE);
                }
                UiGesture::Hold { duration_ms } => {
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_touch(1, 200, TOUCH_NONE);
                    *now += duration_ms.max(1) as i64;
                    ffi::gdc_set_time_ms(*now);
                    ffi::gdc_feed_touch(0, 200, TOUCH_NONE);
                }
            }
        }
        std::mem::take(&mut *PENDING.lock().unwrap())
    }

    /// Replays a conformance `.scenario` script and renders output in the
    /// exact line format of the firmware's `gen_vectors.c` — the parity test
    /// diffs this against the checked-in `.expected` golden vectors.
    pub fn replay_scenario(&self, scenario: &str) -> Result<String, String> {
        let mut now = self.state.lock().unwrap();
        PENDING.lock().unwrap().clear();

        let mut out = String::new();
        out.push_str(&format!("# {}\n", unsafe {
            CStr::from_ptr(ffi::gdc_version()).to_string_lossy()
        }));

        let flush_events = |out: &mut String| {
            for e in PENDING.lock().unwrap().drain(..) {
                out.push_str("EVENT ");
                out.push_str(&e);
                out.push('\n');
            }
        };

        for line in scenario.lines() {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("T") => {
                    let t: i64 = parts
                        .next()
                        .and_then(|v| v.parse().ok())
                        .ok_or_else(|| format!("bad T line: {line}"))?;
                    *now = t;
                    unsafe { ffi::gdc_set_time_ms(t) };
                }
                Some("IMU") => {
                    let v: Vec<i32> = parts.filter_map(|p| p.parse().ok()).collect();
                    if v.len() != 6 {
                        return Err(format!("bad IMU line: {line}"));
                    }
                    unsafe { ffi::gdc_feed_imu(v[0], v[1], v[2], v[3], v[4], v[5]) };
                    flush_events(&mut out);
                }
                Some("TOUCH") => {
                    let v: Vec<i32> = parts.filter_map(|p| p.parse().ok()).collect();
                    if v.len() != 3 {
                        return Err(format!("bad TOUCH line: {line}"));
                    }
                    unsafe { ffi::gdc_feed_touch(v[0], v[1] as u16, v[2]) };
                    flush_events(&mut out);
                }
                Some("CFG") => {
                    let bytes: Vec<u8> = parts
                        .filter_map(|p| u8::from_str_radix(p, 16).ok())
                        .collect();
                    unsafe { ffi::gdc_write_config(bytes.as_ptr(), bytes.len()) };
                    let mut rb = [0u8; 4];
                    unsafe { ffi::gdc_read_config(rb.as_mut_ptr()) };
                    let hid = unsafe { ffi::gdc_hid_enabled() };
                    out.push_str(&format!(
                        "CFGREAD {:02x} {:02x} {:02x} {:02x} hid={}\n",
                        rb[0], rb[1], rb[2], rb[3], hid
                    ));
                }
                Some("HAPTIC") => {
                    let json = line[7..].trim_end();
                    let mut raw = ffi::GdcHapticResult {
                        pattern: 0,
                        sequence: 0,
                        samples: [0; 1024],
                        n_samples: 0,
                        sample_rate_hz: 0,
                        ack_json: [0; 512],
                        ack_len: 0,
                    };
                    let err =
                        unsafe { ffi::gdc_decode_haptic(json.as_ptr(), json.len(), &mut raw) };
                    if err == 0 {
                        out.push_str(&format!(
                            "DECODED pattern={} seq={} n_samples={} rate={}\n",
                            raw.pattern, raw.sequence, raw.n_samples, raw.sample_rate_hz
                        ));
                    } else {
                        out.push_str(&format!("REJECTED err={err}\n"));
                    }
                    let ack_len = raw.ack_len.min(511);
                    let ack = unsafe {
                        std::slice::from_raw_parts(raw.ack_json.as_ptr() as *const u8, ack_len)
                    };
                    out.push_str(&format!("ACK {}\n", String::from_utf8_lossy(ack)));
                }
                Some("SNAPSHOT") => {
                    let v: Vec<i32> = parts.filter_map(|p| p.parse().ok()).collect();
                    if v.len() != 5 {
                        return Err(format!("bad SNAPSHOT line: {line}"));
                    }
                    let mut buf = [0i8; 600];
                    let n = unsafe {
                        ffi::gdc_state_snapshot_json(
                            v[0] as u8,
                            v[1],
                            v[2] as u16,
                            v[3],
                            v[4],
                            buf.as_mut_ptr() as *mut c_char,
                            buf.len(),
                        )
                    };
                    if n > 0 {
                        let bytes = unsafe {
                            std::slice::from_raw_parts(buf.as_ptr() as *const u8, n as usize)
                        };
                        out.push_str(&format!("SNAPSHOT {}\n", String::from_utf8_lossy(bytes)));
                    }
                }
                // C3 sensor-frame vector (device-core v1.1):
                //   FRAME t0_ms period_ms touch_valid
                //   FS ax_mg ay_mg az_mg gx_mdps gy_mdps gz_mdps slider touched
                //   FEND  -> FRAME_HEX <wire bytes>
                Some("FRAME") => {
                    let v: Vec<i64> = parts.filter_map(|p| p.parse().ok()).collect();
                    if v.len() != 3 {
                        return Err(format!("bad FRAME line: {line}"));
                    }
                    unsafe { ffi::gdc_frame_begin(v[0] as u32, v[1] as u8, v[2] as c_int) };
                }
                Some("FS") => {
                    let v: Vec<i64> = parts.filter_map(|p| p.parse().ok()).collect();
                    if v.len() != 8 {
                        return Err(format!("bad FS line: {line}"));
                    }
                    unsafe {
                        ffi::gdc_frame_add(
                            v[0] as i32,
                            v[1] as i32,
                            v[2] as i32,
                            v[3] as i32,
                            v[4] as i32,
                            v[5] as i32,
                            v[6] as u16,
                            v[7] as c_int,
                        )
                    };
                }
                Some("FEND") => {
                    let mut buf = [0u8; 328];
                    let n = unsafe { ffi::gdc_frame_finish(buf.as_mut_ptr(), buf.len()) };
                    if n > 0 {
                        let hex: String = buf[..n as usize]
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect();
                        out.push_str(&format!("FRAME_HEX {hex}\n"));
                    }
                }
                _ => return Err(format!("unparsed scenario line: {line}")),
            }
        }
        flush_events(&mut out);
        Ok(out)
    }
}
