use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::library::Library;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    Artist,
    Album,
    Track,
}

pub struct AppState {
    pub focus: Focus,
    pub artist_state: ListState,
    pub album_state: ListState,
    pub track_state: ListState,
}

impl AppState {
    pub fn new() -> Self {
        let mut artist_state = ListState::default();
        artist_state.select(Some(0));
        Self {
            focus: Focus::Artist,
            artist_state,
            album_state: ListState::default(),
            track_state: ListState::default(),
        }
    }
}

pub fn render(f: &mut Frame, library: &Library, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(f.area());

    // 1. Artists
    let artists: Vec<String> = library.artists.keys().cloned().collect();
    let artist_items: Vec<ListItem> = artists
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    render_list(
        f,
        "Artist",
        artist_items,
        chunks[0],
        state.focus == Focus::Artist,
        &mut state.artist_state,
    );

    // 2. Albums (selected artist)
    let selected_artist = state
        .artist_state
        .select_index()
        .and_then(|i| artists.get(i));
    let mut albums: Vec<String> = Vec::new();
    if let Some(artist) = selected_artist {
        if let Some(artist_data) = library.artists.get(artist) {
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
        state.focus == Focus::Album,
        &mut state.album_state,
    );

    // 3. Tracks (selected album)
    let selected_album = state.album_state.select_index().and_then(|i| albums.get(i));
    let mut tracks: Vec<String> = Vec::new();
    if let (Some(artist), Some(album)) = (selected_artist, selected_album) {
        if let Some(album_data) = library.artists.get(artist).and_then(|a| a.get(album)) {
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
        state.focus == Focus::Track,
        &mut state.track_state,
    );
}

fn render_list(
    f: &mut Frame,
    title: &str,
    items: Vec<ListItem>,
    area: ratatui::layout::Rect,
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

    f.render_stateful_widget(list, area, state);
}

// Helper trait to get selection index safely
trait ListStateExt {
    fn select_index(&self) -> Option<usize>;
}
impl ListStateExt for ListState {
    fn select_index(&self) -> Option<usize> {
        self.selected()
    }
}
