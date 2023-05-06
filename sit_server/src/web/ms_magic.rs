pub fn resolve_profile_health_status(health_status: i16) -> String {
    match health_status {
        0 => "Gesund".to_owned(),
        1 => "Ungesund".to_owned(),
        2 => "Achtung".to_owned(),
        _ => String::new(),
    }
}

pub fn resolve_profile_status(status: i64) -> Vec<String> {
    let mut output: Vec<String> = vec![];
    if status & 0x1 == 0x1 {
        output.push("Temporary".to_owned());
    }
    if status & 0x2 == 0x2 {
        output.push("Roaming".to_owned());
    }
    if status & 0x4 == 0x4 {
        output.push("Mandatory".to_owned());
    }
    if status & 0x8 == 0x8 {
        output.push("Corrupted".to_owned());
    }
    output
}
