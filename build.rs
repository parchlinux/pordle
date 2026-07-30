use std::fs;
use std::path::Path;

fn main() {
    let words_path = Path::new("data/words.txt");
    println!("cargo:rerun-if-changed=data/words.txt");

    if !words_path.exists() {
        panic!("data/words.txt does not exist!");
    }

    let content = fs::read_to_string(words_path)
        .expect("Failed to read data/words.txt during build");

    let count = content.lines().filter(|l| !l.trim().is_empty()).count();
    if count == 0 {
        panic!("data/words.txt is empty!");
    }

    println!("cargo:warning=Building Pordle with {} embedded Persian words", count);
}
