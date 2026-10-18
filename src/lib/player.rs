use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;

/// Разделитель полей в выводе `playerctl --format` (ASCII Unit Separator),
/// чтобы название трека с любыми символами не ломало разбор.
const SEP: &str = "\u{1f}";

/// URL обложек, которые мы уже пытались скачать в этом запуске (повторно не качаем).
static ATTEMPTED: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();

#[derive(Clone)]
pub struct PlayerLib {}
struct TechZone {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

pub struct PlayerState {
    pub status: PlaybackStatus,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// `mpris:artUrl`: `file://...` или `http(s)://...`
    pub art_url: Option<String>,
    /// Текущая позиция в секундах.
    pub position: f64,
    /// Длина трека в секундах; 0.0, если плеер её не сообщает (стримы и т.п.).
    pub length: f64,
}

impl PlaybackStatus {
    fn parse(s: &str) -> Self {
        match s.trim() {
            "Playing" => Self::Playing,
            "Paused" => Self::Paused,
            _ => Self::Stopped,
        }
    }

    /// Иконка для кнопки play/pause в формате freedesktop.
    /// Показывает действие, которое случится по нажатию.
    pub fn icon_name(self) -> &'static str {
        match self {
            Self::Playing => "media-playback-pause-symbolic",
            Self::Paused | Self::Stopped => "media-playback-start-symbolic",
        }
    }
}

impl PlayerState {
    /// Строка вида "Исполнитель — Название" (для подсказки), если есть хоть что-то из этого.
    pub fn description(&self) -> Option<String> {
        match (self.artist.is_empty(), self.title.is_empty()) {
            (true, true) => None,
            (false, true) => Some(self.artist.clone()),
            (true, false) => Some(self.title.clone()),
            (false, false) => Some(format!("{} — {}", self.artist, self.title)),
        }
    }

    /// Можно ли показывать ползунок перемотки (длина трека известна).
    pub fn is_seekable(&self) -> bool {
        self.length > 0.0
    }
}

impl PlayerLib {
    pub fn play_pause() {
        TechZone::playerctl(&["play-pause"]);
    }

    pub fn play() {
        TechZone::playerctl(&["play"]);
    }

    pub fn pause() {
        TechZone::playerctl(&["pause"]);
    }

    pub fn stop() {
        TechZone::playerctl(&["stop"]);
    }

    pub fn next() {
        TechZone::playerctl(&["next"]);
    }

    pub fn previous() {
        TechZone::playerctl(&["previous"]);
    }

    /// Перематывает на абсолютную позицию (в секундах).
    /// Передаём целое число: дробная часть не нужна ползунку, а `playerctl`
    /// разбирает дроби по локали (при ru_RU «30.5» превратилось бы в 30).
    pub fn set_position(secs: f64) {
        let whole = secs.max(0.0).round() as u64;
        TechZone::playerctl(&["position", &whole.to_string()]);
    }

    /// "3:07" или "1:02:03" — для подписей у ползунка.
    pub fn format_time(secs: f64) -> String {
        let total = secs.max(0.0) as u64;
        let (h, m, s) = (total / 3600, total % 3600 / 60, total % 60);
        if h > 0 {
            format!("{h}:{m:02}:{s:02}")
        } else {
            format!("{m}:{s:02}")
        }
    }

    pub fn get_status() -> PlaybackStatus {
        TechZone::playerctl(&["status"])
            .map(|s| PlaybackStatus::parse(&s))
            .unwrap_or(PlaybackStatus::Stopped)
    }

    /// Возвращает название иконки в формате freedesktop для кнопки play/pause.
    pub fn get_icon_name() -> &'static str {
        Self::get_status().icon_name()
    }

    /// Состояние активного плеера одним вызовом playerctl.
    /// `None`, если плееров нет (или playerctl не установлен).
    pub fn get_state() -> Option<PlayerState> {
        let format = [
            "{{status}}",
            "{{title}}",
            "{{artist}}",
            "{{album}}",
            "{{mpris:artUrl}}",
            // обе величины playerctl отдаёт в микросекундах
            "{{position}}",
            "{{mpris:length}}",
        ]
        .join(SEP);

        let out = TechZone::playerctl(&["metadata", "--format", &format])?;
        let mut parts = out.split(SEP).map(str::trim);

        let status = PlaybackStatus::parse(parts.next()?);
        let title = parts.next().unwrap_or("").to_string();
        let artist = parts.next().unwrap_or("").to_string();
        let album = parts.next().unwrap_or("").to_string();
        let art_url = parts
            .next()
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        // пустая строка (у стримов нет длины) -> 0
        let mut micros_to_secs = || {
            parts
                .next()
                .and_then(|s| s.parse::<i64>().ok())
                .map_or(0.0, |us| us.max(0) as f64 / 1_000_000.0)
        };
        let position = micros_to_secs();
        let length = micros_to_secs();

        Some(PlayerState { status, title, artist, album, art_url, position, length })
    }

    /// Превращает `mpris:artUrl` в локальный путь к картинке.
    /// - `file://` — просто путь (с раскодированием %XX);
    /// - `http(s)://` — файл из кэша; если его ещё нет, в фоне запускается
    ///   скачивание и пока возвращается `None` (картинка появится на одном из следующих вызовов).
    pub fn resolve_cover(url: &str) -> Option<PathBuf> {
        if let Some(rest) = url.strip_prefix("file://") {
            let path = PathBuf::from(TechZone::percent_decode(rest));
            return path.is_file().then_some(path);
        }
        if url.starts_with("http://") || url.starts_with("https://") {
            return TechZone::cached_cover(url);
        }
        None
    }
}

impl TechZone {
    // технические функции, нужные для выполнения других

    fn playerctl(args: &[&str]) -> Option<String> {
        let out = Command::new("playerctl").args(args).output().ok()?;
        // "No players found" и подобное — ненулевой код возврата
        if !out.status.success() {
            return None;
        }
        Some(
            String::from_utf8_lossy(&out.stdout)
                .trim_end_matches('\n')
                .to_string(),
        )
    }

    fn cache_dir() -> PathBuf {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
            .unwrap_or_else(env::temp_dir)
            .join("barricade-sound")
            .join("covers")
    }

    fn cached_cover(url: &str) -> Option<PathBuf> {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let dir = Self::cache_dir();
        let path = dir.join(format!("{:016x}", hasher.finish()));

        if path.is_file() {
            return Some(path);
        }

        // каждый URL пробуем скачать один раз за запуск, чтобы не плодить curl'ы
        let attempted = ATTEMPTED.get_or_init(|| Mutex::new(HashSet::new()));
        if !attempted.lock().ok()?.insert(path.clone()) {
            return None;
        }

        let url = url.to_owned();
        thread::spawn(move || {
            if fs::create_dir_all(&dir).is_err() {
                return;
            }
            let tmp = path.with_extension("part");
            let ok = Command::new("curl")
                .args(["-sfL", "--max-time", "10", "-o"])
                .arg(&tmp)
                .arg(&url)
                .status()
                .is_ok_and(|s| s.success());
            if ok {
                let _ = fs::rename(&tmp, &path);
            } else {
                let _ = fs::remove_file(&tmp);
            }
        });
        None
    }

    fn percent_decode(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }
}
