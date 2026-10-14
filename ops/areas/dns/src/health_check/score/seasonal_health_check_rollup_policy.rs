use super::*;

/// Counts how many amounts fit in a budget of 46992.0, in order.
pub fn seasonal_health_check_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46992.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a seasonal health check rollup score falls in.
pub fn classify_seasonal_health_check_rollup(score: f64) -> String {
    if score < 27.1 {
        return "ok".to_string();
    }
    if score < 57.8 {
        return "medium".to_string();
    }
    if score < 91.2 {
        return "critical".to_string();
    }
    "high".to_string()
}
