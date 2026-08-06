use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::{
    app::App,
    audio::{AudioEngine, DeviceManager},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let settings_items = vec![
        ListItem::new(format!("Volume: {:.2}", app.settings.audio().volume())),
        ListItem::new(format!(
            "Library Path: {}",
            app.settings.audio().library_path()
        )),
        ListItem::new(format!(
            "Output Device: {}",
            app.settings.audio().output_device()
        )),
    ];

    let list = List::new(settings_items)
        .block(
            Block::default()
                .title("Settings")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.state.settings_state);

    if app.state.is_device_dialog_open {
        render_device_dialog(f, app);
    }
}

fn render_device_dialog(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .state
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

    f.render_stateful_widget(list, area, &mut app.state.device_dialog_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    if app.state.is_device_dialog_open {
        return handle_dialog_key_event(app, key);
    }

    match key.code {
        KeyCode::Up => {
            let i = app.state.settings_state.selected().unwrap_or(0);
            if i > 0 {
                app.state.settings_state.select(Some(i - 1));
            }
        }
        KeyCode::Down => {
            let i = app.state.settings_state.selected().unwrap_or(0);
            if i < 2 {
                app.state.settings_state.select(Some(i + 1));
            }
        }
        KeyCode::Left => {
            if app.state.settings_state.selected() == Some(0) {
                let new_vol = (app.settings.audio().volume() - 0.1).max(0.0);
                app.settings.audio_mut().set_volume(new_vol);
                app.audioengine.set_volume(new_vol);
                app.settings.save()?;
            }
        }
        KeyCode::Right => {
            if app.state.settings_state.selected() == Some(0) {
                let new_vol = (app.settings.audio().volume() + 0.1).min(1.0);
                app.settings.audio_mut().set_volume(new_vol);
                app.audioengine.set_volume(new_vol);
                app.settings.save()?;
            }
        }
        KeyCode::Enter => {
            if app.state.settings_state.selected() == Some(2) {
                let devices = DeviceManager::list_device_names();
                app.state.open_device_dialog(devices);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_dialog_key_event(
    app: &mut App,
    key: KeyEvent,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Up => {
            let i = app.state.device_dialog_state.selected().unwrap_or(0);
            if i > 0 {
                app.state.device_dialog_state.select(Some(i - 1));
            }
        }
        KeyCode::Down => {
            let i = app.state.device_dialog_state.selected().unwrap_or(0);
            if !app.state.available_devices.is_empty() && i < app.state.available_devices.len() - 1
            {
                app.state.device_dialog_state.select(Some(i + 1));
            }
        }
        KeyCode::Enter => {
            if let Some(i) = app.state.device_dialog_state.selected() {
                if let Some(device_name) = app.state.available_devices.get(i) {
                    app.settings
                        .audio_mut()
                        .set_output_device(device_name.clone());
                    app.recreate_audio_engine()?;
                    app.settings.save()?;
                }
            }
            app.state.close_device_dialog();
        }
        KeyCode::Esc => {
            app.state.close_device_dialog();
        }
        _ => {}
    }
    Ok(false)
}
