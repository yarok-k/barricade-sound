use crate::lib::player::{PlaybackStatus, PlayerLib, PlayerState};
use crate::lib::widget::WidgetTrait;
use anyhow::Result;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

const COVER_SIZE: i32 = 128;
const COVER_PLACEHOLDER: &str = "audio-x-generic";
const NO_TIME: &str = "--:--";

/// Как часто отправлять перемотку в плеер, пока ползунок тянут.
const SEEK_THROTTLE: Duration = Duration::from_millis(150);
/// Сколько после последнего действия пользователя update() не трогает ползунок,
/// чтобы он не отскакивал назад, пока плеер применяет перемотку.
const SEEK_HOLD: Duration = Duration::from_millis(1500);

/// Состояние перемотки, общее для обработчиков сигналов и update().
#[derive(Default)]
struct SeekState {
    /// кнопка мыши зажата на ползунке
    dragging: Cell<bool>,
    /// до этого момента ползунок принадлежит пользователю
    hold_until: Cell<Option<Instant>>,
    /// последняя позиция, ещё не отправленная в плеер
    pending: Cell<Option<f64>>,
    /// запущен ли таймер отправки
    timer_active: Cell<bool>,
}

impl SeekState {
    fn user_active(&self) -> bool {
        self.dragging.get() || self.hold_until.get().is_some_and(|t| Instant::now() < t)
    }
}

#[derive(Clone)]
pub struct Player {
    pub main_container: gtk::Box,
    start_pause: gtk::Button,
    play_icon: gtk::Image,
    switch_forward: gtk::Button,
    switch_backward: gtk::Button,
    album: gtk::Image,
    seek: gtk::Scale,
    elapsed: gtk::Label,
    total: gtk::Label,
    seek_state: Rc<SeekState>,
    /// какая обложка сейчас показана (чтобы не перечитывать файл на каждом update)
    shown_cover: Rc<RefCell<Option<PathBuf>>>,
}

impl WidgetTrait for Player {
    fn new() -> Result<Self> {
        // верхний ряд
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let switch_backward =
            gtk::Button::from_icon_name(Some("media-skip-backward-symbolic"), gtk::IconSize::Button);
        let switch_forward =
            gtk::Button::from_icon_name(Some("media-skip-forward-symbolic"), gtk::IconSize::Button);

        let play_icon = gtk::Image::from_icon_name(
            Some(PlaybackStatus::Stopped.icon_name()),
            gtk::IconSize::Button,
        );
        let start_pause = gtk::Button::new();
        start_pause.add(&play_icon);

        switch_backward.connect_clicked(|_| PlayerLib::previous());
        switch_forward.connect_clicked(|_| PlayerLib::next());

        let icon = play_icon.clone();
        start_pause.connect_clicked(move |_| {
            let was_playing = PlayerLib::get_status() == PlaybackStatus::Playing;
            PlayerLib::play_pause();
            // плеер меняет состояние не мгновенно — сразу показываем ожидаемое,
            // а update() потом сверит с реальным
            let expected = if was_playing {
                PlaybackStatus::Paused
            } else {
                PlaybackStatus::Playing
            };
            icon.set_from_icon_name(Some(expected.icon_name()), gtk::IconSize::Button);
        });

        row.pack_start(&switch_backward, false, false, 0);
        row.pack_start(&start_pause, true, true, 0);
        row.pack_start(&switch_forward, false, false, 0);

        // ряд перемотки: 1:23 ──●────── 3:45
        let elapsed = gtk::Label::new(Some(NO_TIME));
        let total = gtk::Label::new(Some(NO_TIME));
        elapsed.set_width_chars(5);
        total.set_width_chars(5);

        let seek = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
        seek.set_hexpand(true);
        seek.set_draw_value(false);
        seek.set_increments(5.0, 15.0); // стрелки: ±5 с, PgUp/PgDn: ±15 с
        seek.set_sensitive(false);

        let seek_state = Rc::new(SeekState::default());

        // пока мышь зажата, update() ползунок не трогает
        let state = seek_state.clone();
        seek.connect_button_press_event(move |_, _| {
            state.dragging.set(true);
            glib::Propagation::Proceed
        });
        let state = seek_state.clone();
        seek.connect_button_release_event(move |_, _| {
            state.dragging.set(false);
            state.hold_until.set(Some(Instant::now() + SEEK_HOLD));
            glib::Propagation::Proceed
        });

        // change-value приходит только от пользователя (мышь, клавиши, колесо),
        // программный set_value() в update() его не вызывает
        let state = seek_state.clone();
        let elapsed_label = elapsed.clone();
        seek.connect_change_value(move |scale, _, value| {
            let adj = scale.adjustment();
            let value = value.clamp(adj.lower(), adj.upper());

            elapsed_label.set_text(&PlayerLib::format_time(value));
            state.pending.set(Some(value));
            state.hold_until.set(Some(Instant::now() + SEEK_HOLD));

            // не чаще раза в SEEK_THROTTLE, но последнее значение уйдёт обязательно
            if !state.timer_active.replace(true) {
                let state = state.clone();
                glib::timeout_add_local_once(SEEK_THROTTLE, move || {
                    state.timer_active.set(false);
                    if let Some(pos) = state.pending.take() {
                        PlayerLib::set_position(pos);
                    }
                });
            }
            glib::Propagation::Proceed
        });

        let seek_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        seek_row.pack_start(&elapsed, false, false, 0);
        seek_row.pack_start(&seek, true, true, 0);
        seek_row.pack_start(&total, false, false, 0);

        let album = gtk::Image::builder()
            .icon_name(COVER_PLACEHOLDER)
            .pixel_size(COVER_SIZE)
            .build();

        // главный контейнер вертикальный
        let main_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_container.pack_start(&album, false, false, 0);
        main_container.pack_start(&seek_row, false, false, 0);
        main_container.pack_start(&row, false, false, 0);

        let selfer = Self {
            main_container,
            start_pause,
            play_icon,
            switch_forward,
            switch_backward,
            album,
            seek,
            elapsed,
            total,
            seek_state,
            shown_cover: Rc::new(RefCell::new(None)),
        };

        selfer.refresh();

        Ok(selfer)
    }

