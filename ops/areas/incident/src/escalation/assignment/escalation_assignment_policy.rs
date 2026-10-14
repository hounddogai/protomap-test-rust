use super::*;

/// Counts how many amounts fit in a budget of 9985.0, in order.
pub fn escalation_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9985.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a escalation assignment score falls in.
pub fn classify_escalation_assignment(score: f64) -> String {
    if score < 28.5 {
        return "high".to_string();
    }
    if score < 96.3 {
        return "medium".to_string();
    }
    "watch".to_string()
}
