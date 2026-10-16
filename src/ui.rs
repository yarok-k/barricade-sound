use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::widgets::sink_control::{SinkControl};
use crate::widgets::mic_cintrol::{MicControl};

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
        let mut sink_controller = SinkControl::new()?;
        sink_controller.main_container.set_halign(gtk::Align::Fill);
        sink_controller.main_container.set_hexpand(true);
        main_container.pack_start(&sink_controller.main_container, false, false,0);

        let mut mic_controller = MicControl::new()?;
        mic_controller.main_container.set_halign(gtk::Align::Fill);
        mic_controller.main_container.set_hexpand(true);
        main_container.pack_start(&mic_controller.main_container, false, false,0);


        let sink_controller_rc = Rc::new(RefCell::new(sink_controller));
        let mic_controller_rc = Rc::new(RefCell::new(mic_controller));

        gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            sink_controller_rc.borrow_mut().update();
            mic_controller_rc.borrow_mut().update();
            gtk::glib::ControlFlow::Continue
        });
        Ok(Self {})
    }
}
