pub const INDENTATION_STRING: &str = "  ";

pub fn indent_string(text: &str) -> String {
    let mut result = String::new();
    let parts: Vec<&str> = text.split("\n").collect();
    for (i, part) in parts.iter().enumerate() {
        result.push_str(INDENTATION_STRING);
        result.push_str(part);
        let is_last = i == parts.len() - 1;
        if !is_last {
            result.push('\n');
        }
    }
    return result;
}

pub fn format_ratio(a: i32, b: i32) -> String {
    if b != 0 {
        let ratio = (a as f32) / (b as f32);
        let integer_ratio = (100.0 * ratio) as i32;
        let mut text = integer_ratio.to_string();
        text.push('%');
        return text;
    } else {
        return String::from("?");
    }
}

pub fn format_percent(a: f32) -> String {
    let percent = (a * 100.0) as i32;
    let mut text = String::from(percent.to_string());
    text.push('%');
    return text;
}