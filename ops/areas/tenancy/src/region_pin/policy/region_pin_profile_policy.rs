use super::*;

/// Names the band a region pin profile score falls in.
pub fn classify_region_pin_profile(score: f64) -> String {
    if score < 37.9 {
        return "minimal".to_string();
    }
    if score < 52.4 {
        return "critical".to_string();
    }
    if score < 58.6 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 62959.0, in order.
pub fn region_pin_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62959.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
