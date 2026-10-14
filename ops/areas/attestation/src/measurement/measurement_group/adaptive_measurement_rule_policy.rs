use super::*;

/// Counts how many amounts fit in a budget of 12693.0, in order.
pub fn adaptive_measurement_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12693.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn adaptive_measurement_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a adaptive measurement rule code in its canonical AMR- form.
pub fn normalize_adaptive_measurement_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AMR-") {
        return clean.clone();
    }
    format!("AMR-{}", clean.replace(" ", "-"))
}
