fn main() {
    #[cfg(feature = "tauri-gui")]
    tauri_build::build();

    #[cfg(feature = "device-core")]
    build_device_core();
}

/// Compiles the ring firmware's ACTUAL decision-making sources (gesture
/// engine, semantic layer, wire codec) into the simulator, per
/// `device_core/README.md` in the firmware repo. Not a reimplementation —
/// behavior is firmware-identical by construction.
///
/// Firmware repo location: `GESTURA_FIRMWARE_DIR` env var, defaulting to the
/// sibling checkout `../haptic-basic-firmware`. Build without the firmware
/// repo via `--no-default-features` (the simulator then falls back to its
/// legacy behavioral emulation).
#[cfg(feature = "device-core")]
fn build_device_core() {
    use std::path::PathBuf;

    let fw_dir = std::env::var("GESTURA_FIRMWARE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../haptic-basic-firmware"));

    let header = fw_dir.join("device_core/gestura_device_core.h");
    if !header.exists() {
        panic!(
            "device-core feature: firmware repo not found at {} (checked for {}).\n\
             Set GESTURA_FIRMWARE_DIR to your haptic-basic-firmware checkout, or build \n\
             with --no-default-features to use the legacy behavioral emulation.",
            fw_dir.display(),
            header.display()
        );
    }

    let sources = [
        "device_core/core.c",
        "src/gesture/gesture_engine.c",
        "src/protocol/semantic.c",
        "src/protocol/sensor_frame.c",
        "src/protocol/wire_json.c",
    ];

    let mut build = cc::Build::new();
    for src in sources {
        let path = fw_dir.join(src);
        println!("cargo:rerun-if-changed={}", path.display());
        build.file(path);
    }
    println!("cargo:rerun-if-changed={}", header.display());
    println!("cargo:rerun-if-env-changed=GESTURA_FIRMWARE_DIR");

    build
        .include(fw_dir.join("device_core"))
        .include(fw_dir.join("src"))
        .include(fw_dir.join("tests/stubs"))
        .std("c11")
        .opt_level(2)
        .warnings(true)
        // Suppress cc's default `-l static=...` so we can emit the directive
        // ourselves with the whole-archive modifier below.
        .cargo_metadata(false)
        .compile("gestura_device_core");

    // Link with +whole-archive: the crate's bin/test targets reference the
    // gdc_* symbols from a separate module copy, and a plain `-l static`
    // is order-sensitive — the linker can process the archive before the
    // objects that need it and drop the members. whole-archive pulls every
    // member regardless of order, so all targets link deterministically.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static:+whole-archive=gestura_device_core");
    // rustc-link-lib from a build script attaches to the LIB target only.
    // The bin compiles its own copy of the device_core module (main.rs
    // `mod device_core`), and its gdc_* references never see the archive:
    // the bin links only while that copy is dead code (default features)
    // and fails under cli-only. Feed bin targets the archive directly.
    println!("cargo:rustc-link-arg-bins={out_dir}/libgestura_device_core.a");
}
