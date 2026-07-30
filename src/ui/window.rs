use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{ApplicationWindow, Toast};
use gtk::{Box, Orientation, ToggleButton};

use crate::db::DatabaseManager;
use crate::game::state::{Game, Phase, WORD_LENGTH};
use crate::persian::is_persian_letter;
use crate::ui::board::Board;
use crate::ui::keyboard::{KeyEvent, Keyboard};

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    Daily,
    Practice,
}

pub struct GameWindow {
    pub window: ApplicationWindow,
    game: Rc<RefCell<Game>>,
    board: Rc<Board>,
    keyboard: Rc<Keyboard>,
    db: Rc<RefCell<DatabaseManager>>,
    mode: Rc<RefCell<GameMode>>,
    toast_overlay: adw::ToastOverlay,
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    let css = include_str!("../../data/pordle.css");
    provider.load_from_data(&css);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn pick_word(db: &DatabaseManager, mode: &GameMode) -> String {
    match mode {
        GameMode::Daily => db.daily_word(),
        GameMode::Practice => db.random_word(),
    }
    .unwrap_or_else(|| {
        if db.word_count() == 0 {
            "?????".to_string()
        } else {
            db.random_word().unwrap_or_else(|| "?????".to_string())
        }
    })
}

fn today_key() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    (secs / 86400).to_string()
}

impl GameWindow {
    pub fn new(app: &adw::Application) -> Self {
        load_css();

        let db = Rc::new(RefCell::new(DatabaseManager::open_or_default()));
        {
            let mut db_mut = db.borrow_mut();
            db_mut.populate_from_default_files();
        }

        let mode = Rc::new(RefCell::new(GameMode::Daily));
        let today = today_key();

        let (game, restored) = {
            let db_ref = db.borrow();
            if *mode.borrow() == GameMode::Daily {
                if let Some(result) = db_ref.load_daily_result(&today) {
                    let g = Game::restore_with_guesses(result.answer, &result.guesses);
                    (g, true)
                } else {
                    let answer = pick_word(&db_ref, &GameMode::Daily);
                    (Game::new(answer), false)
                }
            } else {
                let answer = pick_word(&db_ref, &GameMode::Practice);
                (Game::new(answer), false)
            }
        };

        let game = Rc::new(RefCell::new(game));
        let board = Rc::new(Board::new());
        let keyboard = Rc::new(Keyboard::new());

        let toast_overlay = adw::ToastOverlay::new();

        let header_bar = adw::HeaderBar::new();

        let mode_box = Box::new(Orientation::Horizontal, 0);
        mode_box.set_css_classes(&["mode-switch"]);
        mode_box.set_halign(gtk::Align::Center);

        let daily_btn = ToggleButton::with_label("روزانه");
        daily_btn.set_css_classes(&["mode-btn"]);
        daily_btn.set_active(true);

        let practice_btn = ToggleButton::with_label("آزاد");
        practice_btn.set_css_classes(&["mode-btn"]);

        mode_box.append(&daily_btn);
        mode_box.append(&practice_btn);
        header_bar.set_title_widget(Some(&mode_box));

        let new_game_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        new_game_button.set_tooltip_text(Some("بازی جدید"));
        header_bar.pack_end(&new_game_button);

        let hamburger = gtk::MenuButton::new();
        hamburger.set_icon_name("open-menu-symbolic");
        hamburger.set_tooltip_text(Some("منو"));

        let popover = gtk::Popover::new();
        popover.set_autohide(true);

        let popover_box = Box::new(Orientation::Vertical, 0);
        popover_box.set_margin_top(6);
        popover_box.set_margin_bottom(6);
        popover_box.set_margin_start(6);
        popover_box.set_margin_end(6);

        let stats_btn = gtk::Button::with_label("آمار بازی");
        stats_btn.set_css_classes(&["flat"]);
        stats_btn.set_halign(gtk::Align::Fill);

        let about_btn = gtk::Button::with_label("درباره پردل");
        about_btn.set_css_classes(&["flat"]);
        about_btn.set_halign(gtk::Align::Fill);

        popover_box.append(&stats_btn);
        popover_box.append(&about_btn);
        popover.set_child(Some(&popover_box));
        hamburger.set_popover(Some(&popover));
        header_bar.pack_end(&hamburger);

        let clamp = adw::Clamp::builder()
            .maximum_size(500)
            .tightening_threshold(400)
            .build();

        let content_box = Box::new(Orientation::Vertical, 0);
        content_box.set_vexpand(true);
        content_box.set_valign(gtk::Align::Center);
        content_box.set_halign(gtk::Align::Center);
        content_box.append(board.widget());

        clamp.set_child(Some(&content_box));

        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&clamp);
        main_box.append(keyboard.widget());

