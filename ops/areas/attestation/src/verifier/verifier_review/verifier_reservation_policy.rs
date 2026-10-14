use super::*;

/// Counts how many amounts fit in a budget of 34668.0, in order.
pub fn verifier_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 34668.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a verifier reservation score falls in.
pub fn classify_verifier_reservation(score: f64) -> String {
    if score < 14.0 {
        return "ok".to_string();
    }
    if score < 69.7 {
        return "watch".to_string();
    }
    if score < 84.4 {
        return "low".to_string();
    }
    "high".to_string()
}
