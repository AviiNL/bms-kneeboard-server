use std::path::PathBuf;
use winreg::{enums::*, *};

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub base_dir: PathBuf,
    pub theater: String,
    pub callsign: String,
    pub name: String,
}

impl GameInfo {
    pub fn new() -> Option<GameInfo> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let cur_ver = hklm
            .open_subkey("SOFTWARE\\WOW6432Node\\Benchmark Sims\\Falcon BMS 4.37")
            .ok()?;
        let base_dir: String = cur_ver.get_value("baseDir").ok()?;
        let theater: String = cur_ver.get_value("curTheater").ok()?;
        let callsign: Vec<u8> = cur_ver.get_raw_value("PilotCallsign").ok()?.bytes;
        let callsign = String::from_utf8_lossy(&callsign)
            .replace('\0', " ")
            .trim()
            .to_string();
        let name: Vec<u8> = cur_ver.get_raw_value("PilotName").ok()?.bytes;
        let name = String::from_utf8_lossy(&name)
            .replace('\0', " ")
            .trim()
            .to_string();

        Some(GameInfo {
            base_dir: base_dir.into(),
            theater,
            callsign,
            name,
        })
    }
}
