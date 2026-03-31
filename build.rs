//! build.rs — Embed icon + version info into the Windows .exe
//!
//! Uses `winres` to attach a Win32 VERSIONINFO resource and the app icon.
//! This makes the icon appear in Windows Explorer and the Properties dialog.
//!
//! Cross-compile note (Linux → x86_64-pc-windows-gnu):
//!   `winres` needs `windres`. When the host is Linux, we explicitly use
//!   `x86_64-w64-mingw32-windres` instead of the host `windres`.
//!   Detection: if `CARGO_CFG_TARGET_OS=windows` but `cfg!(target_os)` is
//!   NOT windows, we are cross-compiling.
//!
//! Runtime icon fallback: even if this step fails, main.rs sets the window
//! icon via `viewport.with_icon()` which always works.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");

    // Only embed Win32 resources when the *target* is Windows.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = winres::WindowsResource::new();

    // Cross-compiling from a non-Windows host?
    // Use the MinGW windres rather than whatever `windres` is on the host PATH.
    let host_is_windows = cfg!(target_os = "windows");
    if !host_is_windows {
        res.set_windres_path("x86_64-w64-mingw32-windres");
        res.set_ar_path("x86_64-w64-mingw32-ar");
    }

    res.set_icon("assets/icon.ico");
    res.set("FileDescription", "Roblox Fast Flag Editor");
    res.set("ProductName",     "Roblox Fast Flag Editor");
    res.set("FileVersion",     env!("CARGO_PKG_VERSION"));
    res.set("ProductVersion",  env!("CARGO_PKG_VERSION"));
    res.set("LegalCopyright",  "");

    if let Err(e) = res.compile() {
        println!("cargo:warning=winres could not embed icon: {e}");
        println!("cargo:warning=Tip: apt install gcc-mingw-w64 for file icon support.");
    }
}