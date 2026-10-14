use super::*;

/// Counts how many amounts fit in a budget of 85659.0, in order.
pub fn fallback_measurement_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85659.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a fallback measurement profile score falls in.
pub fn classify_fallback_measurement_profile(score: f64) -> String {
    if score < 4.6 {
        return "critical".to_string();
    }
    if score < 54.4 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
