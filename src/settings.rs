use std::{collections::HashMap, error::Error, fs::*, io::Write, path::PathBuf, rc::Rc};

use serde::*;

use crate::{
    hotkeys::HotkeyMacro,
    modals::{warning_window::DefaultErrorWindow, ModalWindow},
};

pub const SETTINGS_FILE_NAME: &'static str = "fluffy-macro-recorder-settings.txt";

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub record_mouse_movement: bool,
    pub record_mouse_offsets: bool,
    pub playback_speed: f32,
    pub ignore_delays: bool,
    pub repeat_times: u32,
    pub hotkeys: Vec<HotkeyMacro>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            record_mouse_movement: true,
            record_mouse_offsets: false,
            playback_speed: 1.0,
            ignore_delays: false,
            repeat_times: 1,
            hotkeys: vec![],
        }
    }
}

impl Settings {
    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(SETTINGS_FILE_NAME)?;

        let settings = serde_json::to_string(self).unwrap();

        file.write_all(settings.as_bytes())?;

        Ok(())
    }

    pub fn save_with_error_window(&self) -> Option<Rc<dyn ModalWindow>> {
        let result = self.save_to_file();
        if let Err(error) = result {
            Some(DefaultErrorWindow::new(
                "Settings Error".into(),
                vec![
                    "Error saving settings to file:".into(),
                    error.to_string(),
                    "The macro recorder will still function, but the settings will not be saved at next startup.".into()
                ],
            ))
        } else {
            None
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
