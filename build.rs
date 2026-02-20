#[cfg(target_os = "windows")]
use windres::Build;

#[cfg(target_os = "windows")]
fn main() {
    Build::new().compile("bms-kneeboard-server.rc").unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {}
