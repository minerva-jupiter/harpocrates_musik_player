pub mod library_ui;
pub mod setting_ui;

use crossterm::event::KeyEvent;
use ratatui::{Frame, widgets::ListState};

use crate::app::App;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    Artist,
    Album,
    Track,
    Settings,
}

pub struct AppState {
    pub focus: Focus,
    pub artist_state: ListState,
    pub album_state: ListState,
    pub track_state: ListState,
    pub settings_state: ListState,
    pub is_device_dialog_open: bool,
    pub available_devices: Vec<String>,
    pub device_dialog_state: ListState,
}

impl AppState {
    pub fn new() -> Self {
        let mut artist_state = ListState::default();
        artist_state.select(Some(0));
        let mut settings_state = ListState::default();
        settings_state.select(Some(0));
        let mut device_dialog_state = ListState::default();
        device_dialog_state.select(Some(0));
        Self {
            focus: Focus::Artist,
            artist_state,
            album_state: ListState::default(),
            track_state: ListState::default(),
            settings_state,
            is_device_dialog_open: false,
            available_devices: Vec::new(),
            device_dialog_state,
        }
    }

    pub fn open_device_dialog(&mut self, devices: Vec<String>) {
        self.available_devices = devices;
        self.is_device_dialog_open = true;
        self.device_dialog_state.select(Some(0));
    }

    pub fn close_device_dialog(&mut self) {
        self.is_device_dialog_open = false;
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    if app.state.focus == Focus::Settings {
        setting_ui::render(f, app);
    } else {
        library_ui::render(f, app);
    }
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    if app.state.focus == Focus::Settings {
        setting_ui::handle_key_event(app, key)
    } else {
        library_ui::handle_key_event(app, key)
    }
}
