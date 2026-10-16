use std::process::Command;

use anyhow::{Result,Error};
use dbus::arg::RefArg;


#[derive(Clone)]
pub struct AudioSinkLib {}
#[derive(Clone)]
pub struct AudioMicLib {}
pub struct TechZone {}

pub struct  Sink {
    pub isdefault: bool,
    pub id: i32,
    pub name: String
}

impl AudioSinkLib {
    // устанавливает уровень гпромкости передать значение от 0 до 100
    pub fn set_volume(pct: u32) {
        TechZone::wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", pct)]);
    }
    pub fn get_volume() -> u32 {
        let mut vol = 0;
        if let Some(s) = TechZone::wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
            if let Some(v) = s.split_whitespace().nth(1).and_then(|v| v.parse::<f32>().ok()) {
                vol = (v * 100.0).round() as u32;
            }
        }
        vol
    }

    pub fn set_mute(mute: bool) {
        TechZone::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if mute { "1" } else { "0" }]);
    }
    pub fn switch_mute() {
        TechZone::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if !Self::get_mute() { "1" } else { "0" }]);
    }
    pub fn get_mute() -> bool {
        let mut mute = false;
        if let Some(s) = TechZone::wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
            mute = s.contains("[MUTED]");
        }
        mute
    }


    pub fn get_sinks() -> Vec<Sink> {
        let Some(output) = TechZone::wpctl(&["status"]) else {
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

    pub fn set_sink(sink_id:i32) -> Result<(),Error>{
        TechZone::wpctl(&["set-default", &sink_id.to_string()]);
        Ok(())
    }

    /// Возвращает название иконки в формате freedesktop.
    pub fn get_sink_icon_name() -> &'static str {
        if Self::get_mute()|| Self::get_volume() <= 0 {
            return "audio-volume-muted";
        }
        match Self::get_volume() {
            1..=30 => "audio-volume-low",
            31..=70 => "audio-volume-medium",
            _ => "audio-volume-high",
        }
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

impl AudioMicLib {
    // устанавливает уровень гпромкости передать значение от 0 до 100
    pub fn set_gain(pct: u32) {
        TechZone::wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{}%", pct)]);
    }
    pub fn get_gain() -> u32 {
        let mut vol = 0;
        if let Some(s) = TechZone::wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
            if let Some(v) = s.split_whitespace().nth(1).and_then(|v| v.parse::<f32>().ok()) {
                vol = (v * 100.0).round() as u32;
            }
        }
        vol
    }

    pub fn set_mute(mute: bool) {
        TechZone::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if mute { "1" } else { "0" }]);
    }
    pub fn switch_mute() {
        TechZone::wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", if !Self::get_mute() { "1" } else { "0" }]);
    }
    pub fn get_mute() -> bool {
        let mut mute = false;
        if let Some(s) = TechZone::wpctl(&["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
            mute = s.contains("[MUTED]");
        }
        mute
    }


    pub fn get_mics() -> Vec<Sink> { //rework
        let Some(output) = TechZone::wpctl(&["status"]) else {
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

    pub fn set_mic(sink_id:i32) -> Result<(),Error>{
        TechZone::wpctl(&["set-default", &sink_id.to_string()]);
        Ok(())
    }

    /// Возвращает название иконки в формате freedesktop.
    pub fn get_mic_icon_name() -> &'static str { //rework
        if Self::get_mute()|| Self::get_volume() <= 0 {
            return "audio-volume-muted";
        }
        match Self::get_volume() {
            1..=30 => "audio-volume-low",
            31..=70 => "audio-volume-medium",
            _ => "audio-volume-high",
        }
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
impl TechZone{
    // технические функции нужные для выполнения других
    fn wpctl(args: &[&str]) -> Option<String> {
        let out = Command::new("wpctl").args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
