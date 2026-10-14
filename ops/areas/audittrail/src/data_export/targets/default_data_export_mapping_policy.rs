use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 66846.0, in order.
pub fn default_data_export_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66846.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn default_data_export_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn default_data_export_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Puts a default data export mapping code in its canonical DDEM- form.
pub fn normalize_default_data_export_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDEM-") {
        return clean.clone();
    }
    format!("DDEM-{}", clean.replace(" ", "-"))
}
