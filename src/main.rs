mod app;
mod audio;
mod ui;

use app::{App, Mode};
use audio::Audio;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

fn expand_home(raw: String) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    } else if raw == "~" {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(raw)
}

fn load_tracks(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut tracks = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", dir.display(), e)))?;

    for entry in entries {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("mp3") {
                tracks.push(path);
            }
        }
    }

    tracks.sort();
    Ok(tracks)
}

fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    let raw_dir = env::var("music_dir").expect("music_dir is not set");
    let dir = expand_home(raw_dir);

    let tracks = load_tracks(&dir)?;
    if tracks.is_empty() {
        println!("no tracks found");
        return Ok(());
    }

    let full_dir = fs::canonicalize(&dir).unwrap_or(dir);
    let mut audio = Audio::new();
    let mut terminal = ratatui::init();
    let res = run(&mut terminal, App::new(full_dir, tracks), &mut audio);
    ratatui::restore();
    res
}

fn run(terminal: &mut DefaultTerminal, mut app: App, audio: &mut Audio) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &mut app, audio))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let is_playing = app.current.is_some() && !audio.is_paused();

        match app.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('j') => app.move_by(1),
                KeyCode::Char('k') => app.move_by(-1),
                KeyCode::Up => {
                    if is_playing {
                        let vol = audio.change_volume(5.0);
                        app.status = format!("volume: {vol}%");
                    } else {
                        app.move_by(-1);
                    }
                }
                KeyCode::Down => {
                    if is_playing {
                        let vol = audio.change_volume(-5.0);
                        app.status = format!("volume: {vol}%");
                    } else {
                        app.move_by(1);
                    }
                }
                KeyCode::Left if is_playing => audio.seek(-5),
                KeyCode::Right if is_playing => audio.seek(5),
                KeyCode::Char('g') => app.select_first(),
                KeyCode::Char('G') => app.select_last(),
                KeyCode::Enter => app.play_selected(audio),
                KeyCode::Char(' ') => audio.toggle_pause(),
                KeyCode::Char(':') => {
                    app.mode = Mode::Command;
                    app.cmd.clear();
                }
                KeyCode::Char('/') => {
                    app.mode = Mode::Search;
                    app.query.clear();
                    app.filter();
                }
                KeyCode::Esc => {
                    app.query.clear();
                    app.filter();
                }
                _ => {}
            },
            Mode::Command => match key.code {
                KeyCode::Enter => {
                    if app.run_cmd(audio) {
                        return Ok(());
                    }
                    app.mode = Mode::Normal;
                }
                KeyCode::Esc => app.mode = Mode::Normal,
                KeyCode::Backspace => {
                    app.cmd.pop();
                }
                KeyCode::Char(c) => app.cmd.push(c),
                _ => {}
            },
            Mode::Search => match key.code {
                KeyCode::Enter | KeyCode::Esc => app.mode = Mode::Normal,
                KeyCode::Backspace => {
                    app.query.pop();
                    app.filter();
                }
                KeyCode::Char(c) => {
                    app.query.push(c);
                    app.filter();
                }
                _ => {}
            },
        }
    }
}

