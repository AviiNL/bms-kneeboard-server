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
    pub fn new(version: &str) -> Result<GameInfo, Box<dyn std::error::Error>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let versions = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Benchmark Sims")?;

        let cur_ver = versions.open_subkey(version)?;
        let base_dir: String = cur_ver.get_value("baseDir")?;
        let theater: String = cur_ver.get_value("curTheater")?;
        let callsign: Vec<u8> = cur_ver.get_raw_value("PilotCallsign")?.bytes;
        let callsign = String::from_utf8_lossy(&callsign)
            .replace('\0', " ")
            .trim()
            .to_string();
        let name: Vec<u8> = cur_ver.get_raw_value("PilotName")?.bytes;
        let name = String::from_utf8_lossy(&name)
            .replace('\0', " ")
            .trim()
            .to_string();

        Ok(GameInfo {
            base_dir: base_dir.into(),
            theater,
            callsign,
            name,
        })
    }

    pub fn versions() -> Vec<GameInfo> {
        let mut infos = vec![];

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let Ok(versions) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Benchmark Sims") else {
            return vec![];
        };

        for cur_ver in versions.enum_keys().flatten() {
            let Ok(gi) = Self::new(&cur_ver) else {
                continue;
            };

            infos.push(gi);
        }

        infos
    }
}
