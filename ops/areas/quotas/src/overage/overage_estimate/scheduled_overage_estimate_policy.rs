use super::*;

/// Counts how many amounts fit in a budget of 66749.0, in order.
pub fn scheduled_overage_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 66749.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn scheduled_overage_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Puts a scheduled overage estimate code in its canonical SOE- form.
pub fn normalize_scheduled_overage_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SOE-") {
        return clean.clone();
    }
    format!("SOE-{}", clean.replace(" ", "-"))
}
