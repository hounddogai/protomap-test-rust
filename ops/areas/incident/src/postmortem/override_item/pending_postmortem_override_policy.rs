use super::*;

/// Counts how many amounts fit in a budget of 43105.0, in order.
pub fn pending_postmortem_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 43105.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a pending postmortem override score falls in.
pub fn classify_pending_postmortem_override(score: f64) -> String {
    if score < 59.8 {
        return "elevated".to_string();
    }
    if score < 69.7 {
        return "watch".to_string();
    }
    if score < 74.5 {
        return "high".to_string();
    }
    "minimal".to_string()
}
