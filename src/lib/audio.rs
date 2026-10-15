use std::process::Command;


#[derive(Clone)]
pub struct AudioLib {
    pub volume: u32,
    pub mute: bool,
}

pub struct  Sink {
    pub isdefault: bool,
    pub id: i32,
    pub name: String
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
    // устанавливает уровень гпромкости передать значение от 0 до 100
    pub fn set_volume(&mut self, pct: u32) {
        Self::wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", pct)]);
        self.volume = pct;
    }
    pub fn get_volume(&mut self) -> u32 {
        self.update();
        return self.volume
    }

    pub fn set_mute(&mut self, mute: bool) {
        Self::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if mute { "1" } else { "0" }]);
        self.mute = mute;
    }
    pub fn switch_mute(&mut self) {
        Self::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if !self.mute { "1" } else { "0" }]);
        self.mute = !self.mute;
    }
    pub fn get_mute(&mut self) -> bool {
        self.update();
        return self.mute
    }


    pub fn get_sinks(&mut self) -> Vec<Sink> {
        let Some(output) = Self::wpctl(&["status"]) else {
            return Vec::new();
        };

        let mut in_audio = false;
        let mut in_sinks = false;
        let mut sinks = Vec::new();

        for line in output.lines() {
            // Заголовок верхнего уровня (Audio / Video / Settings) — строка без отступа
            if line.chars().next().is_some_and(|c| c.is_alphabetic()) {
                in_audio = line.trim() == "Audio";
                in_sinks = false;
                continue;
            }
            if !in_audio {
                continue;
            }

            // Убираем пробелы и символы псевдографики (│ ├─ └─)
            let content = line
                .trim_start_matches(|c: char| {
                    c.is_whitespace() || ('\u{2500}'..='\u{257F}').contains(&c)
                })
                .trim_end();
            if content.is_empty() {
                continue;
            }

            // Подзаголовок: Devices:, Sinks:, Sink endpoints:, Sources:, Streams: ...
            if let Some(section) = content.strip_suffix(':') {
                in_sinks = section == "Sinks";
                continue;
            }

            if in_sinks {
                if let Some(sink) = Self::parse_sink_line(content) {
                    sinks.push(sink);
                }
            }
        }

        sinks
    }

    pub fn switch_sink(sink_id:i32){

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

    // технические функции нужные для выполнения других
    fn wpctl(args: &[&str]) -> Option<String> {
        let out = Command::new("wpctl").args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    fn parse_sink_line(s: &str) -> Option<Sink> {
        // "*   51. Name [vol: 0.40]" или "52. Name [vol: 1.00 MUTED]"
        let (isdefault, s) = match s.strip_prefix('*') {
            Some(rest) => (true, rest.trim_start()),
            None => (false, s),
        };

        let (id_str, rest) = s.split_once('.')?;
        let id = id_str.trim().parse::<i32>().ok()?;

        let rest = rest.trim();
        // Отрезаем только хвостовой блок "[vol: ...]"
        let name = match rest.rfind(" [") {
            Some(i) if rest.ends_with(']') => &rest[..i],
            _ => rest,
        };

        Some(Sink {
            isdefault,
            id,
            name: name.trim().to_string(),
        })
    }
}
