mod audio;
mod library;
mod setting;
mod ui;

use std::{io, time::Duration};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
    audio::{AudioEngine, AudioPlayer},
    library::Library,
    setting::Settings,
    ui::{AppState, Focus},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut settings: Settings = Settings::load()?;
    let mut audioengine: AudioEngine = AudioEngine::new(settings.audio())?;
    audioengine.set_volume(settings.audio().volume());

    let raw_items = library::scan_library(settings.audio().library_path());
    let library = Library::build(raw_items);
    let mut state = AppState::new();

    loop {
        terminal.draw(|f| {
            ui::render(f, &library, &mut state, &settings);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if state.is_device_dialog_open {
                    match key.code {
                        KeyCode::Up => {
                            let i = state.device_dialog_state.selected().unwrap_or(0);
                            if i > 0 {
                                state.device_dialog_state.select(Some(i - 1));
                            }
                        }
                        KeyCode::Down => {
                            let i = state.device_dialog_state.selected().unwrap_or(0);
                            if !state.available_devices.is_empty()
                                && i < state.available_devices.len() - 1
                            {
                                state.device_dialog_state.select(Some(i + 1));
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(i) = state.device_dialog_state.selected() {
                                if let Some(device_name) = state.available_devices.get(i) {
                                    settings.audio_mut().set_output_device(device_name.clone());
                                    audioengine = AudioEngine::new(settings.audio())?;
                                    audioengine.set_volume(settings.audio().volume());
                                    settings.save()?;
                                }
                            }
                            state.close_device_dialog();
                        }
                        KeyCode::Esc => {
                            state.close_device_dialog();
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('s') => {
                        state.focus = if state.focus == Focus::Settings {
                            Focus::Artist
                        } else {
                            Focus::Settings
                        };
                    }
                    KeyCode::Char('p') => {
                        if audioengine.is_paused() {
                            audioengine.play();
                        } else {
                            audioengine.pause();
                        }
                    }
                    KeyCode::Left => {
                        if state.focus == Focus::Settings {
                            let selected_setting = state.settings_state.selected().unwrap_or(0);
                            if selected_setting == 0 {
                                let current_volume = settings.audio().volume();
                                let new_volume = (current_volume - 0.01).max(0.0);
                                settings.audio_mut().set_volume(new_volume);
                                audioengine.set_volume(new_volume);
                                settings.save()?;
                            }
                        } else {
                            state.focus = match state.focus {
                                Focus::Artist => Focus::Artist,
                                Focus::Album => Focus::Artist,
                                Focus::Track => Focus::Album,
                                Focus::Settings => Focus::Settings,
                            };
                        }
                    }
                    KeyCode::Right => {
                        if state.focus == Focus::Settings {
                            let selected_setting = state.settings_state.selected().unwrap_or(0);
                            if selected_setting == 0 {
                                let current_volume = settings.audio().volume();
                                let new_volume = (current_volume + 0.01).min(1.0);
                                settings.audio_mut().set_volume(new_volume);
                                audioengine.set_volume(new_volume);
                                settings.save()?;
                            }
                        } else {
                            state.focus = match state.focus {
                                Focus::Artist => {
                                    state.album_state.select(Some(0));
                                    Focus::Album
                                }
                                Focus::Album => {
                                    state.track_state.select(Some(0));
                                    Focus::Track
                                }
                                Focus::Track => Focus::Track,
                                Focus::Settings => Focus::Settings,
                            };
                        }
                    }
                    KeyCode::Up => {
                        if state.focus == Focus::Settings {
                            let i = state.settings_state.selected().unwrap_or(0);
                            if i > 0 {
                                state.settings_state.select(Some(i - 1));
                            }
                        } else {
                            let current_state = match state.focus {
                                Focus::Artist => &mut state.artist_state,
                                Focus::Album => &mut state.album_state,
                                Focus::Track => &mut state.track_state,
                                Focus::Settings => &mut state.artist_state,
                            };
                            let i = current_state.selected().unwrap_or(0);
                            if i > 0 {
                                current_state.select(Some(i - 1));
                            }
                        }
                    }
                    KeyCode::Down => {
                        if state.focus == Focus::Settings {
                            let max_len = 3;
                            let i = state.settings_state.selected().unwrap_or(0);
                            if i < max_len - 1 {
                                state.settings_state.select(Some(i + 1));
                            }
                        } else {
                            let max_len = match state.focus {
                                Focus::Artist => library.artists.len(),
                                Focus::Album => {
                                    let artists: Vec<_> = library.artists.keys().collect();
                                    state
                                        .artist_state
                                        .selected()
                                        .and_then(|i| artists.get(i))
                                        .and_then(|a| library.artists.get(*a))
                                        .map(|m| m.len())
                                        .unwrap_or(0)
                                }
                                Focus::Track => {
                                    let artists: Vec<_> = library.artists.keys().collect();
                                    let artist =
                                        state.artist_state.selected().and_then(|i| artists.get(i));
                                    let albums: Vec<_> = artist
                                        .and_then(|a| library.artists.get(*a))
                                        .map(|m| m.keys().collect())
                                        .unwrap_or_default();
                                    let album =
                                        state.album_state.selected().and_then(|i| albums.get(i));
                                    artist
                                        .and_then(|a| library.artists.get(*a))
                                        .and_then(|m| album.and_then(|al| m.get(*al)))
                                        .map(|v| v.len())
                                        .unwrap_or(0)
                                }
                                Focus::Settings => 0,
                            };

                            let current_state = match state.focus {
                                Focus::Artist => &mut state.artist_state,
                                Focus::Album => &mut state.album_state,
                                Focus::Track => &mut state.track_state,
                                Focus::Settings => &mut state.artist_state,
                            };
                            let i = current_state.selected().unwrap_or(0);
                            if max_len > 0 && i < max_len - 1 {
                                current_state.select(Some(i + 1));
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if state.focus == Focus::Settings {
                            let selected_setting = state.settings_state.selected().unwrap_or(0);
                            if selected_setting == 2 {
                                let devices = audio::DeviceManager::list_device_names();
                                state.open_device_dialog(devices);
                            }
                        } else {
                            let artists: Vec<_> = library.artists.keys().collect();
                            if let Some(artist_name) =
                                state.artist_state.selected().and_then(|i| artists.get(i))
                            {
                                let artist_data = library.artists.get(*artist_name).unwrap();
                                let albums: Vec<_> = artist_data.keys().collect();

                                match state.focus {
                                    Focus::Track => {
                                        if let Some(album_name) =
                                            state.album_state.selected().and_then(|i| albums.get(i))
                                        {
                                            let tracks = artist_data.get(*album_name).unwrap();
                                            if let Some(track) = state
                                                .track_state
                                                .selected()
                                                .and_then(|i| tracks.get(i))
                                            {
                                                audioengine.stop();
                                                audioengine
                                                    .append_a_track(track.path.to_str().unwrap())?;
                                                audioengine.play();
                                            }
                                        }
                                    }
                                    Focus::Album => {
                                        if let Some(album_name) =
                                            state.album_state.selected().and_then(|i| albums.get(i))
                                        {
                                            let tracks = artist_data.get(*album_name).unwrap();
                                            audioengine.stop();
                                            for track in tracks {
                                                audioengine
                                                    .append_a_track(track.path.to_str().unwrap())?;
                                            }
                                            audioengine.play();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
