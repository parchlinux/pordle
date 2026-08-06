mod db;
mod game;
mod persian;
mod ui;

use adw::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "پردل",
    version = "0.2.0",
    about = "A Persian Wordle game for Parch Linux"
)]
struct Cli {
    #[arg(long, value_name = "WORD")]
    add_word: Option<String>,

    #[arg(long, value_name = "FILE")]
    import: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match (cli.add_word, cli.import) {
        (Some(word), None) => {
            let mut db = db::DatabaseManager::open().expect("Failed to open database");
            match db.add_word(&word).expect("Failed to add word") {
                db::AddWordResult::Added => println!("Added word: {}", word),
                db::AddWordResult::Duplicate => eprintln!("Word already exists: {}", word),
                db::AddWordResult::Invalid => {
                    eprintln!("Invalid word (must be 5 Persian letters): {}", word)
                }
            }
        }
        (None, Some(file)) => {
            let mut db = db::DatabaseManager::open().expect("Failed to open database");
            match db.import_from_file(&file) {
                Ok((total, added)) => {
                    println!("Imported {}/{} words from {}", added, total, file);
                }
                Err(e) => {
                    eprintln!("Failed to import from {}: {}", file, e);
                }
            }
        }
        (None, None) => {
            run_gui();
        }
        (Some(_), Some(_)) => {
            eprintln!("Use either --add-word OR --import, not both");
            std::process::exit(1);
        }
    }
}

fn run_gui() {
    std::env::set_var("LANG", "fa_IR.UTF-8");
    std::env::set_var("LANGUAGE", "fa:en");

    adw::init().expect("Failed to initialize libadwaita");

    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);

    gtk::Widget::set_default_direction(gtk::TextDirection::Rtl);

    let app = adw::Application::builder()
        .application_id("com.parchlinux.pordle")
        .build();

    app.connect_activate(|app| {
        ui::window::GameWindow::new(app);
    });

    app.run();
}
