use crate::audio::Audio;
use ratatui::widgets::ListState;
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Command,
    Search,
}

pub struct App {
    pub dir: String,
    pub tracks: Vec<PathBuf>,
    pub shown: Vec<usize>,
    pub state: ListState,
    pub mode: Mode,
    pub cmd: String,
    pub query: String,
    pub status: String,
    pub current: Option<String>,
}

impl App {
    pub fn new(dir: PathBuf, tracks: Vec<PathBuf>) -> Self {
        let shown: Vec<usize> = (0..tracks.len()).collect();
        let mut state = ListState::default();
        if !shown.is_empty() {
            state.select(Some(0));
        }

        Self {
            dir: dir.display().to_string(),
            tracks,
            shown,
            state,
            mode: Mode::Normal,
            cmd: String::new(),
            query: String::new(),
            status: "press :help for commands, / to search".to_string(),
            current: None,
        }
    }

    pub fn filter(&mut self) {
        let q = self.query.to_lowercase();
        self.shown.clear();

        for (i, path) in self.tracks.iter().enumerate() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.to_lowercase().contains(&q) {
                self.shown.push(i);
            }
        }

        if self.shown.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
    }

    pub fn move_by(&mut self, step: isize) {
        if self.shown.is_empty() {
            return;
        }
        let pos = self.state.selected().unwrap_or(0) as isize;
        let last = (self.shown.len() - 1) as isize;
        let next = (pos + step).clamp(0, last) as usize;
        self.state.select(Some(next));
    }

    pub fn select_first(&mut self) {
        if !self.shown.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.shown.is_empty() {
            self.state.select(Some(self.shown.len() - 1));
        }
    }

    pub fn play_selected(&mut self, audio: &mut Audio) {
        let Some(selected) = self.state.selected() else { return };
        let Some(&idx) = self.shown.get(selected) else { return };
        let path = &self.tracks[idx];

        match audio.play_file(path) {
            Ok(()) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                self.current = Some(name);
                self.status.clear();
            }
            Err(err) => self.status = err.to_string(),
        }
    }

    pub fn run_cmd(&mut self, audio: &mut Audio) -> bool {
        let args: Vec<&str> = self.cmd.trim().split_whitespace().collect();

        match args.as_slice() {
            ["q"] | ["quit"] => return true,
            ["c"] | ["close"] | ["stop"] => {
                audio.stop();
                self.current = None;
                self.status = "stopped".to_string();
            }
            ["h"] | ["help"] => {
                self.status = ":q quit | :c close track | :pause | :play | :vol <0-100> | / search | ←/→ seek".to_string();
            }
            ["pause"] => {
                audio.pause();
                self.status = "paused".to_string();
            }
            ["play"] => {
                audio.play();
                self.status = "playing".to_string();
            }
            ["vol", val] => match val.parse::<f32>() {
                Ok(v) => {
                    let clamped = v.clamp(0.0, 100.0);
                    audio.set_volume(clamped);
                    self.status = format!("volume: {}%", clamped as u8);
                }
                Err(_) => self.status = "usage: :vol <0-100>".to_string(),
            },
            [] => {}
            _ => self.status = format!("unknown command: {}", self.cmd.trim()),
        }

        false
    }
}

