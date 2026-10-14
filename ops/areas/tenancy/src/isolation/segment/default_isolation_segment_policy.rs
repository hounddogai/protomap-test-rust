use super::*;

pub fn default_isolation_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Puts a default isolation segment code in its canonical DIS- form.
pub fn normalize_default_isolation_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DIS-") {
        return clean.clone();
    }
    format!("DIS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 30682.0, in order.
pub fn default_isolation_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30682.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
