use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use gtk::prelude::*;
use gtk::{CssProvider, StyleContext, Application, ApplicationWindow, glib, gdk};
use ui::Ui;
use lib::parcer::{modified_time, Parser};

mod lib;
mod ui;
mod widgets;

// Следит за конфигом и CSS-файлом и перезагружает стили на лету.
// Провайдер создаётся один раз и только перезаполняется.
struct StyleWatcher {
    provider: CssProvider,
    css_path: Option<PathBuf>,
    config_mtime: Option<SystemTime>,
    css_mtime: Option<SystemTime>,
}

impl StyleWatcher {
    fn new() -> Self {
        let provider = CssProvider::new();
        match gdk::Screen::default() {
            Some(screen) => StyleContext::add_provider_for_screen(
                &screen,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            ),
            None => eprintln!("stylesheet disabled: no default screen"),
        }

        let mut watcher = Self {
            provider,
            css_path: None,
            config_mtime: None,
            css_mtime: None,
        };
        watcher.reload_config();
        watcher
    }

    // Перечитывает конфиг (путь к CSS мог поменяться) и заново грузит стили
    fn reload_config(&mut self) {
        let parser = Parser::new();
        // mtime берём до парсинга, чтобы ошибка в конфиге не повторялась каждый тик
        self.config_mtime = modified_time(parser.config_path());

        match parser.parse() {
            Ok(cfg) => self.css_path = cfg.stylesheet,
            Err(e) => eprintln!("config reload failed: {e:#}"),
        }
        self.reload_css();
    }

    fn reload_css(&mut self) {
        self.css_mtime = self.css_path.as_deref().and_then(modified_time);

        let result = match &self.css_path {
            Some(path) => self.provider.load_from_path(&path.to_string_lossy()),
            // стиль убрали из конфига — очищаем провайдер
            None => self.provider.load_from_data(b""),
        };
        if let Err(e) = result {
            eprintln!("stylesheet load failed: {e}");
        }
    }

    fn tick(&mut self) {
        let parser = Parser::new();
        if modified_time(parser.config_path()) != self.config_mtime {
            self.reload_config();
        } else if self.css_path.as_deref().and_then(modified_time) != self.css_mtime {
            self.reload_css();
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let app = Application::builder()
        .application_id("ru.bst.Barricade.sound")
        .build();

    app.connect_startup(|_| {
        let watcher = Rc::new(RefCell::new(StyleWatcher::new()));
        glib::timeout_add_local(Duration::from_millis(500), move || {
            watcher.borrow_mut().tick();
            glib::ControlFlow::Continue
        });
    });

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

    // Прозрачный визуал, чтобы скруглённые углы не были непрозрачными
    win.set_app_paintable(true);
    if let Some(visual) = gdk::Screen::default().and_then(|s| s.rgba_visual()) {
        win.set_visual(Some(&visual));
    }

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
