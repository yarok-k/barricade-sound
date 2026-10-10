use std::process::Command;


#[derive(Clone)]
pub struct AudioLib {
    pub volume: u32,
    pub mute: bool,
}

impl AudioLib {
    pub fn new() -> Self {
        let mut audio = Self {
            volume: 100,
            mute: false,
        };
        audio.update();

        audio
    }
    pub fn update(&mut self) {
        // Вывод: "Volume: 0.45" или "Volume: 0.45 [MUTED]"
        if let Some(s) = Self::wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
            self.mute = s.contains("[MUTED]");
            if let Some(v) = s.split_whitespace().nth(1).and_then(|v| v.parse::<f32>().ok()) {
                self.volume = (v * 100.0).round() as u32;
            }
        }
    }
    // устанавливает уровень гпромкости и мут
    pub fn set_volume(&mut self, pct: u32) {
        Self::wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", pct)]);
        self.volume = pct;
    }

    pub fn set_mute(&mut self, mute: bool) {
        Self::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if mute { "1" } else { "0" }]);
        self.mute = mute;
    }

    /// Возвращает название иконки в формате freedesktop.
    pub fn get_icon_name(&self) -> &'static str {
        if self.mute || self.volume <= 0 {
            return "audio-volume-muted";
        }
        match self.volume {
            1..=30 => "audio-volume-low",
            31..=70 => "audio-volume-medium",
            _ => "audio-volume-high",
        }
    }
    fn wpctl(args: &[&str]) -> Option<String> {
        let out = Command::new("wpctl").args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
