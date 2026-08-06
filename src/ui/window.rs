use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::{ApplicationWindow, Toast};
use gtk::gdk;
use gtk::{Box, Orientation, ToggleButton};

use crate::db::DatabaseManager;
use crate::game::state::{Game, Phase, WORD_LENGTH};
use crate::persian::is_persian_letter;
use crate::ui::board::Board;
use crate::ui::help;
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
    _size_provider: gtk::CssProvider,
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

const BASE_TILE: f64 = 52.0;
const BASE_KEY_W: f64 = 34.0;
const BASE_KEY_H: f64 = 46.0;
const BOARD_COLS: f64 = 5.0;
const BOARD_ROWS: f64 = 6.0;
const GRID_SPACING: f64 = 4.0;
const GRID_MARGIN: f64 = 8.0;
const KBD_KEYS_PER_ROW: f64 = 9.0;
const KBD_ROWS: f64 = 4.0;
const KBD_ROW_SPACING: f64 = 4.0;
const KBD_COL_SPACING: f64 = 1.0;
const KBD_VERTICAL_MARGIN: f64 = 10.0;
const TILE_MARGIN: f64 = 2.0;
const KEY_MARGIN: f64 = 1.0;
const HEADER_H: f64 = 52.0;
const SAFE_MARGIN: f64 = 24.0;

fn board_width(tile: f64) -> f64 {
    BOARD_COLS * (tile + 2.0 * TILE_MARGIN) + (BOARD_COLS - 1.0) * GRID_SPACING + 2.0 * GRID_MARGIN
}

fn board_height(tile: f64) -> f64 {
    BOARD_ROWS * (tile + 2.0 * TILE_MARGIN) + (BOARD_ROWS - 1.0) * GRID_SPACING + 2.0 * GRID_MARGIN
}

fn keyboard_width(key_w: f64) -> f64 {
    KBD_KEYS_PER_ROW * (key_w + 2.0 * KEY_MARGIN) + (KBD_KEYS_PER_ROW - 1.0) * KBD_COL_SPACING + 4.0
}

fn keyboard_height(key_h: f64) -> f64 {
    KBD_ROWS * (key_h + 2.0 * KEY_MARGIN) + (KBD_ROWS - 1.0) * KBD_ROW_SPACING + KBD_VERTICAL_MARGIN
}

fn content_width(tile: f64, key_w: f64) -> f64 {
    board_width(tile).max(keyboard_width(key_w))
}

fn content_height(tile: f64, key_h: f64) -> f64 {
    HEADER_H + board_height(tile) + keyboard_height(key_h)
}

fn fit_scale(screen_w: i32, screen_h: i32) -> f64 {
    let base_w = content_width(BASE_TILE, BASE_KEY_W);
    let base_h = content_height(BASE_TILE, BASE_KEY_H);
    let s_w = (screen_w as f64 - SAFE_MARGIN) / base_w;
    let s_h = (screen_h as f64 - SAFE_MARGIN) / base_h;
    s_w.min(s_h).clamp(0.5, 1.0)
}

fn layout_scale(screen_w: i32, screen_h: i32) -> f64 {
    let mut scale = fit_scale(screen_w, screen_h);
    let tile = (BASE_TILE * scale).round().max(30.0);
    let key_w = (BASE_KEY_W * scale).round().max(24.0);
    let key_h = (BASE_KEY_H * scale).round().max(34.0);
    let need_w = content_width(tile, key_w);
    let need_h = content_height(tile, key_h);
    let s = ((screen_w as f64 - SAFE_MARGIN) / need_w)
        .min((screen_h as f64 - SAFE_MARGIN) / need_h);
    if s < scale {
        scale = s.clamp(0.5, scale);
    }
    scale
}

fn layout_sizes(screen_w: i32, screen_h: i32) -> (f64, f64, f64, i32, i32) {
    let scale = layout_scale(screen_w, screen_h);
    let tile = (BASE_TILE * scale).round().max(30.0);
    let key_w = (BASE_KEY_W * scale).round().max(24.0);
    let key_h = (BASE_KEY_H * scale).round().max(34.0);
    let win_w = content_width(tile, key_w).ceil() as i32;
    let win_h = content_height(tile, key_h).ceil() as i32;
    (scale, tile, key_w, win_w, win_h)
}

fn primary_monitor_size() -> Option<(i32, i32)> {
    let display = gtk::gdk::Display::default()?;
    let monitors = display.monitors();
    let monitor = monitors.item(0)?.downcast::<gdk::Monitor>().ok()?;
    let geo = monitor.geometry();
    Some((geo.width(), geo.height()))
}

