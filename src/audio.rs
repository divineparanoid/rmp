use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

pub struct Audio {
    _sink: MixerDeviceSink,
    player: Player,
    current_path: Option<PathBuf>,
    volume: f32,
    pub duration: Option<Duration>,
}

impl Audio {
    pub fn new() -> Self {
        let mut sink = DeviceSinkBuilder::open_default_sink().expect("failed to open audio output");
        sink.log_on_drop(false);
        let player = Player::connect_new(sink.mixer());

        Self {
            _sink: sink,
            player,
            current_path: None,
            volume: 1.0,
            duration: None,
        }
    }

    pub fn play_file(&mut self, path: &Path) -> Result<(), &'static str> {
        let file = File::open(path).map_err(|_| "failed to open file")?;
        let source = Decoder::new(BufReader::new(file)).map_err(|_| "failed to decode mp3")?;

        self.duration = source.total_duration();
        self.current_path = Some(path.to_path_buf());

        self.player.stop();
        self.player = Player::connect_new(self._sink.mixer());
        self.player.set_volume(self.volume);
        self.player.append(source);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        if self.player.is_paused() {
            self.player.play();
        } else {
            self.player.pause();
        }
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn play(&self) {
        self.player.play();
    }

    pub fn stop(&mut self) {
        self.player.stop();
        self.duration = None;
        self.current_path = None;
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn pos(&self) -> Duration {
        self.player.get_pos()
    }

    pub fn set_volume(&mut self, percent: f32) {
        self.volume = percent.clamp(0.0, 100.0) / 100.0;
        self.player.set_volume(self.volume);
    }

    pub fn change_volume(&mut self, delta_percent: f32) -> u8 {
        let current = self.volume * 100.0;
        let next = (current + delta_percent).clamp(0.0, 100.0);
        self.set_volume(next);
        next.round() as u8
    }

    pub fn seek(&mut self, delta_secs: i64) {
        let Some(path) = self.current_path.clone() else { return };

        let pos = self.player.get_pos();
        let target = if delta_secs >= 0 {
            pos.saturating_add(Duration::from_secs(delta_secs as u64))
        } else {
            pos.saturating_sub(Duration::from_secs(delta_secs.unsigned_abs()))
        };

        let target = match self.duration {
            Some(total) => target.min(total),
            None => target,
        };

        if delta_secs > 0 && self.player.try_seek(target).is_ok() {
            return;
        }

        let was_paused = self.player.is_paused();
        let Ok(file) = File::open(&path) else { return };
        let Ok(source) = Decoder::new(BufReader::new(file)) else { return };

        self.player.stop();
        self.player = Player::connect_new(self._sink.mixer());
        self.player.set_volume(self.volume);
        if was_paused {
            self.player.pause();
        }
        self.player.append(source);
        let _ = self.player.try_seek(target);
    }
}

