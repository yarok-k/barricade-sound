use crate::lib::audio::{AudioMicLib as AudioLib, Mic};
use crate::lib::widget::WidgetTrait;
use anyhow::Result;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

struct MicRow {
    button: gtk::Button,
    status: gtk::Label,
    name: gtk::Label,
}

#[derive(Clone)]
pub struct MicControl {
    pub main_container: gtk::Box,
    mute_trigger: gtk::Button,
    volume_switch: gtk::Scale,
    indicator: gtk::Image,
    mic_switcher: gtk::ToggleButton,
    mic_revealer: gtk::Revealer,
    mic_list: gtk::Box,
    mic_rows: Rc<RefCell<HashMap<i32, MicRow>>>,
}

impl WidgetTrait for MicControl {
    fn new() -> Result<Self> {
        // верхний ряд
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let mute_trigger = gtk::Button::new();
        let indicator = gtk::Image::builder()
            .icon_name(AudioLib::get_icon_name())
            .build();
        mute_trigger.add(&indicator);

        let ind = indicator.clone();
        mute_trigger.connect_clicked(move |_| {
            AudioLib::switch_mute();
            ind.set_icon_name(Some(AudioLib::get_icon_name()));
        });

        let volume_switch = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_switch.set_hexpand(true);
        volume_switch.set_value(AudioLib::get_volume() as f64);
        volume_switch.set_digits(0);
        volume_switch.set_draw_value(false);

        // кнопка-переключатель со стрелкой
        let arrow = gtk::Image::from_icon_name(Some("pan-end-symbolic"), gtk::IconSize::Button);
        let mic_switcher = gtk::ToggleButton::new();
        mic_switcher.add(&arrow);

        // ревелер со списком микрофонов
        let mic_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let mic_revealer = gtk::Revealer::new();
        mic_revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        mic_revealer.set_transition_duration(200);
        mic_revealer.add(&mic_list);

        // active кнопки -> reveal-child ревелера
        mic_switcher
            .bind_property("active", &mic_revealer, "reveal-child")
            .flags(gtk::glib::BindingFlags::SYNC_CREATE)
            .build();

        // меняем стрелку при переключении
        mic_switcher.connect_toggled(move |b| {
            arrow.set_from_icon_name(
                Some(if b.is_active() { "pan-down-symbolic" } else { "pan-end-symbolic" }),
                gtk::IconSize::Button,
            );
        });

        row.pack_start(&mute_trigger, false, false, 0);
        row.pack_start(&volume_switch, true, true, 0);
        row.pack_start(&mic_switcher, false, false, 0);

        // главный контейнер вертикальный
        let main_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_container.pack_start(&row, false, false, 0);
        main_container.pack_start(&mic_revealer, false, false, 0);

        let selfer = Self {
            main_container,
            mute_trigger,
            volume_switch,
            indicator,
            mic_switcher,
            mic_revealer,
            mic_list,
            mic_rows: Rc::new(RefCell::new(HashMap::new())),
        };

        selfer.sync_mics(AudioLib::get_devices());

        Ok(selfer)
    }

    fn update(&mut self) -> Result<()> {
        self.indicator.set_icon_name(Some(AudioLib::get_icon_name()));
        let vol = self.volume_switch.value().clamp(0.0, 100.0).round() as u32;
        AudioLib::set_gain(vol);
        self.sync_mics(AudioLib::get_devices());
        Ok(())
    }
}

impl MicControl {
    fn sync_mics(&self, mics: Vec<Mic>) {
        let mut rows = self.mic_rows.borrow_mut();
        let incoming: HashSet<i32> = mics.iter().map(|m| m.id).collect();

        // 1. удаляем устройства, которых больше нет
        rows.retain(|id, row| {
            if incoming.contains(id) {
                true
            } else {
                self.mic_list.remove(&row.button);
                false
            }
        });

        // 2. обновляем существующие и добавляем новые
        for (pos, mic) in mics.iter().enumerate() {
            let status_text = if mic.isdefault { "*" } else { " " };

            if let Some(row) = rows.get(&mic.id) {
                // обновляем только то, что изменилось
                if row.status.text() != status_text {
                    row.status.set_text(status_text);
                }
                if row.name.text() != mic.name.as_str() {
                    row.name.set_text(&mic.name);
                }
                self.mic_list.reorder_child(&row.button, pos as i32);
            } else {
                let status = gtk::Label::new(Some(status_text));
                status.set_width_chars(1);
                let name = gtk::Label::new(Some(&mic.name));
                name.set_xalign(0.0);

                let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                content.pack_start(&status, false, false, 0);
                content.pack_start(&name, true, true, 0);

                let button = gtk::Button::new();
                button.add(&content);

                let id = mic.id;
                let switcher = self.mic_switcher.clone();
                button.connect_clicked(move |_| {
                    let _ = AudioLib::set_device(id);
                    switcher.set_active(false);
                });

                self.mic_list.pack_start(&button, false, false, 0);
                self.mic_list.reorder_child(&button, pos as i32);
                button.show_all();

                rows.insert(mic.id, MicRow { button, status, name });
            }
        }
    }
}
