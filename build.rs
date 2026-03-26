// build.rs — Embed application icon into the Windows executable.
//
// This script runs at compile time (before main.rs is built).
// It uses the `winres` crate to attach `assets/icon.ico` as a Win32
// resource, which makes the icon appear in:
//   • Explorer file icon
//   • Taskbar / Alt-Tab thumbnail
//   • Window title bar (small icon)
//
// On non-Windows targets (Linux cross-compile, CI) the script does nothing
// so the build still succeeds.

fn main() {
    // Only embed resources when targeting Windows.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        // Optional: version info visible in Explorer → Properties → Details
        res.set("FileDescription", "Roblox Fast Flags Editor");
        res.set("ProductName",     "Roblox Fast Flags Editor");
        res.set("LegalCopyright",  "");
        if let Err(e) = res.compile() {
            // Warn but don't abort — exe still works, just without the icon.
            eprintln!("cargo:warning=winres failed (icon will be missing): {e}");
        }
    }
}