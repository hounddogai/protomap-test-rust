use super::*;

use acme_kit::numbers;

pub fn scheduled_redaction_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn scheduled_redaction_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a scheduled redaction assignment code in its canonical SRA- form.
pub fn normalize_scheduled_redaction_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRA-") {
        return clean.clone();
    }
    format!("SRA-{}", clean.replace(" ", "-"))
}
