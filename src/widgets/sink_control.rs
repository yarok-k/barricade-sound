use crate::lib::audio::{AudioSinkLib as AudioLib, Sink};
use crate::lib::widget::WidgetTrait;
use anyhow::Result;
use dbus::arg::RefArg;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};

struct SinkRow {
    button: gtk::Button,
    status: gtk::Label,
    name: gtk::Label,
}

#[derive(Clone)]
pub struct SinkControl {
    pub main_container: gtk::Box,
    mute_trigger: gtk::Button,
    volume_switch: gtk::Scale,
    indicator: gtk::Image,
    sink_switcher: gtk::ToggleButton,
    sink_revealer: gtk::Revealer,
    sink_list: gtk::Box,
    sink_rows: Rc<RefCell<HashMap<i32, SinkRow>>>,
}

impl WidgetTrait for SinkControl {
    fn new() -> Result<Self> {
        // верхний ряд (то, что раньше было main_container)
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let mute_trigger = gtk::Button::new();
        let indicator = gtk::Image::builder()
            .icon_name(AudioLib::get_sink_icon_name())
            .build();
        mute_trigger.add(&indicator);

        let ind = indicator.clone();
        mute_trigger.connect_clicked(move |_| {
            AudioLib::switch_mute();
            ind.set_icon_name(Some(AudioLib::get_sink_icon_name()));
        });

        let volume_switch = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_switch.set_hexpand(true);
        volume_switch.set_value(AudioLib::get_volume().as_f64().unwrap_or(0.0));
        volume_switch.set_digits(0);
        volume_switch.set_draw_value(false);

        // кнопка-переключатель со стрелкой
        let arrow = gtk::Image::from_icon_name(Some("pan-end-symbolic"), gtk::IconSize::Button);
        let sink_switcher = gtk::ToggleButton::new();
        sink_switcher.add(&arrow);

        // ревелер со списком
        let sink_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let sink_revealer = gtk::Revealer::new();
        sink_revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        sink_revealer.set_transition_duration(200);
        sink_revealer.add(&sink_list);

        // active кнопки -> reveal-child ревелера
        sink_switcher
            .bind_property("active", &sink_revealer, "reveal-child")
            .flags(gtk::glib::BindingFlags::SYNC_CREATE)
            .build();

        // меняем стрелку при переключении
        sink_switcher.connect_toggled(move |b| {
            arrow.set_from_icon_name(
                Some(if b.is_active() { "pan-down-symbolic" } else { "pan-end-symbolic" }),
                gtk::IconSize::Button,
            );
        });

        row.pack_start(&mute_trigger, false, false, 0);
        row.pack_start(&volume_switch, true, true, 0);
        row.pack_start(&sink_switcher, false, false, 0);

        // главный контейнер теперь вертикальный
        let main_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_container.pack_start(&row, false, false, 0);
        main_container.pack_start(&sink_revealer, false, false, 0);

        let selfer = Self {
            main_container,
            mute_trigger,
            volume_switch,
            indicator,
            sink_switcher,
            sink_revealer,
            sink_list,
            sink_rows: Rc::new(RefCell::new(HashMap::new()))
        };

        let sinks = AudioLib::get_sinks();
        selfer.sync_sinks(sinks);

        Ok(selfer)
    }

    fn update(&mut self) -> Result<()> {
        let sinks = {
            self.indicator.set_icon_name(Some(AudioLib::get_sink_icon_name()));
            let vol = self.volume_switch.value().clamp(0.0, 100.0).round() as u32;
            AudioLib::set_volume(vol);
            AudioLib::get_sinks()
        };
        self.sync_sinks(sinks);
        Ok(())
    }
}

impl SinkControl {
    fn sync_sinks(&self, sinks: Vec<Sink>) {
        let mut rows = self.sink_rows.borrow_mut();
        let incoming: HashSet<i32> = sinks.iter().map(|s| s.id).collect();

        // 1. удаляем устройства, которых больше нет
        rows.retain(|id, row| {
            if incoming.contains(id) {
                true
            } else {
                self.sink_list.remove(&row.button);
                false
            }
        });

        // 2. обновляем существующие и добавляем новые
        for (pos, sink) in sinks.iter().enumerate() {
            let status_text = if sink.isdefault { "*" } else { " " };

            if let Some(row) = rows.get(&sink.id) {
                // обновляем только то, что изменилось
                if row.status.text() != status_text {
                    row.status.set_text(status_text);
                }
                if row.name.text() != sink.name.as_str() {
                    row.name.set_text(&sink.name);
                }
                self.sink_list.reorder_child(&row.button, pos as i32);
            } else {
                let status = gtk::Label::new(Some(status_text));
                status.set_width_chars(1);
                let name = gtk::Label::new(Some(&sink.name));
                name.set_xalign(0.0);

                let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                content.pack_start(&status, false, false, 0);
                content.pack_start(&name, true, true, 0);

                let button = gtk::Button::new();
                button.add(&content);

                let id = sink.id;
                let switcher = self.sink_switcher.clone();
                button.connect_clicked(move |_| {
                    AudioLib::set_sink(id);
                    switcher.set_active(false);
                });

                self.sink_list.pack_start(&button, false, false, 0);
                self.sink_list.reorder_child(&button, pos as i32);
                button.show_all();

                rows.insert(sink.id, SinkRow { button, status, name });
            }
        }
    }
}
