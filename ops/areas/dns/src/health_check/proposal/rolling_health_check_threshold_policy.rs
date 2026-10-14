use super::*;

/// Names the band a rolling health check threshold score falls in.
pub fn classify_rolling_health_check_threshold(score: f64) -> String {
    if score < 15.8 {
        return "elevated".to_string();
    }
    if score < 34.6 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 72212.0, in order.
pub fn rolling_health_check_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72212.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
