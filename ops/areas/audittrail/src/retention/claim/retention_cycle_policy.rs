use super::*;

pub fn retention_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a retention cycle score falls in.
pub fn classify_retention_cycle(score: f64) -> String {
    if score < 50.3 {
        return "elevated".to_string();
    }
    if score < 79.7 {
        return "critical".to_string();
    }
    if score < 87.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Puts a retention cycle code in its canonical RC- form.
pub fn normalize_retention_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 76423.0, in order.
pub fn retention_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 76423.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
