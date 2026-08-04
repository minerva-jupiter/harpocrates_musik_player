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

    let settings: Settings = Settings::load()?;
    let mut audioengine: AudioEngine = AudioEngine::new()?;
    audioengine.set_volume(settings.audio().volume());

    let raw_items = library::scan_library(settings.audio().library_path());
    let library = Library::build(raw_items);
    let mut state = AppState::new();

    loop {
        terminal.draw(|f| {
            ui::render(f, &library, &mut state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('p') => {
                        if audioengine.is_paused() {
                            audioengine.play();
                        } else {
                            audioengine.pause();
                        }
                    }
                    KeyCode::Left => {
                        state.focus = match state.focus {
                            Focus::Artist => Focus::Artist,
                            Focus::Album => Focus::Artist,
                            Focus::Track => Focus::Album,
                        };
                    }
                    KeyCode::Right => {
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
                        };
                    }
                    KeyCode::Up => {
                        let current_state = match state.focus {
                            Focus::Artist => &mut state.artist_state,
                            Focus::Album => &mut state.album_state,
                            Focus::Track => &mut state.track_state,
                        };
                        let i = current_state.selected().unwrap_or(0);
                        if i > 0 {
                            current_state.select(Some(i - 1));
                        }
                    }
                    KeyCode::Down => {
                        // カウントを取得するためにデータを取得
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
                        };

                        let current_state = match state.focus {
                            Focus::Artist => &mut state.artist_state,
                            Focus::Album => &mut state.album_state,
                            Focus::Track => &mut state.track_state,
                        };
                        let i = current_state.selected().unwrap_or(0);
                        if max_len > 0 && i < max_len - 1 {
                            current_state.select(Some(i + 1));
                        }
                    }
                    KeyCode::Enter => {
                        // 再生ロジック
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
                                        if let Some(track) =
                                            state.track_state.selected().and_then(|i| tracks.get(i))
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
                    _ => {}
                }
            }
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
