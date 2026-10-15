use crate::lib::audio::{AudioLib,Sink};
use crate::lib::widget::WidgetTrait;
use anyhow::Result;
use dbus::arg::RefArg;
use gtk::{false_, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct AudioSwitch {
    pub main_container: gtk::Box,
    mute_trigger: gtk::Button,
    volume_switch: gtk::Scale,
    indicator: gtk::Image,
    sink_switcher: gtk::MenuButton,
    sink_list: gtk::Box,
    audio_lib: Rc<RefCell<AudioLib>>,
}

impl WidgetTrait for AudioSwitch {
    fn new() -> Result<Self> {
        let audio_lib = Rc::new(RefCell::new(AudioLib::new()));
        let internal_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let mute_trigger = gtk::Button::new();
        let indicator = gtk::Image::builder()
            .icon_name(audio_lib.borrow_mut().get_icon_name())
            .build();
        mute_trigger.add(&indicator);

        // клоны для замыкания: Rc и gtk::Image дёшево клонируются
        // и указывают на тот же самый объект
        let al = audio_lib.clone();
        let ind = indicator.clone();
        mute_trigger.connect_clicked(move |_| {
            let mut lib = al.borrow_mut();
            lib.switch_mute();
            ind.set_icon_name(Some(lib.get_icon_name()));
        });

        let volume_switch = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        volume_switch.set_hexpand(true);
        volume_switch.set_value(audio_lib.borrow_mut().get_volume().as_f64().unwrap_or(0.0));
        volume_switch.set_digits(0);
        volume_switch.set_draw_value(false);

        let sink_switcher = gtk::MenuButton::new();
        let sink_list_container = gtk::Popover::new(Some(&sink_switcher));
        let sink_list = gtk::Box::new(gtk::Orientation::Vertical,0);
        sink_list_container.add(&sink_list);
        sink_switcher.set_popover(Some(&sink_list_container));

        sink_list.show_all();

        for u in &audio_lib.borrow_mut().get_sinks(){
            println!("{} | {}",u.name, u.id)
        }
        internal_container.pack_start(&mute_trigger, false, false, 0);
        internal_container.pack_start(&volume_switch, true, true, 0);
        internal_container.pack_start(&sink_switcher, false, false, 0);


        let selfer = Self {
            main_container: internal_container,
            mute_trigger,
            volume_switch,
            indicator,
            sink_switcher,
            sink_list,
            audio_lib,
        };

        selfer.fill_sinks(selfer.audio_lib.borrow_mut().get_sinks());


        Ok(selfer)
    }

    fn update(&mut self) -> Result<()> {
        let mut lib = self.audio_lib.borrow_mut();
        lib.update();
        self.indicator.set_icon_name(Some(lib.get_icon_name()));
        let vol = self.volume_switch.value().clamp(0.0, 100.0).round() as u32;
        lib.set_volume(vol);
        Ok(())
    }
}
impl AudioSwitch {
    fn fill_sinks(&self ,sinks: Vec<Sink>){
        for sink in sinks {
            println!("{}",sink.name);
            let sink_inf = gtk::Label::builder().label(sink.name).build();
            let sink_inf_trigger = gtk::Button::new();
            sink_inf_trigger.add(&sink_inf);
            sink_inf_trigger.connect_clicked(move |_| {

            });
            self.sink_list.pack_start(&sink_inf_trigger, false, false, 0);
        }
        self.sink_list.show_all();
    }
}
