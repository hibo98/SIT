use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::NaiveDateTime;

pub fn format_date_time(datetime: NaiveDateTime) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_option_big_decimal(bd: &Option<BigDecimal>, f: fn(f64, u8) -> String) -> String {
    bd.as_ref()
        .map(|bd| format_big_decimal(bd, f))
        .unwrap_or_default()
}

pub fn format_big_decimal(bd: &BigDecimal, f: fn(f64, u8) -> String) -> String {
    bd.to_f64().map(|bd| f(bd, 0)).unwrap_or_default()
}

pub fn format_bd_percentage(part: &BigDecimal, total: &BigDecimal) -> String {
    if let (Some(part), Some(total)) = (part.to_f64(), total.to_f64()) {
        format!("{:.1} %", part / total * 100_f64)
    } else {
        String::new()
    }
}

pub fn format_filesize_byte(size: f64, exp: u8) -> String {
    if size >= 1000_f64 {
        format_filesize_byte(size / 1000_f64, exp + 3)
    } else {
        format!("{:.1} {}B", size, get_prefix(exp)).replacen('.', ",", 1)
    }
}

pub fn format_filesize_byte_iec(size: f64, exp: u8) -> String {
    if size >= 1024_f64 {
        format_filesize_byte_iec(size / 1024_f64, exp + 3)
    } else {
        format!(
            "{:.1} {}{}B",
            size,
            get_prefix(exp),
            if exp == 0 { "" } else { "i" }
        )
        .replacen('.', ",", 1)
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
