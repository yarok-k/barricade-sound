use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

// Содержимое конфига, создаваемое при первом запуске
const DEFAULT_CONFIG: &str =
    "elements:\n  left:\n  center:\n    - \"clock\"\n  right:\nstylesheet: \"\"\n";

// Позиции элементов
#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ElementPos {
    Left,
    Center,
    Right,
}

// Типы элементов для парсинга YAML.
// Hash нужен, потому что тип используется как ключ HashMap в ElementEntry.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ElementType {
    Clock,
    Audio,
    Layout,
    Network,
    Battery,
}

// Элемент списка в YAML. Поддерживаются оба формата:
//   - "clock"
//   - audio: "/path/to/command"
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ElementEntry {
    Plain(ElementType),
    WithCommand(HashMap<ElementType, Option<String>>),
}

impl ElementEntry {
    // Приводим оба варианта к списку пар (тип, команда)
    fn into_specs(self) -> Vec<ElementSpec> {
        match self {
            ElementEntry::Plain(t) => vec![ElementSpec {
                kind: t,
                command: None,
            }],
            ElementEntry::WithCommand(map) => map
                .into_iter()
                .map(|(kind, cmd)| ElementSpec {
                    kind,
                    command: normalize(cmd),
                })
                .collect(),
        }
    }
}

// Структуры описания YAML-файла
#[derive(Debug, Deserialize)]
struct RawConfig {
    elements: RawElements,
    // Путь к CSS-файлу. Поле необязательное
    #[serde(default)]
    stylesheet: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawElements {
    #[serde(flatten)]
    positions: HashMap<ElementPos, Option<Vec<ElementEntry>>>,
}

// ----- Результат парсинга, который получает Placer -----

// Один элемент бара: тип и необязательная команда
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementSpec {
    pub kind: ElementType,
    pub command: Option<String>,
}

#[derive(Debug)]
pub struct ParsedConfig {
    pub elements: HashMap<ElementPos, Vec<ElementSpec>>,
    pub stylesheet: Option<PathBuf>,  // Абсолютный путь к CSS-файлу, если он задан
}

// "None", "" и null означают "значения нет"
fn normalize(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none"))
}

// Раскрывает "~/", относительные пути считает от каталога конфига
fn resolve_path(raw: &str, config_dir: &Path) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        return Path::new(&home).join(rest);
    }
    let path = Path::new(raw);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    }
}

pub struct Parser {
    config_dir: PathBuf,
    file_path: PathBuf,
}

impl Parser {
    // Стандартное расположение: ~/.config/barricade/bar.bst
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = Path::new(&home).join(".config/barricade");
        let file_path = config_dir.join("bar.bst");
        Self { config_dir, file_path }
    }

    // Читает файл (создавая его при отсутствии) и разбирает в ParsedConfig
    pub fn parse(&self) -> Result<ParsedConfig> {
        let raw = self.read()?;
        self.parse_str(&raw)
    }

    pub fn parse_str(&self, raw: &str) -> Result<ParsedConfig> {
        let config: RawConfig = serde_yaml::from_str(raw)
            .with_context(|| format!("failed to parse {}", self.file_path.display()))?;

        let mut elements = HashMap::new();
        for (pos, entries) in config.elements.positions {
            if let Some(entries) = entries {
                let specs = entries
                    .into_iter()
                    .flat_map(ElementEntry::into_specs)
                    .collect();
                elements.insert(pos, specs);
            }
        }

        let stylesheet = normalize(config.stylesheet)
            .map(|s| resolve_path(&s, &self.config_dir));

        Ok(ParsedConfig { elements, stylesheet })
    }

    fn read(&self) -> Result<String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .with_context(|| format!("cannot create {}", self.config_dir.display()))?;
        }

        if !self.file_path.exists() {
            fs::write(&self.file_path, DEFAULT_CONFIG)
                .with_context(|| format!("cannot write {}", self.file_path.display()))?;
        }

        fs::read_to_string(&self.file_path)
            .with_context(|| format!("cannot read {}", self.file_path.display()))
    }
}
