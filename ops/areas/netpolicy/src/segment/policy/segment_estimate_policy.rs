use super::*;

/// Counts how many amounts fit in a budget of 33218.0, in order.
pub fn segment_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33218.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a segment estimate code in its canonical SE- form.
pub fn normalize_segment_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SE-") {
        return clean.clone();
    }
    format!("SE-{}", clean.replace(" ", "-"))
}

pub fn segment_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
