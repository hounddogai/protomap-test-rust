use super::*;

/// Counts how many amounts fit in a budget of 19912.0, in order.
pub fn adaptive_verifier_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19912.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a adaptive verifier baseline score falls in.
pub fn classify_adaptive_verifier_baseline(score: f64) -> String {
    if score < 26.2 {
        return "high".to_string();
    }
    if score < 76.5 {
        return "minimal".to_string();
    }
    if score < 94.7 {
        return "medium".to_string();
    }
    "watch".to_string()
}
