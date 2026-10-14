fn main() {
    tauri_build::build();
    embed_test_manifest();
}

/// On Windows, test executables that link Tauri fail to start (STATUS_ENTRYPOINT_NOT_FOUND)
/// unless they carry the Common Controls v6 manifest the application itself embeds.
fn embed_test_manifest() {
    println!("cargo:rerun-if-changed=windows-test.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{manifest_dir}\\windows-test.manifest");
}
