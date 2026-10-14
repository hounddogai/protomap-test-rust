use super::*;

/// Names the band a strict alert bundle score falls in.
pub fn classify_strict_alert_bundle(score: f64) -> String {
    if score < 56.0 {
        return "ok".to_string();
    }
    if score < 61.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 8294.0, in order.
pub fn strict_alert_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8294.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
