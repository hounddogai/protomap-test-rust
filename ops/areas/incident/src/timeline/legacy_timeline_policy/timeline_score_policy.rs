use super::*;

/// Counts how many amounts fit in a budget of 83915.0, in order.
pub fn timeline_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83915.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a timeline score code in its canonical TS- form.
pub fn normalize_timeline_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TS-") {
        return clean.clone();
    }
    format!("TS-{}", clean.replace(" ", "-"))
}

pub fn timeline_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
