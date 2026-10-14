use super::*;

/// Counts how many amounts fit in a budget of 98766.0, in order.
pub fn seasonal_freeze_window_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98766.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a seasonal freeze window code in its canonical SFW- form.
pub fn normalize_seasonal_freeze_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFW-") {
        return clean.clone();
    }
    format!("SFW-{}", clean.replace(" ", "-"))
}

pub fn seasonal_freeze_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
