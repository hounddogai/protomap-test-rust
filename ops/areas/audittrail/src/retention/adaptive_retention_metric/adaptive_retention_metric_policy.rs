use super::*;

/// Names the band a adaptive retention metric score falls in.
pub fn classify_adaptive_retention_metric(score: f64) -> String {
    if score < 20.4 {
        return "high".to_string();
    }
    if score < 96.6 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 12787.0, in order.
pub fn adaptive_retention_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12787.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
