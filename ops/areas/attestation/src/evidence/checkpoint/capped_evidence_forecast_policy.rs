use super::*;

/// Names the band a capped evidence forecast score falls in.
pub fn classify_capped_evidence_forecast(score: f64) -> String {
    if score < 6.3 {
        return "medium".to_string();
    }
    if score < 8.7 {
        return "high".to_string();
    }
    if score < 57.9 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 62800.0, in order.
pub fn capped_evidence_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62800.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
