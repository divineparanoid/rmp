use crate::{
    app::{App, Mode},
    audio::Audio,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    widgets::{List, ListItem, Paragraph},
};
use std::time::Duration;

pub fn draw(frame: &mut Frame, app: &mut App, audio: &Audio) {
    let [title_area, _, list_area, playing_area, bar_area, cmd_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(app.dir.as_str()).alignment(Alignment::Center),
        title_area,
    );

    let mut items = Vec::with_capacity(app.shown.len());
    for &idx in &app.shown {
        let name = app.tracks[idx]
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        items.push(ListItem::new(name));
    }

    let purple = Style::default().fg(Color::LightMagenta);
    let list = List::new(items)
        .highlight_style(purple)
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, list_area, &mut app.state);

    if let Some(name) = &app.current {
        let now_playing = if audio.is_paused() {
            format!("⏸  {name}")
        } else {
            format!("♪ {name}")
        };
        frame.render_widget(
            Paragraph::new(now_playing)
                .style(purple)
                .alignment(Alignment::Center),
            playing_area,
        );

        let bar = render_bar(audio.pos(), audio.duration, bar_area.width as usize);
        frame.render_widget(
            Paragraph::new(bar)
                .style(purple)
                .alignment(Alignment::Center),
            bar_area,
        );
    }

    let bottom = match app.mode {
        Mode::Command => format!(":{}", app.cmd),
        Mode::Search => format!("/{}", app.query),
        Mode::Normal => app.status.clone(),
    };
    frame.render_widget(Paragraph::new(bottom), cmd_area);
}

fn render_bar(pos: Duration, total: Option<Duration>, width: usize) -> String {
    let Some(total) = total else {
        return format_time(pos);
    };
    if total.is_zero() {
        return format_time(pos);
    }

    let pos = pos.min(total);
    let left = format_time(pos);
    let right = format_time(total);

    let overhead = left.len() + right.len() + 2;
    let bar_len = width.saturating_sub(overhead).clamp(10, 50);

    let ratio = pos.as_secs_f64() / total.as_secs_f64();
    let filled = ((bar_len as f64) * ratio).round() as usize;
    let filled = filled.min(bar_len.saturating_sub(1));

    let before = "━".repeat(filled);
    let after = "─".repeat(bar_len.saturating_sub(filled + 1));

    format!("{left} {before}●{after} {right}")
}

fn format_time(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

