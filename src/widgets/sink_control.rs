use crate::lib::audio::{AudioSinkLib as AudioLib, Sink};
use crate::lib::widget::WidgetTrait;
use anyhow::Result;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

struct SinkRow {
    button: gtk::Button,
    status: gtk::Label,
    name: gtk::Label,
}

/// Виджеты верхней строки (мут, ползунок, стрелка раскрытия)
struct LineWidgets {
    container: gtk::Box,
    indicator: gtk::Image,
    switcher: gtk::ToggleButton,
    volume_scale: gtk::Scale,
}

#[derive(Clone)]
pub struct SinkControl {
    pub main_container: gtk::Box,
    indicator: gtk::Image,
    volume_scale: gtk::Scale,
    switcher: gtk::ToggleButton,
    list: gtk::Box,
    rows: Rc<RefCell<HashMap<i32, SinkRow>>>,
}

impl WidgetTrait for SinkControl {
    fn new() -> Result<Self> {
        let line = Self::build_line();

        let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        list.style_context().add_class("barricade_popup_list_background");
        let revealer = Self::build_revealer(&line.switcher, &list);

        let main_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_container.pack_start(&line.container, false, false, 0);
        main_container.pack_start(&revealer, false, false, 0);

        let this = Self {
            main_container,
            indicator: line.indicator,
            volume_scale: line.volume_scale,
            switcher: line.switcher,
            list,
            rows: Rc::new(RefCell::new(HashMap::new())),
        };

        this.sync_rows(AudioLib::get_devices());
        Ok(this)
    }

    fn update(&mut self) -> Result<()> {
        self.indicator
            .set_icon_name(Some(AudioLib::get_icon_name()));

        // система -> ползунок (обратное направление: connect_value_changed)
        let vol = AudioLib::get_volume() as f64;
        if (self.volume_scale.value() - vol).abs() >= 1.0 {
            self.volume_scale.set_value(vol);
        }

        self.sync_rows(AudioLib::get_devices());
        Ok(())
    }
}

impl SinkControl {
    // ---------- список устройств ----------

    /// Собирает одну строку списка (кнопка + статус + имя) и вешает обработчик клика.
    fn build_row(sink: &Sink, switcher: &gtk::ToggleButton) -> SinkRow {
        let status = gtk::Label::new(Some(Self::status_text(sink)));
        status.set_width_chars(1);

        let name = gtk::Label::new(Some(&sink.name));
        name.set_xalign(0.0);

        let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        content.pack_start(&status, false, false, 0);
        content.pack_start(&name, true, true, 0);

        let button = gtk::Button::new();
        button.add(&content);

        let id = sink.id;
        let switcher = switcher.clone();
        button.connect_clicked(move |_| {
            let _ = AudioLib::set_device(id);
            switcher.set_active(false);
        });

        SinkRow { button, status, name }
    }

    /// Обновляет только изменившиеся поля существующей строки.
    fn update_row(row: &SinkRow, sink: &Sink) {
        let status_text = Self::status_text(sink);
        if row.status.text().as_str() != status_text {
            row.status.set_text(status_text);
        }
        if row.name.text().as_str() != sink.name.as_str() {
            row.name.set_text(&sink.name);
        }
    }

    fn status_text(sink: &Sink) -> &'static str {
        if sink.isdefault { "*" } else { " " }
    }

    /// Приводит список в соответствие с актуальным набором устройств.
    fn sync_rows(&self, sinks: Vec<Sink>) {
        let mut rows = self.rows.borrow_mut();
        let incoming: HashSet<i32> = sinks.iter().map(|s| s.id).collect();

        // 1. удаляем устройства, которых больше нет
        rows.retain(|id, row| {
            let keep = incoming.contains(id);
            if !keep {
                self.list.remove(&row.button);
            }
            keep
        });

        // 2. обновляем существующие / добавляем новые и расставляем порядок
        for (pos, sink) in sinks.iter().enumerate() {
            match rows.get(&sink.id) {
                Some(row) => {
                    Self::update_row(row, sink);
                    self.list.reorder_child(&row.button, pos as i32);
                }
                None => {
                    let row = Self::build_row(sink, &self.switcher);
                    self.list.pack_start(&row.button, false, false, 0);
                    self.list.reorder_child(&row.button, pos as i32);
                    row.button.show_all();
                    rows.insert(sink.id, row);
                }
            }
        }
    }

    // ---------- сборка интерфейса ----------

    fn build_revealer(switcher: &gtk::ToggleButton, list: &gtk::Box) -> gtk::Revealer {
        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(200);
        revealer.add(list);

        // active кнопки -> reveal-child ревелера
        switcher
            .bind_property("active", &revealer, "reveal-child")
            .flags(gtk::glib::BindingFlags::SYNC_CREATE)
            .build();

        revealer
    }

    fn build_line() -> LineWidgets {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // кнопка мута + индикатор состояния
        let mute_button = gtk::Button::new();
        mute_button.style_context().add_class("barricade_mute_butt");
        mute_button.set_can_focus(false);
        let indicator = gtk::Image::builder()
            .icon_name(AudioLib::get_icon_name())
            .build();
        mute_button.add(&indicator);

        let ind = indicator.clone();
        mute_button.connect_clicked(move |_| {
            AudioLib::switch_mute();
            ind.set_icon_name(Some(AudioLib::get_icon_name()));
        });

        // ползунок громкости
        let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_scale.style_context().add_class("barricade_audio_scale");
        volume_scale.set_hexpand(true);
        volume_scale.set_digits(0);
        volume_scale.set_draw_value(false);
        volume_scale.set_can_focus(false);
        volume_scale.set_value(AudioLib::get_volume() as f64);
        // подключаем ПОСЛЕ начальной установки значения
        volume_scale.connect_value_changed(|s| {
            AudioLib::set_volume(s.value().clamp(0.0, 100.0).round() as u32);
        });

        // кнопка со стрелкой раскрытия списка
        let switcher = gtk::ToggleButton::new();
        switcher.style_context().add_class("barricade_list_popup_butt");
        switcher.set_can_focus(false);
        let arrow = gtk::Image::from_icon_name(Some("pan-end-symbolic"), gtk::IconSize::Button);
        switcher.add(&arrow);
        switcher.connect_toggled(move |b| {
            arrow.set_from_icon_name(
                Some(if b.is_active() { "pan-down-symbolic" } else { "pan-end-symbolic" }),
                gtk::IconSize::Button,
            );
        });

        container.pack_start(&mute_button, false, false, 0);
        container.pack_start(&volume_scale, true, true, 0);
        container.pack_start(&switcher, false, false, 0);

        LineWidgets { container, indicator, switcher, volume_scale }
    }
}
