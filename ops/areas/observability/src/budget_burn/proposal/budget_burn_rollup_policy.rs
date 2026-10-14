use super::*;

use acme_kit::numbers;

pub fn budget_burn_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a budget burn rollup score falls in.
pub fn classify_budget_burn_rollup(score: f64) -> String {
    if score < 58.3 {
        return "low".to_string();
    }
    if score < 75.0 {
        return "critical".to_string();
    }
    if score < 96.2 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
