use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    audio: AudioSettings,
    keybind: KeybindSettings,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AudioSettings {
    volume: f32,
    output_device: String,
    library_path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct KeybindSettings {
    play_pause: String,
    stop: String,
    next: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            audio: AudioSettings {
                volume: 1.0,
                output_device: std::env::var("DEFAULT_OUTPUT_DEVICE").unwrap_or_default(),
                library_path: std::env::var("DEFAULT_LIBRARY_PATH").unwrap_or_default(),
            },
            keybind: KeybindSettings {
                play_pause: "p".to_string(),
                stop: "s".to_string(),
                next: "n".to_string(),
            },
        }
    }
}

impl Settings {
    pub fn audio(&self) -> &AudioSettings {
        &self.audio
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_path().ok_or("Failed to get config directory path")?;
        if !path.exists() {
            return Ok(Settings::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Settings = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn get_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "harpocrates_musik_player").map(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = std::fs::create_dir_all(config_dir);
            }
            config_dir.join("config.toml")
        })
    }
}

impl AudioSettings {
    pub fn library_path(&self) -> &str {
        &self.library_path
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}
