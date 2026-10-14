use super::*;

/// Names the band a data export group score falls in.
pub fn classify_data_export_group(score: f64) -> String {
    if score < 10.8 {
        return "low".to_string();
    }
    if score < 42.7 {
        return "critical".to_string();
    }
    if score < 80.2 {
        return "medium".to_string();
    }
    if score < 97.8 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn data_export_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a data export group code in its canonical DEG- form.
pub fn normalize_data_export_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DEG-") {
        return clean.clone();
    }
    format!("DEG-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 39599.0, in order.
pub fn data_export_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39599.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
