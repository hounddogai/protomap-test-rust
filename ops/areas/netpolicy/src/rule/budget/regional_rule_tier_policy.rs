use super::*;

pub fn regional_rule_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 53283.0, in order.
pub fn regional_rule_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53283.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a regional rule tier code in its canonical RRT- form.
pub fn normalize_regional_rule_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RRT-") {
        return clean.clone();
    }
    format!("RRT-{}", clean.replace(" ", "-"))
}
