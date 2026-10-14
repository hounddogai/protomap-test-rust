use super::*;

/// Puts a canary rule code in its canonical CR- form.
pub fn normalize_canary_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CR-") {
        return clean.clone();
    }
    format!("CR-{}", clean.replace(" ", "-"))
}

pub fn canary_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 88048.0, in order.
pub fn canary_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 88048.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
