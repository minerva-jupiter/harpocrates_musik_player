use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::library::Library;
use crate::setting::Settings;

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

pub fn render(f: &mut Frame, library: &Library, state: &mut AppState, settings: &Settings) {
    if state.focus == Focus::Settings {
        render_settings(f, state, settings);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(f.area());

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

    if state.is_device_dialog_open {
        render_device_dialog(f, state);
    }
}

pub fn render_settings(frame: &mut Frame, state: &mut AppState, settings: &Settings) {
    let area = frame.area();
    let settings_items = vec![
        ListItem::new(format!("Volume: {:.2}", settings.audio().volume())),
        ListItem::new(format!("Library Path: {}", settings.audio().library_path())),
        ListItem::new(format!(
            "Output Device: {}",
            settings.audio().output_device()
        )),
    ];

    let list = List::new(settings_items)
        .block(
            Block::default()
                .title("Settings")
                .borders(Borders::ALL)
                .border_style(if state.focus == Focus::Settings {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state.settings_state);
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

trait ListStateExt {
    fn select_index(&self) -> Option<usize>;
}

impl ListStateExt for ListState {
    fn select_index(&self) -> Option<usize> {
        self.selected()
    }
}

pub fn render_device_dialog(frame: &mut Frame, state: &mut AppState) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = state
        .available_devices
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Select Output Device")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state.device_dialog_state);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
