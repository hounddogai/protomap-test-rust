use super::*;

pub fn manual_data_export_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a manual data export rule code in its canonical MDER- form.
pub fn normalize_manual_data_export_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MDER-") {
        return clean.clone();
    }
    format!("MDER-{}", clean.replace(" ", "-"))
}
