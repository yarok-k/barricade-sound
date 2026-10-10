use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Ui {}

impl Ui {
    pub fn new(
        window: &gtk::ApplicationWindow,
        app: &gtk::Application,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // тестовый размер окна 200x200
        window.set_size_request(200, 200);
        window.set_default_size(200, 200);

        let main_container = gtk::Grid::new();
        main_container.set_column_homogeneous(true);
        main_container.set_hexpand(true);
        window.add(&main_container);

        gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            gtk::glib::ControlFlow::Continue
        });
        Ok(Self {})
    }
}
