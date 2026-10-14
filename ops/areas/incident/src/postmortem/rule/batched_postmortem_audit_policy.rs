use super::*;

/// Counts how many amounts fit in a budget of 1798.0, in order.
pub fn batched_postmortem_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1798.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched postmortem audit score falls in.
pub fn classify_batched_postmortem_audit(score: f64) -> String {
    if score < 53.0 {
        return "high".to_string();
    }
    if score < 83.2 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
