use std::{error::Error, fs::*, io::Write};

use serde::*;

pub const SETTINGS_FILE_NAME: &'static str = "fluffy-macro-recorder-settings.txt";

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub record_mouse_movement: bool,
    pub playback_speed: f32,
    pub ignore_delays: bool,
    pub repeat_times: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            record_mouse_movement: true,
            playback_speed: 1.0,
            ignore_delays: false,
            repeat_times: 1,
        }
    }
}

pub fn load_settings() -> Result<Settings, Box<dyn Error>> {
    let settings = read_to_string(SETTINGS_FILE_NAME)?;
    let settings = serde_json::from_str(&settings)?;

    Ok(settings)
}

pub fn create_settings_file() -> Result<(), Box<dyn Error>> {
    let mut file = File::create(SETTINGS_FILE_NAME)?;
    let settings = serde_json::to_string(&Settings::default()).unwrap();

    file.write_all(settings.as_bytes())?;
    Ok(())
}

pub fn save_settings_file(settings: &Settings) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(SETTINGS_FILE_NAME)?;

    let settings = serde_json::to_string(settings).unwrap();

    file.write_all(settings.as_bytes())?;

    Ok(())
}
