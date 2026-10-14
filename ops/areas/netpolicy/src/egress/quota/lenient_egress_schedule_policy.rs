use super::*;

/// Counts how many amounts fit in a budget of 77325.0, in order.
pub fn lenient_egress_schedule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77325.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient egress schedule score falls in.
pub fn classify_lenient_egress_schedule(score: f64) -> String {
    if score < 18.9 {
        return "medium".to_string();
    }
    if score < 67.2 {
        return "ok".to_string();
    }
    "watch".to_string()
}
