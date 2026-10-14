use super::*;

/// Counts how many amounts fit in a budget of 45538.0, in order.
pub fn legacy_probe_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45538.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a legacy probe target code in its canonical LPT- form.
pub fn normalize_legacy_probe_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPT-") {
        return clean.clone();
    }
    format!("LPT-{}", clean.replace(" ", "-"))
}

pub fn legacy_probe_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