        toast_overlay.set_child(Some(&main_box));

        let content = Box::new(Orientation::Vertical, 0);
        content.append(&header_bar);
        content.append(&toast_overlay);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("پردل")
            .default_width(380)
            .default_height(720)
            .content(&content)
            .build();

        window.set_icon_name(Some("com.parchlinux.pordle"));
        window.present();

        let gw = Self {
            window,
            game,
            board,
            keyboard,
            db,
            mode,
            toast_overlay,
        };

        if restored {
            let g = gw.game.borrow();
            gw.board.restore_from_game(&g);
            let states = g.keyboard_states();
            gw.keyboard.update_states(&states);
        }

        gw.connect_signals(
            &daily_btn,
            &practice_btn,
            &new_game_button,
            &about_btn,
            &stats_btn,
        );
        gw.setup_keyboard_input();
        gw
    }

    fn connect_signals(
        &self,
        daily_btn: &ToggleButton,
        practice_btn: &ToggleButton,
        new_game_button: &gtk::Button,
        about_btn: &gtk::Button,
        stats_btn: &gtk::Button,
    ) {
        let daily_game = self.game.clone();
        let daily_board = self.board.clone();
        let daily_keyboard = self.keyboard.clone();
        let daily_db = self.db.clone();
        let daily_mode = self.mode.clone();
        let daily_today = today_key();
        let pbtn = practice_btn.clone();

        daily_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                *daily_mode.borrow_mut() = GameMode::Daily;
                let db_ref = daily_db.borrow();
                let (g, _restored) =
                    if let Some(result) = db_ref.load_daily_result(&daily_today) {
                        (Game::restore_with_guesses(result.answer, &result.guesses), true)
                    } else {
                        (Game::new(pick_word(&db_ref, &GameMode::Daily)), false)
                    };
                drop(db_ref);
                *daily_game.borrow_mut() = g;
                daily_board.reset();
                daily_keyboard.reset_states();
                let g = daily_game.borrow();
                daily_board.restore_from_game(&g);
                let states = g.keyboard_states();
                daily_keyboard.update_states(&states);
                pbtn.set_active(false);
            } else if !pbtn.is_active() {
                btn.set_active(true);
            }
        });

        let practice_game = self.game.clone();
        let practice_board = self.board.clone();
        let practice_keyboard = self.keyboard.clone();
        let practice_db = self.db.clone();
        let practice_mode = self.mode.clone();
        let dbtn = daily_btn.clone();

        practice_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                *practice_mode.borrow_mut() = GameMode::Practice;
                let answer = pick_word(&practice_db.borrow(), &GameMode::Practice);
                *practice_game.borrow_mut() = Game::new(answer);
                practice_board.reset();
                practice_keyboard.reset_states();
                dbtn.set_active(false);
            } else if !dbtn.is_active() {
                btn.set_active(true);
            }
        });

        let new_game = self.game.clone();
        let new_board = self.board.clone();
        let new_keyboard = self.keyboard.clone();
        let new_db = self.db.clone();
        let new_mode = self.mode.clone();
        let new_today = today_key();

        new_game_button.connect_clicked(move |_| {
            let m = *new_mode.borrow();
            let db_ref = new_db.borrow();
            let (g, _restored) = if m == GameMode::Daily {
                if let Some(result) = db_ref.load_daily_result(&new_today) {
                    (Game::restore_with_guesses(result.answer, &result.guesses), true)
                } else {
                    (Game::new(pick_word(&db_ref, &GameMode::Daily)), false)
                }
            } else {
                (Game::new(pick_word(&db_ref, &GameMode::Practice)), false)
            };
            drop(db_ref);
            *new_game.borrow_mut() = g;
            new_board.reset();
            new_keyboard.reset_states();
            let g = new_game.borrow();
            new_board.restore_from_game(&g);
            let states = g.keyboard_states();
            new_keyboard.update_states(&states);
        });

        about_btn.connect_clicked(|btn| {
            let about = adw::AboutDialog::builder()
                .application_name("پردل")
                .version("0.1.0")
                .comments("یک بازی وردل فارسی برای پارچ لینوکس")
                .website("https://parchlinux.com")
                .issue_url("https://github.com/parchlinux/pordle")
                .license_type(gtk::License::Gpl30)
                .build();
            about.present(Some(btn));
        });

        let stats_db = self.db.clone();
        let stats_toast = self.toast_overlay.clone();
        stats_btn.connect_clicked(move |_| {
            let stats = stats_db.borrow().get_stats();
            let played = stats.games_played;
            let won = stats.games_won;
            let streak = stats.current_streak;
            let max_streak = stats.max_streak;
            let pct = if played > 0 {
                (won as f64 / played as f64) * 100.0
            } else {
                0.0
            };
            let msg = format!(
                "بازی: {} | برد: {} ({:.0}%) | رکورد: {} | بیشترین: {}",
                played, won, pct, streak, max_streak
            );
            stats_toast.add_toast(Toast::new(&msg));
        });
    }

    fn setup_keyboard_input(&self) {
        let game = self.game.clone();
        let board = self.board.clone();
        let keyboard = self.keyboard.clone();
        let db = self.db.clone();
        let toast = self.toast_overlay.clone();
        let mode = self.mode.clone();
        let today = today_key();

        self.keyboard.set_callback(move |event| {
            match event {
                KeyEvent::Letter(c) => {
                    let mut g = game.borrow_mut();
                    if g.phase != Phase::Playing {
                        return;
                    }
                    if g.current_col >= WORD_LENGTH {
                        return;
                    }
                    if g.type_letter(c).is_ok() {
                        board.set_letter(g.current_row, g.current_col - 1, c);
                    }
                }
                KeyEvent::Enter => {
                    let mut g = game.borrow_mut();
                    if g.phase != Phase::Playing {
                        return;
                    }
                    if g.current_col < WORD_LENGTH {
                        toast.add_toast(Toast::new("حروف کافی نیست"));
                        return;
                    }

                    let guess = g.current_guess_string();

                    if !db.borrow().is_valid_word(&guess) {
                        toast.add_toast(Toast::new("کلمه معتبر نیست"));
                        return;
                    }

                    let result = match g.submit_guess() {
                        Ok(r) => r,
                        Err(e) => {
                            toast.add_toast(Toast::new(e));
                            return;
                        }
                    };

                    let row = g.current_row - 1;
                    let phase = g.phase;
                    let answer = g.answer_string();
                    let guesses = g.guesses();
                    let is_daily = *mode.borrow() == GameMode::Daily;
                    drop(g);

                    board.reveal_row(row, &result);

                    let g = game.borrow();
                    let states = g.keyboard_states();
                    keyboard.update_states(&states);
                    drop(g);

                    match phase {
                        Phase::Won => {
                            toast.add_toast(Toast::new(&format!("🎉 {} حدس", row + 1)));
                            let _ = db.borrow().record_game(true, row + 1);
                            if is_daily {
                                let _ = db.borrow().save_daily_result(&today, &answer, true, &guesses);
                            }
                        }
                        Phase::Lost => {
                            toast.add_toast(Toast::new(&format!("متأسفم! جواب: {}", answer)));
                            let _ = db.borrow().record_game(false, 0);
                            if is_daily {
                                let _ = db.borrow().save_daily_result(&today, &answer, false, &guesses);
                            }
                        }
                        Phase::Playing => {}
                    }
                }
                KeyEvent::Backspace => {
                    let mut g = game.borrow_mut();
                    if g.phase != Phase::Playing {
                        return;
                    }
                    if g.current_col == 0 {
                        return;
                    }
                    let col = g.current_col - 1;
                    g.delete_letter();
                    board.clear_letter(g.current_row, col);
                }
            }
        });

        let ctrl = gtk::EventControllerKey::new();
        ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
        let keyboard_sender = self.keyboard.clone();

        ctrl.connect_key_pressed(move |_ctrl, keyval, _keycode, _state| {
            if keyval == gdk::Key::Return || keyval == gdk::Key::KP_Enter {
                keyboard_sender.send_event(KeyEvent::Enter);
                return glib::Propagation::Stop;
            }

            if keyval == gdk::Key::BackSpace {
                keyboard_sender.send_event(KeyEvent::Backspace);
                return glib::Propagation::Stop;
            }

            if let Some(c) = keyval.to_unicode() {
                if is_persian_letter(c) {
                    keyboard_sender.send_event(KeyEvent::Letter(c));
                    return glib::Propagation::Stop;
                }
            }

            glib::Propagation::Proceed
        });

        self.window.add_controller(ctrl);
    }
}
