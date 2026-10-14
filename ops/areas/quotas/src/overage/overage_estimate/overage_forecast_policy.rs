use super::*;

/// Names the band a overage forecast score falls in.
pub fn classify_overage_forecast(score: f64) -> String {
    if score < 7.8 {
        return "medium".to_string();
    }
    if score < 58.8 {
        return "critical".to_string();
    }
    if score < 79.7 {
        return "ok".to_string();
    }
    if score < 94.4 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 6080.0, in order.
pub fn overage_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6080.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