fn build_size_css(scale: f64) -> String {
    let tile = (BASE_TILE * scale).round().max(30.0);
    let tile_font = (28.0 * scale).round().max(18.0);
    let key_w = (BASE_KEY_W * scale).round().max(24.0);
    let key_h = (BASE_KEY_H * scale).round().max(34.0);
    let key_font = (15.0 * scale).round().max(12.0);
    let special_w = (44.0 * scale).round().max(30.0);
    let special_font = (12.0 * scale).round().max(10.0);
    let mode_font = (13.0 * scale).round().max(11.0);
    format!(
        ".tile {{ min-width: {tile}px; min-height: {tile}px; font-size: {tile_font}px; }}\n\
         .keyboard-key {{ min-width: {key_w}px; min-height: {key_h}px; font-size: {key_font}px; }}\n\
         .special-key {{ min-width: {special_w}px; font-size: {special_font}px; }}\n\
         .mode-btn {{ font-size: {mode_font}px; }}"
    )
}

fn apply_size_css(provider: &gtk::CssProvider, scale: f64) {
    provider.load_from_data(&build_size_css(scale));
}

fn refit_window(provider: &gtk::CssProvider, window: &ApplicationWindow) {
    if let Some((screen_w, screen_h)) = primary_monitor_size() {
        let (scale, _tile, _key_w, win_w, win_h) = layout_sizes(screen_w, screen_h);
        apply_size_css(provider, scale);
        window.set_default_size(win_w, win_h);
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
    glib::DateTime::now_local()
        .and_then(|dt| dt.format("%Y-%m-%d"))
        .map(|s| s.to_string())
        .unwrap_or_else(|_| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            (secs / 86400).to_string()
        })
}

fn generate_emoji_share(game: &Game) -> String {
    let mut share = String::from("پردل\n");
    for row in 0..game.current_row {
        for col in 0..WORD_LENGTH {
            match game.results[row][col] {
                Some(crate::game::state::LetterState::Correct) => share.push('🟩'),
                Some(crate::game::state::LetterState::Misplaced) => share.push('🟨'),
                Some(crate::game::state::LetterState::Absent) => share.push('⬛'),
                _ => {}
            }
        }
        share.push('\n');
    }
    share
}

fn show_stats_dialog(
    parent: &ApplicationWindow,
    db: &DatabaseManager,
    game: &Game,
    toast_overlay: &adw::ToastOverlay,
) {
    let stats = db.get_stats();
    let played = stats.games_played;
    let won = stats.games_won;
    let streak = stats.current_streak;
    let max_streak = stats.max_streak;
    let pct = if played > 0 {
        (won as f64 / played as f64) * 100.0
    } else {
        0.0
    };

    let dialog = gtk::Window::builder()
        .title("آمار بازی")
        .transient_for(parent)
        .modal(true)
        .default_width(320)
        .default_height(420)
        .resizable(false)
        .build();

    let root = Box::new(Orientation::Vertical, 16);
    root.set_margin_top(16);
    root.set_margin_bottom(16);
    root.set_margin_start(16);
    root.set_margin_end(16);

    let title_label = gtk::Label::builder()
        .label("آمار کل")
        .css_classes(["title-2"])
        .build();
    root.append(&title_label);

    let stats_grid = gtk::Grid::builder()
        .column_spacing(12)
        .row_spacing(6)
        .halign(gtk::Align::Center)
        .build();

    let create_stat_box = |value: String, label: &str| {
        let b = Box::new(Orientation::Vertical, 2);
        b.set_halign(gtk::Align::Center);
        let val_lbl = gtk::Label::builder()
            .label(&value)
            .css_classes(["stat-number"])
            .build();
        let name_lbl = gtk::Label::builder()
            .label(label)
            .css_classes(["stat-label"])
            .build();
        b.append(&val_lbl);
        b.append(&name_lbl);
        b
    };

    stats_grid.attach(&create_stat_box(played.to_string(), "بازی‌ها"), 0, 0, 1, 1);
    stats_grid.attach(&create_stat_box(format!("{:.0}%", pct), "برد"), 1, 0, 1, 1);
    stats_grid.attach(&create_stat_box(streak.to_string(), "رکورد"), 2, 0, 1, 1);
    stats_grid.attach(&create_stat_box(max_streak.to_string(), "بیشترین"), 3, 0, 1, 1);

    root.append(&stats_grid);

    let dist_title = gtk::Label::builder()
        .label("توزیع حدس‌ها")
        .css_classes(["title-3"])
        .halign(gtk::Align::Start)
        .build();
    root.append(&dist_title);

    let dist_box = Box::new(Orientation::Vertical, 4);
    let max_dist = *stats.guess_distribution.iter().max().unwrap_or(&1).max(&1);

    for (i, &count) in stats.guess_distribution.iter().enumerate() {
        let row = Box::new(Orientation::Horizontal, 8);
        let num_lbl = gtk::Label::new(Some(&(i + 1).to_string()));
        num_lbl.set_width_chars(2);
        row.append(&num_lbl);

        let bar_width = if max_dist > 0 {
            ((count as f64 / max_dist as f64) * 100.0).max(8.0) as i32
        } else {
            8
        };

        let bar = gtk::Label::builder()
            .label(&count.to_string())
            .css_classes(["dist-bar"])
            .halign(gtk::Align::Start)
            .build();
        bar.set_size_request(bar_width * 2, -1);
        row.append(&bar);

        dist_box.append(&row);
    }
    root.append(&dist_box);

    let btn_box = Box::new(Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::Center);

    let share_btn = gtk::Button::with_label("اشتراک‌گذاری 📋");
    share_btn.set_css_classes(&["suggested-action"]);
    let emoji_text = generate_emoji_share(game);
    let share_toast = toast_overlay.clone();
    let win_dialog = dialog.clone();
    share_btn.connect_clicked(move |_| {
        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&emoji_text);
        }
        share_toast.add_toast(Toast::new("نتیجه در حافظه کپی شد!"));
        win_dialog.close();
    });
    btn_box.append(&share_btn);

    let close_btn = gtk::Button::with_label("بستن");
    let dlg = dialog.clone();
    close_btn.connect_clicked(move |_| {
        dlg.close();
    });
    btn_box.append(&close_btn);

    root.append(&btn_box);

    dialog.set_child(Some(&root));
    dialog.present();
}

