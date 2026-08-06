use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    audio::{AudioEngine, AudioPlayer},
    library::Library,
    setting::Settings,
    ui::{self, AppState, Focus},
};

pub struct App {
    pub state: AppState,
    pub settings: Settings,
    pub library: Library,
    pub audioengine: AudioEngine,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings = Settings::load()?;
        let mut audioengine = AudioEngine::new(settings.audio())?;
        audioengine.set_volume(settings.audio().volume());

        let raw_items = crate::library::scan_library(settings.audio().library_path());
        let library = Library::build(raw_items);

        Ok(Self {
            state: AppState::new(),
            settings,
            library,
            audioengine,
        })
    }

    pub fn recreate_audio_engine(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut new_engine = AudioEngine::new(self.settings.audio())?;
        new_engine.set_volume(self.settings.audio().volume());
        self.audioengine = new_engine;
        Ok(())
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        // 全画面共通のショートカット操作を先に判定
        if !self.state.is_device_dialog_open {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('s') => {
                    self.state.focus = if self.state.focus == Focus::Settings {
                        Focus::Artist
                    } else {
                        Focus::Settings
                    };
                    return Ok(false);
                }
                KeyCode::Char('p') => {
                    if self.audioengine.is_paused() {
                        self.audioengine.play();
                    } else {
                        self.audioengine.pause();
                    }
                    return Ok(false);
                }
                _ => {}
            }
        }

        // 画面個別の処理へ委譲
        ui::handle_key_event(self, key)
    }
}
