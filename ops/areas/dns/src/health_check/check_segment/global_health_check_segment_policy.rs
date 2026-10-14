use super::*;

/// Counts how many amounts fit in a budget of 2930.0, in order.
pub fn global_health_check_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2930.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a global health check segment code in its canonical GHCS- form.
pub fn normalize_global_health_check_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GHCS-") {
        return clean.clone();
    }
    format!("GHCS-{}", clean.replace(" ", "-"))
}

/// Names the band a global health check segment score falls in.
pub fn classify_global_health_check_segment(score: f64) -> String {
    if score < 27.7 {
        return "elevated".to_string();
    }
    if score < 65.5 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn global_health_check_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
