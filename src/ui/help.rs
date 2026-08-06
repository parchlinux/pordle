use gtk::prelude::*;
use gtk::{Box, Justification, Orientation};

use crate::game::state::LetterState;
use crate::ui::tile::Tile;

pub fn build_welcome_page<F: Fn() + 'static, G: Fn() + 'static>(
    on_play: F,
    on_help: G,
) -> adw::StatusPage {
    let play = gtk::Button::with_label("بازی");
    play.add_css_class("pill");
    play.add_css_class("suggested-action");
    play.set_halign(gtk::Align::Center);
    play.connect_clicked(move |_| on_play());

    let help = gtk::Button::with_label("راهنما");
    help.add_css_class("pill");
    help.add_css_class("flat");
    help.set_halign(gtk::Align::Center);
    help.connect_clicked(move |_| on_help());

    let buttons = Box::new(Orientation::Vertical, 8);
    buttons.set_valign(gtk::Align::Center);
    buttons.append(&play);
    buttons.append(&help);

    let page = adw::StatusPage::builder()
        .icon_name("com.parchlinux.pordle")
        .title("پردل")
        .description("کلمه پنج حرفی را در شش تلاش حدس بزنید")
        .build();
    page.set_child(Some(&buttons));
    page
}

pub fn build_help_page() -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);

    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .tightening_threshold(400)
        .build();
    clamp.add_css_class("container");
    scrolled.set_child(Some(&clamp));
    let main_box = Box::new(Orientation::Vertical, 24);
    main_box.set_valign(gtk::Align::Center);
    clamp.set_child(Some(&main_box));

    let title = gtk::Label::new(Some("راهنما"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Center);
    main_box.append(&title);

    let body = gtk::Label::new(Some(
        "حدس بزنید کلمه پنهان چیست! شما <b>شش</b> فرصت دارید تا یک کلمه پنج حرفی را حدس بزنید. \
         بعد از هر حدس، رنگ خانه‌ها نشان می‌دهد چقدر به کلمه درست نزدیک شده‌اید:",
    ));
    body.set_wrap(true);
    body.set_use_markup(true);
    body.set_justify(Justification::Center);
    body.set_halign(gtk::Align::Center);
    main_box.append(&body);

    main_box.append(&state_preview(
        "درخت",
        2,
        LetterState::Correct,
        "«<b>خ</b>» در کلمه است و در جای درست قرار دارد.",
    ));
    main_box.append(&state_preview(
        "میزان",
        1,
        LetterState::Misplaced,
        "«<b>ز</b>» در کلمه است اما جای آن درست نیست.",
    ));
    main_box.append(&state_preview(
        "سفارت",
        3,
        LetterState::Absent,
        "«<b>ر</b>» در کلمه نیست.",
    ));

    let modes = gtk::Label::new(Some(
        "در حالت <b>روزانه</b> همه کاربران یک کلمه یکسان بر اساس تاریخ دریافت می‌کنند و نتیجه ذخیره می‌شود. \
         در حالت <b>آزاد</b> می‌توانید با کلمات تصادفی تمرین کنید.",
    ));
    modes.set_wrap(true);
    modes.set_use_markup(true);
    modes.set_justify(Justification::Center);
    modes.set_halign(gtk::Align::Center);
    main_box.append(&modes);

    let input = gtk::Label::new(Some(
        "می‌توانید با صفحه‌کلید فیزیکی یا صفحه‌کلید لمسی داخل برنامه تایپ کنید.",
    ));
    input.set_wrap(true);
    input.set_justify(Justification::Center);
    input.set_halign(gtk::Align::Center);
    main_box.append(&input);

    scrolled
}

fn state_preview(word: &str, highlight: usize, state: LetterState, desc: &str) -> Box {
    let box_ = Box::new(Orientation::Vertical, 12);

    let cells = Box::new(Orientation::Horizontal, 6);
    cells.set_halign(gtk::Align::Center);
    for (i, c) in word.chars().enumerate() {
        let tile = Tile::new();
        tile.set_letter(c);
        if i == highlight {
            tile.set_state(state);
        }
        cells.append(tile.widget());
    }
    box_.append(&cells);

    let label = gtk::Label::new(Some(desc));
    label.set_wrap(true);
    label.set_use_markup(true);
    label.set_justify(Justification::Center);
    label.set_halign(gtk::Align::Center);
    box_.append(&label);

    box_
}
