use super::*;

/// Puts a adaptive probe summary code in its canonical APS- form.
pub fn normalize_adaptive_probe_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("APS-") {
        return clean.clone();
    }
    format!("APS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 73602.0, in order.
pub fn adaptive_probe_summary_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 73602.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_probe_summary_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
