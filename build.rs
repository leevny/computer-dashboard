fn main() {
    // Embed the Windows app icon (assets/tray.res is a pre-built COFF resource,
    // generated once by windres from assets/tray.rc — no rc.exe/windres needed
    // on the building machine).
    if std::env::var("CARGO_CFG_TARGET_OS").map(|v| v == "windows").unwrap_or(false) {
        println!("cargo:rustc-link-arg=assets/tray.res");
    }
    println!("cargo:rerun-if-changed=assets/tray.res");
}