impl GameWindow {
    pub fn new(app: &adw::Application) -> Self {
        load_css();

        let size_provider = gtk::CssProvider::new();
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &size_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let (screen_w, screen_h) = primary_monitor_size().unwrap_or((1280, 720));
        let (scale, _tile, _key_w, win_w, win_h) = layout_sizes(screen_w, screen_h);
        apply_size_css(&size_provider, scale);

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
                let daily_answer = pick_word(&db_ref, &GameMode::Daily);
                if let Some(result) = db_ref.load_daily_result(&today) {
                    if result.answer == daily_answer {
                        let g = Game::restore_with_guesses(result.answer, &result.guesses);
                        (g, true)
                    } else {
                        (Game::new(daily_answer), false)
                    }
                } else {
                    (Game::new(daily_answer), false)
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

        let back_button = gtk::Button::from_icon_name("go-previous-symbolic");
        back_button.set_tooltip_text(Some("بازگشت"));
        back_button.set_visible(false);
        header_bar.pack_start(&back_button);

        let app_title = gtk::Label::new(Some("پردل"));
        app_title.add_css_class("title");
        header_bar.set_title_widget(Some(&app_title));

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

        let new_game_button = gtk::Button::from_icon_name("view-refresh-symbolic");
        new_game_button.set_tooltip_text(Some("بازی جدید"));
        new_game_button.set_visible(false);
        header_bar.pack_end(&new_game_button);

        let hamburger = gtk::MenuButton::new();
        hamburger.set_icon_name("open-menu-symbolic");
        hamburger.set_tooltip_text(Some("منو"));

        let popover = gtk::Popover::new();
        popover.set_autohide(true);

        let popover_box = Box::new(Orientation::Vertical, 4);
        popover_box.set_margin_top(6);
        popover_box.set_margin_bottom(6);
        popover_box.set_margin_start(6);
        popover_box.set_margin_end(6);

        let help_btn = gtk::Button::with_label("راهنما");
        help_btn.set_css_classes(&["flat"]);
        help_btn.set_halign(gtk::Align::Fill);

        let stats_btn = gtk::Button::with_label("آمار بازی");
        stats_btn.set_css_classes(&["flat"]);
        stats_btn.set_halign(gtk::Align::Fill);

        let hc_btn = ToggleButton::with_label("کنتراست بالا");
        hc_btn.set_css_classes(&["flat"]);
        hc_btn.set_halign(gtk::Align::Fill);

        let about_btn = gtk::Button::with_label("درباره پردل");
        about_btn.set_css_classes(&["flat"]);
        about_btn.set_halign(gtk::Align::Fill);

        popover_box.append(&help_btn);
        popover_box.append(&stats_btn);
        popover_box.append(&hc_btn);
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

        let started = Rc::new(Cell::new(false));
        let page_active = Rc::new(Cell::new(false));

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        let welcome_page = help::build_welcome_page(
            {
                let stack = stack.clone();
                let started = started.clone();
                move || {
                    started.set(true);
                    stack.set_visible_child_name("game");
                }
            },
            {
                let stack = stack.clone();
                move || {
                    stack.set_visible_child_name("help");
                }
            },
        );
        let help_page = help::build_help_page();

        stack.add_named(&main_box, Some("game"));
        stack.add_named(&welcome_page, Some("welcome"));
        stack.add_named(&help_page, Some("help"));
        stack.set_visible_child_name("welcome");

        toast_overlay.set_child(Some(&stack));

        let content = Box::new(Orientation::Vertical, 0);
        content.append(&header_bar);
        content.append(&toast_overlay);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("پردل")
            .default_width(win_w)
            .default_height(win_h)
            .content(&content)
            .build();

        window.set_icon_name(Some("com.parchlinux.pordle"));
        window.present();

        if let Some(display) = gdk::Display::default() {
            let monitors = display.monitors();
            for i in 0..monitors.n_items() {
                if let Some(monitor) = monitors.item(i).and_then(|o| o.downcast::<gdk::Monitor>().ok()) {
                    let win_ref = window.clone();
                    let provider_ref = size_provider.clone();
                    monitor.connect_invalidate(move |_| {
                        refit_window(&provider_ref, &win_ref);
                    });
                }
            }
        }

        let back_ref = back_button.clone();
        let new_game_ref = new_game_button.clone();
        let mode_box_ref = mode_box.clone();
        let app_title_ref = app_title.clone();
        let header_ref = header_bar.clone();
        let page_active_ref = page_active.clone();
        stack.connect_visible_child_name_notify(move |stack| {
            let name = stack
                .visible_child_name()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let on_game = name == "game";
            page_active_ref.set(on_game);
            back_ref.set_visible(name == "help");
            new_game_ref.set_visible(on_game);
            if on_game {
                header_ref.set_title_widget(Some(&mode_box_ref));
            } else {
                header_ref.set_title_widget(Some(&app_title_ref));
            }
        });

        let stack_back = stack.clone();
        let started_back = started.clone();
        back_button.connect_clicked(move |_| {
            if started_back.get() {
                stack_back.set_visible_child_name("game");
            } else {
                stack_back.set_visible_child_name("welcome");
            }
        });

        let stack_help = stack.clone();
        help_btn.connect_clicked(move |_| {
            stack_help.set_visible_child_name("help");
        });

        let win_ref = window.clone();
        hc_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                win_ref.add_css_class("high-contrast");
            } else {
                win_ref.remove_css_class("high-contrast");
            }
        });

        let gw = Self {
            window,
            game,
            board,
            keyboard,
            db,
            mode,
            toast_overlay,
            _size_provider: size_provider,
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
        gw.setup_keyboard_input(page_active, back_button.clone());
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
                let daily_answer = pick_word(&db_ref, &GameMode::Daily);
                let (g, _restored) =
                    if let Some(result) = db_ref.load_daily_result(&daily_today) {
                        if result.answer == daily_answer {
                            (Game::restore_with_guesses(result.answer, &result.guesses), true)
                        } else {
                            (Game::new(daily_answer), false)
                        }
                    } else {
                        (Game::new(daily_answer), false)
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

        new_game_button.connect_clicked(move |_| {
            let m = *new_mode.borrow();
            let db_ref = new_db.borrow();
            let g = if m == GameMode::Daily {
                Game::new(pick_word(&db_ref, &GameMode::Daily))
            } else {
                Game::new(pick_word(&db_ref, &GameMode::Practice))
            };
            drop(db_ref);
            *new_game.borrow_mut() = g;
            new_board.reset();
            new_keyboard.reset_states();
        });

        about_btn.connect_clicked(|btn| {
            let about = adw::AboutDialog::builder()
                .application_name("پردل")
                .application_icon("com.parchlinux.pordle")
                .version("0.2.0")
                .comments("یک بازی وردل فارسی برای پارچ لینوکس")
                .website("https://parchlinux.com")
                .issue_url("https://github.com/parchlinux/pordle")
                .license_type(gtk::License::Gpl30)
                .build();
            about.present(Some(btn));
        });

        let stats_db = self.db.clone();
        let stats_game = self.game.clone();
        let stats_toast = self.toast_overlay.clone();
        let window_parent = self.window.clone();

        stats_btn.connect_clicked(move |_| {
            show_stats_dialog(
                &window_parent,
                &stats_db.borrow(),
                &stats_game.borrow(),
                &stats_toast,
            );
        });
    }

    fn setup_keyboard_input(&self, active: Rc<Cell<bool>>, back_button: gtk::Button) {
        let game = self.game.clone();
        let board = self.board.clone();
        let keyboard = self.keyboard.clone();
        let db = self.db.clone();
        let toast = self.toast_overlay.clone();
        let mode = self.mode.clone();
        let today = today_key();

        self.keyboard.set_callback(move |event| {
            if !active.get() {
                return;
            }
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
        let back_btn = back_button.clone();

        ctrl.connect_key_pressed(move |_ctrl, keyval, _keycode, _state| {
            if keyval == gdk::Key::Escape && back_btn.is_visible() {
                back_btn.activate();
                return glib::Propagation::Stop;
            }

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
