use std::process::Command;

use anyhow::{Error, Result};

#[derive(Clone)]
pub struct AudioSinkLib {}
#[derive(Clone)]
pub struct AudioMicLib {}
pub struct TechZone {}

pub struct Sink {
    pub isdefault: bool,
    pub id: i32,
    pub name: String,
}
pub struct Mic {
    pub isdefault: bool,
    pub id: i32,
    pub name: String,
}

const DEFAULT_SINK: &str = "@DEFAULT_AUDIO_SINK@";
const DEFAULT_SOURCE: &str = "@DEFAULT_AUDIO_SOURCE@";

impl AudioSinkLib {
    /// Устанавливает уровень громкости, значение от 0 до 100
    pub fn set_volume(pct: u32) {
        TechZone::wpctl(&["set-volume", DEFAULT_SINK, &format!("{}%", pct)]);
    }

    pub fn get_volume() -> u32 {
        TechZone::get_level(DEFAULT_SINK)
    }

    pub fn set_mute(mute: bool) {
        TechZone::wpctl(&["set-mute", DEFAULT_SINK, if mute { "1" } else { "0" }]);
    }

    pub fn switch_mute() {
        TechZone::wpctl(&["set-mute", DEFAULT_SINK, "toggle"]);
    }

    pub fn get_mute() -> bool {
        TechZone::get_muted(DEFAULT_SINK)
    }

    pub fn get_devices() -> Vec<Sink> {
        TechZone::parse_section("Sinks")
            .into_iter()
            .map(|(isdefault, id, name)| Sink { isdefault, id, name })
            .collect()
    }

    pub fn set_device(sink_id: i32) -> Result<(), Error> {
        TechZone::wpctl(&["set-default", &sink_id.to_string()]);
        Ok(())
    }

    /// Возвращает название иконки в формате freedesktop.
    pub fn get_icon_name() -> &'static str {
        let vol = Self::get_volume();
        if Self::get_mute() || vol == 0 {
            return "audio-volume-muted";
        }
        match vol {
            1..=30 => "audio-volume-low",
            31..=70 => "audio-volume-medium",
            _ => "audio-volume-high",
        }
    }
}

impl AudioMicLib {
    /// Устанавливает уровень усиления микрофона, значение от 0 до 100
    pub fn set_gain(pct: u32) {
        TechZone::wpctl(&["set-volume", DEFAULT_SOURCE, &format!("{}%", pct)]);
    }

    pub fn get_volume() -> u32 {
        TechZone::get_level(DEFAULT_SOURCE)
    }

    pub fn set_mute(mute: bool) {
        TechZone::wpctl(&["set-mute", DEFAULT_SOURCE, if mute { "1" } else { "0" }]);
    }

    pub fn switch_mute() {
        TechZone::wpctl(&["set-mute", DEFAULT_SOURCE, "toggle"]);
    }

    pub fn get_mute() -> bool {
        TechZone::get_muted(DEFAULT_SOURCE)
    }

    pub fn get_devices() -> Vec<Mic> {
        TechZone::parse_section("Sources")
            .into_iter()
            .map(|(isdefault, id, name)| Mic { isdefault, id, name })
            .collect()
    }

    pub fn set_device(mic_id: i32) -> Result<(), Error> {
        TechZone::wpctl(&["set-default", &mic_id.to_string()]);
        Ok(())
    }

    /// Возвращает название иконки в формате freedesktop.
    pub fn get_icon_name() -> &'static str {
        let gain = Self::get_volume();
        if Self::get_mute() || gain == 0 {
            return "microphone-sensitivity-muted";
        }
        match gain {
            1..=30 => "microphone-sensitivity-low",
            31..=70 => "microphone-sensitivity-medium",
            _ => "microphone-sensitivity-high",
        }
    }
}

impl TechZone {
    // технические функции, нужные для выполнения других

    fn wpctl(args: &[&str]) -> Option<String> {
        let out = Command::new("wpctl").args(args).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Уровень громкости/усиления в процентах для указанного узла.
    fn get_level(target: &str) -> u32 {
        TechZone::wpctl(&["get-volume", target])
            .and_then(|s| {
                s.split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse::<f32>().ok())
            })
            .map(|v| (v * 100.0).round() as u32)
            .unwrap_or(0)
    }

    fn get_muted(target: &str) -> bool {
        TechZone::wpctl(&["get-volume", target])
            .map(|s| s.contains("[MUTED]"))
            .unwrap_or(false)
    }

    /// Разбирает вывод `wpctl status` и возвращает устройства из секции
    /// раздела Audio ("Sinks" или "Sources") как (isdefault, id, name).
    fn parse_section(wanted: &str) -> Vec<(bool, i32, String)> {
        let Some(output) = TechZone::wpctl(&["status"]) else {
            return Vec::new();
        };

        let mut in_audio = false;
        let mut in_section = false;
        let mut items = Vec::new();

        for line in output.lines() {
            // Заголовок верхнего уровня (Audio / Video / Settings) — строка без отступа
            if line.chars().next().is_some_and(|c| c.is_alphabetic()) {
                in_audio = line.trim() == "Audio";
                in_section = false;
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
                in_section = section == wanted;
                continue;
            }

            if in_section {
                if let Some(item) = Self::parse_node_line(content) {
                    items.push(item);
                }
            }
        }

        items
    }

    fn parse_node_line(s: &str) -> Option<(bool, i32, String)> {
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

        Some((isdefault, id, name.trim().to_string()))
    }
}
