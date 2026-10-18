use gtk::prelude::*;
use gtk::{CssProvider, StyleContext,Application, ApplicationWindow, glib, gdk};
use ui::Ui;

mod lib;
mod ui;
mod widgets;

fn main() -> Result<(), anyhow::Error> {
    let app = Application::builder()
        .application_id("ru.bst.Barricade.sound")
        .build();
    app.connect_startup(|_| load_css());
    app.connect_activate(move |app| {
        create_window(app);
    });

    app.run();
    Ok(())
}

fn create_window(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .build();

    gtk_layer_shell::init_for_window(&win);
    gtk_layer_shell::set_layer(&win, gtk_layer_shell::Layer::Overlay);
    gtk_layer_shell::set_anchor(&win, gtk_layer_shell::Edge::Top, true);
    gtk_layer_shell::set_anchor(&win, gtk_layer_shell::Edge::Right, true);
    gtk_layer_shell::set_margin(&win, gtk_layer_shell::Edge::Top, 10);
    gtk_layer_shell::set_margin(&win, gtk_layer_shell::Edge::Right, 10);
    gtk_layer_shell::set_keyboard_interactivity(&win, true);

    let _ui = Ui::new(&win, app);

    // Esc -> закрыть
    win.connect_key_press_event(|win, event| {
        if event.keyval() == gdk::keys::constants::Escape {
            win.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    // Потеря фокуса (клик снаружи) -> закрыть
    win.connect_focus_out_event(|win, _| {
        win.close();
        glib::Propagation::Proceed
    });

    win.show_all();
}
fn load_css() {
    let provider = CssProvider::new();
    provider
        .load_from_data(include_bytes!("style.css"))
        .expect("Не удалось загрузить src/style.css");

    StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Нет gdk::Screen"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