    fn update(&mut self) -> Result<()> {
        self.refresh();
        Ok(())
    }
}

impl Player {
    fn refresh(&self) {
        let state = PlayerLib::get_state();
        self.refresh_seek(state.as_ref());

        let Some(state) = state else {
            // плееров нет — гасим кнопки и возвращаем заглушку
            self.set_controls_sensitive(false);
            self.play_icon
                .set_from_icon_name(Some(PlaybackStatus::Stopped.icon_name()), gtk::IconSize::Button);
            self.album.set_tooltip_text(None);
            self.show_cover(None);
            return;
        };

        self.set_controls_sensitive(true);
        self.play_icon
            .set_from_icon_name(Some(state.status.icon_name()), gtk::IconSize::Button);
        self.album.set_tooltip_text(state.description().as_deref());

        let cover = state.art_url.as_deref().and_then(PlayerLib::resolve_cover);
        self.show_cover(cover);
    }

    /// Подтягивает ползунок и подписи времени к состоянию плеера.
    fn refresh_seek(&self, state: Option<&PlayerState>) {
        let seekable = state.is_some_and(PlayerState::is_seekable);
        self.seek.set_sensitive(seekable);

        let position = state.map_or(0.0, |s| s.position);
        let length = state.filter(|_| seekable).map_or(0.0, |s| s.length);

        if !seekable {
            // ползунок погас — отпускание кнопки могло не дойти
            self.seek_state.dragging.set(false);
        }

        // диапазон и общая длительность могут меняться вместе с треком
        let upper = if seekable { length } else { 1.0 };
        if self.seek.adjustment().upper() != upper {
            self.seek.set_range(0.0, upper);
        }
        self.total.set_text(&if seekable {
            PlayerLib::format_time(length)
        } else {
            NO_TIME.to_string()
        });

        // пока пользователь тянет/только что отпустил — его значение важнее
        if self.seek_state.user_active() {
            return;
        }
        self.seek.set_value(if seekable { position.min(length) } else { 0.0 });
        self.elapsed.set_text(&if state.is_some() {
            PlayerLib::format_time(position)
        } else {
            NO_TIME.to_string()
        });
    }

    fn set_controls_sensitive(&self, sensitive: bool) {
        self.switch_backward.set_sensitive(sensitive);
        self.start_pause.set_sensitive(sensitive);
        self.switch_forward.set_sensitive(sensitive);
    }

    fn show_cover(&self, cover: Option<PathBuf>) {
        let mut shown = self.shown_cover.borrow_mut();
        if *shown == cover {
            return;
        }

        match cover
            .as_deref()
            .map(|p| Pixbuf::from_file_at_scale(p, COVER_SIZE, COVER_SIZE, true))
        {
            Some(Ok(pixbuf)) => self.album.set_from_pixbuf(Some(&pixbuf)),
            // нет обложки или файл не удалось прочитать — заглушка
            _ => self.album.set_icon_name(Some(COVER_PLACEHOLDER)),
        }
        *shown = cover;
    }
}
