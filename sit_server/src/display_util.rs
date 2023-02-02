use chrono::NaiveDateTime;

pub fn format_date_time(datetime: NaiveDateTime) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_filesize_byte(size: f64, exp: u8) -> String {
    if size >= 1000_f64 {
        format_filesize_byte(size / 1000_f64, exp + 3)
    } else {
        format!("{:.1} {}B", size, get_prefix(exp)).replacen('.', ",", 1)
    }
}

fn get_prefix(exp: u8) -> String {
    match exp {
        0 => "",
        3 => "k",
        6 => "M",
        9 => "G",
        12 => "T",
        15 => "E",
        _ => "",
    }
    .to_string()
}
