pub fn normalize_persian_char(c: char) -> char {
    match c {
        '\u{0643}' => '\u{06A9}', // Arabic Kaf -> Persian Keheh (ك -> ک)
        '\u{064A}' | '\u{0649}' => '\u{06CC}', // Arabic Yeh / Alef Maksura -> Persian Yeh (ي / ى -> ی)
        '\u{0629}' => '\u{0647}', // Teh Marbuta -> Heh (ة -> ه)
        '\u{0622}' | '\u{0623}' | '\u{0625}' => '\u{0627}', // Alef Madda / Hamza -> Alef (آ / أ / إ -> ا)
        _ => c,
    }
}

pub fn normalize_persian_str(s: &str) -> String {
    s.chars().map(normalize_persian_char).collect()
}

pub fn is_persian_letter(c: char) -> bool {
    let c = normalize_persian_char(c);
    matches!(c,
        '\u{0622}' | '\u{0627}' | '\u{0628}' | '\u{067E}' | '\u{062A}' | '\u{062B}' |
        '\u{062C}' | '\u{0686}' | '\u{062D}' | '\u{062E}' |
        '\u{062F}' | '\u{0630}' | '\u{0631}' | '\u{0632}' | '\u{0698}' |
        '\u{0633}' | '\u{0634}' | '\u{0635}' | '\u{0636}' |
        '\u{0637}' | '\u{0638}' | '\u{0639}' | '\u{063A}' |
        '\u{0641}' | '\u{0642}' | '\u{06A9}' | '\u{06AF}' |
        '\u{0644}' | '\u{0645}' | '\u{0646}' |
        '\u{0648}' | '\u{0647}' | '\u{06CC}' | '\u{0621}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        assert_eq!(normalize_persian_str("كتاب"), "کتاب");
        assert_eq!(normalize_persian_str("سیب"), "سیب");
        assert_eq!(normalize_persian_str("علي"), "علی");
        assert_eq!(normalize_persian_str("آبشار"), "ابشار");
        assert!(is_persian_letter('ك'));
        assert!(is_persian_letter('ک'));
        assert!(is_persian_letter('ي'));
        assert!(is_persian_letter('ی'));
    }
}

