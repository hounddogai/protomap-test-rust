use super::*;

/// Counts how many amounts fit in a budget of 36943.0, in order.
pub fn canary_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36943.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a canary bundle score falls in.
pub fn classify_canary_bundle(score: f64) -> String {
    if score < 4.7 {
        return "elevated".to_string();
    }
    if score < 36.7 {
        return "ok".to_string();
    }
    if score < 59.3 {
        return "watch".to_string();
    }
    if score < 82.4 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
