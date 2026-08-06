use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::{app::App, ui::Focus};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(f.area());

    let artists: Vec<String> = app.library.artists.keys().cloned().collect();
    let artist_items: Vec<ListItem> = artists
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();
    render_list(
        f,
        "Artist",
        artist_items,
        chunks[0],
        app.state.focus == Focus::Artist,
        &mut app.state.artist_state,
    );

    let selected_artist = app
        .state
        .artist_state
        .selected()
        .and_then(|i| artists.get(i));
    let mut albums: Vec<String> = Vec::new();
    if let Some(artist) = selected_artist {
        if let Some(artist_data) = app.library.artists.get(artist) {
            albums = artist_data.keys().cloned().collect();
        }
    }
    let album_items: Vec<ListItem> = albums
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();
    render_list(
        f,
        "Album",
        album_items,
        chunks[1],
        app.state.focus == Focus::Album,
        &mut app.state.album_state,
    );

    let selected_album = app.state.album_state.selected().and_then(|i| albums.get(i));
    let mut tracks: Vec<String> = Vec::new();
    if let (Some(artist), Some(album)) = (selected_artist, selected_album) {
        if let Some(album_data) = app.library.artists.get(artist).and_then(|a| a.get(album)) {
            tracks = album_data.iter().map(|t| t.title.clone()).collect();
        }
    }
    let track_items: Vec<ListItem> = tracks
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();
    render_list(
        f,
        "Track",
        track_items,
        chunks[2],
        app.state.focus == Focus::Track,
        &mut app.state.track_state,
    );
}

fn render_list(
    frame: &mut Frame,
    title: &str,
    items: Vec<ListItem>,
    area: Rect,
    focused: bool,
    state: &mut ListState,
) {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, state);
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Left => {
            app.state.focus = match app.state.focus {
                Focus::Artist => Focus::Artist,
                Focus::Album => Focus::Artist,
                Focus::Track => Focus::Album,
                Focus::Settings => Focus::Settings,
            };
        }
        KeyCode::Right => {
            app.state.focus = match app.state.focus {
                Focus::Artist => {
                    app.state.album_state.select(Some(0));
                    Focus::Album
                }
                Focus::Album => {
                    app.state.track_state.select(Some(0));
                    Focus::Track
                }
                Focus::Track => Focus::Track,
                Focus::Settings => Focus::Settings,
            };
        }
        KeyCode::Up => {
            let current_state = match app.state.focus {
                Focus::Artist => &mut app.state.artist_state,
                Focus::Album => &mut app.state.album_state,
                Focus::Track => &mut app.state.track_state,
                Focus::Settings => &mut app.state.artist_state,
            };
            let i = current_state.selected().unwrap_or(0);
            if i > 0 {
                current_state.select(Some(i - 1));
            }
        }
        KeyCode::Down => {
            let max_len = match app.state.focus {
                Focus::Artist => app.library.artists.len(),
                Focus::Album => {
                    let artists: Vec<_> = app.library.artists.keys().collect();
                    app.state
                        .artist_state
                        .selected()
                        .and_then(|i| artists.get(i))
                        .and_then(|a| app.library.artists.get(*a))
                        .map(|m| m.len())
                        .unwrap_or(0)
                }
                Focus::Track => {
                    let artists: Vec<_> = app.library.artists.keys().collect();
                    let artist = app
                        .state
                        .artist_state
                        .selected()
                        .and_then(|i| artists.get(i));
                    let albums: Vec<_> = artist
                        .and_then(|a| app.library.artists.get(*a))
                        .map(|m| m.keys().collect())
                        .unwrap_or_default();
                    let album = app.state.album_state.selected().and_then(|i| albums.get(i));
                    artist
                        .and_then(|a| app.library.artists.get(*a))
                        .and_then(|m| album.and_then(|al| m.get(*al)))
                        .map(|v| v.len())
                        .unwrap_or(0)
                }
                Focus::Settings => 0,
            };

            let current_state = match app.state.focus {
                Focus::Artist => &mut app.state.artist_state,
                Focus::Album => &mut app.state.album_state,
                Focus::Track => &mut app.state.track_state,
                Focus::Settings => &mut app.state.artist_state,
            };
            let i = current_state.selected().unwrap_or(0);
            if max_len > 0 && i < max_len - 1 {
                current_state.select(Some(i + 1));
            }
        }
        KeyCode::Enter => {
            let artists: Vec<_> = app.library.artists.keys().collect();
            if let Some(artist_name) = app
                .state
                .artist_state
                .selected()
                .and_then(|i| artists.get(i))
            {
                let artist_data = app.library.artists.get(*artist_name).unwrap();
                let albums: Vec<_> = artist_data.keys().collect();

                match app.state.focus {
                    Focus::Track => {
                        if let Some(album_name) =
                            app.state.album_state.selected().and_then(|i| albums.get(i))
                        {
                            let tracks = artist_data.get(*album_name).unwrap();
                            if let Some(track) =
                                app.state.track_state.selected().and_then(|i| tracks.get(i))
                            {
                                app.audioengine.stop();
                                app.audioengine
                                    .append_a_track(track.path.to_str().unwrap())?;
                                app.audioengine.play();
                            }
                        }
                    }
                    Focus::Album => {
                        if let Some(album_name) =
                            app.state.album_state.selected().and_then(|i| albums.get(i))
                        {
                            let tracks = artist_data.get(*album_name).unwrap();
                            app.audioengine.stop();
                            for track in tracks {
                                app.audioengine
                                    .append_a_track(track.path.to_str().unwrap())?;
                            }
                            app.audioengine.play();
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(false)
}
