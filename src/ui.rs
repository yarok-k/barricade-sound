use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::widgets::main_audio_switch::AudioSwitch;
use crate::lib::widget::WidgetTrait;

pub struct Ui {}

impl Ui {
    pub fn new(
        window: &gtk::ApplicationWindow,
        app: &gtk::Application,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // тестовый размер окна 200x200
        window.set_size_request(200, 200);
        window.set_default_size(200, 200);

        let main_container = gtk::Box::new(gtk::Orientation::Vertical,0);
        // main_container.set_hexpand(true);
        window.add(&main_container);
        // виджет для настройки звука системы (применяется к default sink)
        let mut system_audio_switch = AudioSwitch::new()?;
        system_audio_switch.main_container.set_halign(gtk::Align::Fill);
        system_audio_switch.main_container.set_hexpand(true);
        main_container.pack_start(&system_audio_switch.main_container, false, false,0);



        let sas_rc = Rc::new(RefCell::new(system_audio_switch));


        gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            sas_rc.borrow_mut().update();
            gtk::glib::ControlFlow::Continue
        });
        Ok(Self {})
    }
}
