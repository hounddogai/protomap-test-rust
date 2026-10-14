use super::*;

/// Puts a adaptive alert cycle code in its canonical AAC- form.
pub fn normalize_adaptive_alert_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AAC-") {
        return clean.clone();
    }
    format!("AAC-{}", clean.replace(" ", "-"))
}

pub fn adaptive_alert_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 74699.0, in order.
pub fn adaptive_alert_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74699.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
